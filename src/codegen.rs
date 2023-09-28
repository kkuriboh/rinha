use std::collections::HashMap;

use micromap::Map;

use crate::expr::{Expr, Ident};

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

pub struct Codegen {
	main_func: Vec<String>,
	builtins: Map<&'static str, &'static str, 3>,
	variables: HashMap<String, String>,
}

// XXX: too buggy, need a whole rewrite
impl Codegen {
	const STD: &'static str = concat!('\n', include_str!("../std.hvm"));

	pub fn new() -> Self {
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

	pub fn transpile(mut self, expr: Expr) -> String {
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
			Expr::Let { name, value, next } if depth == 0 => {
				let Ident(name_) = name;

				match *value {
					Expr::Abstraction { args, body } => {
						let name = ToPascalCase(name_.clone());
						self.variables.insert(name_, name.clone());

						for arg in args.iter() {
							self.variables.insert(arg.0.clone(), arg.0.clone());
						}

						let body = self.transpile_expr(*body, 1);

						match next.as_ref() {
							Expr::Let { value, .. }
								if matches!(value.as_ref(), Expr::Abstraction { .. }) =>
							{
								let next = self.transpile_expr(*next, depth);
								format!(
									"({name}{}) = (STD.closure ({}))\n{next}",
									args.iter().fold(String::new(), |mut acc, ident| {
										acc.push(' ');
										acc.push_str(ident.val());
										acc
									}),
									body
								)
							}
							_ => {
								let next = self.transpile_expr(*next, depth);
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
						}
					}
					expr => {
						self.variables.insert(name_.clone(), name_.clone());
						let next = self.transpile_expr(*next, depth);

						// XXX: terrible workaround
						// so i don't need to handle closures in global scope
						// (only works for literals)
						match &expr {
							Expr::Str(_) | Expr::Int(_) | Expr::Bool(_) => {
								let literal = self.transpile_expr(expr, depth + 1);
								#[cfg(debug_assertions)]
								self.main_func.push(format!("let {name_} = {literal};\n\t"));
								#[cfg(not(debug_assertions))]
								self.main_func.push(format!("let {name_} = {literal};"));
								return format!("let {name_} = {literal};\n{next}");
							}
							_ => {}
						}

						let val = self.transpile_expr(expr, depth + 1);

						#[cfg(not(debug_assertions))]
						self.main_func.push(format!("let {name_} = {val};"));
						#[cfg(debug_assertions)]
						self.main_func.push(format!("let {name_} = {val};\n\t"));
						next
					}
				}
			}
			expr if depth == 0 => match &expr {
				Expr::Int(_)
				| Expr::Str(_)
				| Expr::Bool(_)
				| Expr::Variable(_)
				| Expr::Binary { .. }
				| Expr::Application { .. }
				| Expr::If { .. }
				| Expr::Tuple(_, _) => {
					let ret = self.transpile_expr(expr, depth + 1);
					self.main_func.push(ret);
					String::new()
				}
				_ => unreachable!(),
			},
			Expr::Int(i) => format!(
				"(STD.int {})",
				if i < 0 {
					2147483647 + (-i) as u32
				} else {
					i as u32
				}
			),
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
					"(STD.if ({}) ({}) ({}))",
					self.transpile_expr(*condition, depth + 1),
					self.transpile_expr(*then, depth + 1),
					self.transpile_expr(*otherwise, depth + 1)
				)
			}
			Expr::Let { name, value, next } => {
				let Ident(name_) = name;

				self.variables.insert(name_.clone(), name_.clone());
				let val = self.transpile_expr(*value, depth + 1);
				let next = self.transpile_expr(*next, depth);

				#[cfg(not(debug_assertions))]
				return format!("let {name_} = {};{}", val, next);
				#[cfg(debug_assertions)]
				return format!("let {name_} = {};\n\t{}", val, next);
			}
			Expr::Application { callee, args } => {
				let args = args
					.into_iter()
					.map(|v| self.transpile_expr(v, depth))
					.collect::<Box<[String]>>()
					.join(" ");

				let callee = 'id: {
					match *callee {
						Expr::Variable(var) => match self.builtins.get(var.val()) {
							Some(fn_name) => return format!("({fn_name} {args})"),
							None => break 'id self.variables.get(var.val()).unwrap().to_string(),
						},
						expr => break 'id self.transpile_expr(expr, depth + 1),
					}
				};

				format!("(STD.call ({} {}))", callee, args)
			}
			Expr::Abstraction { args, body } => {
				for arg in args.iter() {
					self.variables.insert(arg.0.clone(), arg.0.clone());
				}

				format!(
					"(STD.closure {}({}))",
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
