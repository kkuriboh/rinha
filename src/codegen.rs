use std::{collections::HashMap, io::Write};

use micromap::Map;

use crate::expr::{Expr, Ident};

pub struct Codegen<T: Write> {
	expr: Expr,
	buffer: T,
}

impl<T: Write> Codegen<T> {
	pub fn new(expr: Expr, buffer: T) -> Self {
		Self { expr, buffer }
	}

	pub fn transpile(mut self) {
		let code = Transpiler::new().transpile(self.expr);
		self.buffer.write_all(code.as_bytes()).unwrap();
	}
}

#[allow(non_snake_case)] // just for the luls
fn ToPascalCase(string: String) -> String {
	let mut chars = string.chars();
	let mut ret = String::with_capacity(string.capacity());

	if let Some(char) = chars.next() {
		ret.push(char.to_ascii_uppercase())
	}

	while let Some(char) = chars.next() {
		if char == '_' {
			if let Some(char) = chars.next() {
				ret.push(char.to_ascii_uppercase())
			}
			continue;
		}

		ret.push(char);
	}

	ret
}

struct Transpiler {
	main_func: Vec<String>,
	builtins: Map<&'static str, &'static str, 3>,
	variables: HashMap<String, String>,
}

impl Transpiler {
	const STD: &'static str = concat!('\n', include_str!("../std.hvm"));

	fn new() -> Self {
		let builtins = Map::from([
			("print", "STD.print"),
			("first", "STD.first"),
			("second", "STD.second"),
		]);

		Self {
			main_func: vec![],
			builtins,
			variables: HashMap::from([
				("print".into(), "STD.print".into()),
				("first".into(), "STD.first".into()),
				("second".into(), "STD.second".into()),
			]),
		}
	}

	fn transpile(mut self, expr: Expr) -> String {
		let mut code = self.transpile_expr(expr, 0);
		code.push_str(Self::STD);
		code.push_str(&format!(
			"(Main) = ({})",
			self.main_func.into_iter().rev().collect::<String>()
		));
		code
	}

	fn transpile_expr(&mut self, expr: Expr, depth: usize) -> String {
		match expr {
			Expr::Int(i) => format!("(STD.int {i})"),
			Expr::Str(s) => format!("{s:?}"),
			Expr::Bool(true) => format!("(STD.bool 1)"),
			Expr::Bool(false) => format!("(STD.bool 0)"),
			Expr::Variable(v) => self.variables.get(v.val()).unwrap().clone(),
			Expr::Binary { lhs, op, rhs } => {
				let lhs = self.transpile_expr(*lhs, depth + 1);
				let rhs = self.transpile_expr(*rhs, depth + 1);
				format!("({op} {lhs} {rhs})")
			}
			Expr::If {
				condition,
				then,
				otherwise,
			} => {
				format!(
					"(STD.if ({}) @_({}) @_({}))",
					self.transpile_expr(*condition, depth + 1),
					self.transpile_expr(*then, depth + 1),
					self.transpile_expr(*otherwise, depth + 1)
				)
			}
			Expr::Let { name, value, next } => {
				let Ident(name_) = name;

				match *value {
					Expr::Abstraction { args, body } if depth == 0 => {
						let name = ToPascalCase(name_.clone());
						self.variables.insert(name_, name.clone());
						let next = self.transpile_expr(*next, depth);

						for arg in args.iter() {
							self.variables.insert(arg.0.clone(), arg.0.clone());
						}

						let body = self.transpile_expr(*body, 1);
						self.main_func.insert(0, next);

						format!(
							"({name}{}) = (STD.closure ({}))",
							args.iter().fold(String::new(), |mut acc, ident| {
								acc.push(' ');
								acc.push_str(ident.val());
								acc
							}),
							body
						)
					}
					expr => {
						self.variables.insert(name_.clone(), name_.clone());
						let next = self.transpile_expr(*next, depth);

						// XXX: terrible workaround
						// so i don't need to handle closures in global scope
						// (only works for literals)
						match &expr {
							Expr::Str(_) | Expr::Int(_) | Expr::Bool(_) => {
								let literal = self.transpile_expr(expr, depth);
								#[cfg(debug_assertions)]
								self.main_func.push(format!("let {name_} = {literal};\n\t"));
								#[cfg(not(debug_assertions))]
								self.main_func.push(format!("let {name_} = {literal};"));
								return format!("let {name_} = {literal};\n{next}");
							}
							_ => {}
						}

						let val = self.transpile_expr(expr, depth + 1);

						if depth == 0 {
							#[cfg(not(debug_assertions))]
							self.main_func.push(format!("let {name_} = {val};"));
							#[cfg(debug_assertions)]
							self.main_func.push(format!("let {name_} = {val};\n\t"));
							return next;
						}
						#[cfg(not(debug_assertions))]
						return format!("let {name_} = {};{}", val, next);
						#[cfg(debug_assertions)]
						return format!("let {name_} = {};\n\t{}", val, next);
					}
				}
			}
			Expr::Application { funct, args } => {
				let fn_name = funct.val();

				let args = args
					.into_iter()
					.map(|v| self.transpile_expr(v, depth))
					.collect::<Box<[String]>>()
					.join(" ");

				match self.builtins.get(fn_name) {
					Some(name) => format!("({} {})", name, args),
					None => format!(
						"(STD.call ({} {}))",
						self.variables.get(fn_name).unwrap(),
						args
					),
				}
			}
			Expr::Abstraction { args, body } => {
				for arg in args.iter() {
					self.variables.insert(arg.0.clone(), arg.0.clone());
				}

				format!(
					"(STD.closure ({}({})))",
					args.iter().fold(String::new(), |mut acc, ident| {
						acc.push('@');
						acc.push_str(ident.val());
						acc
					}),
					self.transpile_expr(*body, depth + 1)
				)
			}
			Expr::Tuple(e1, e2) => {
				let depth = depth + 1;
				format!(
					"(Pair {} {})",
					self.transpile_expr(*e1, depth),
					self.transpile_expr(*e2, depth)
				)
			}
		}
	}
}
