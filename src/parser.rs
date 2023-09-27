use std::path::Path;

use crate::{
	expr::{BinOp, Expr, Ident},
	json::{self, JsonValue},
};

pub struct File {
	pub name: String,
	pub expr: Expr,
}

pub fn parse(file_path: impl AsRef<Path>) -> File {
	let data = std::fs::read_to_string(file_path).unwrap();
	let file = json::run::<winnow::error::ErrorKind>(&mut data.as_str()).unwrap();

	let name = file.extract_object_key(0).extract_str();
	let expr = file.extract_object_key(1).extract_object();
	let expr = parse_expr(&expr);

	File {
		name: name.to_owned(),
		expr,
	}
}

fn parse_expr(expr: &[JsonValue]) -> Expr {
	let kind = expr[0].extract_str();

	match kind {
		"Int" => Expr::Int(expr[1].extract_num()),
		"Str" => Expr::Str(expr[1].extract_str().to_owned()),
		"Bool" => Expr::Bool(expr[1].extract_bool()),
		"Var" => parse_variable(&expr),
		"Binary" => parse_binary(&expr),
		"Let" => parse_let(&expr, parse_expr(&expr[2].extract_object()).into()),
		"If" => parse_if(&expr),
		"Tuple" => parse_tuple(&expr),
		"Call" => parse_application(&expr),
		"Function" => parse_abstraction(&expr),
		"Print" => parse_native(&expr, "print"),
		"First" => parse_native(&expr, "first"),
		"Second" => parse_native(&expr, "second"),
		_ => panic!("Unknown Kind"),
	}
}

#[inline]
fn parse_param(value: &JsonValue) -> Ident {
	Ident::from(value.extract_object_key(0).extract_str().to_owned())
}

#[inline]
fn parse_variable(parent: &[JsonValue]) -> Expr {
	Expr::Variable(Ident::from(parent[1].extract_str().to_owned()))
}

#[inline]
fn parse_binary(parent: &[JsonValue]) -> Expr {
	let lhs: Box<Expr> = parse_expr(&parent[1].extract_object()).into();
	let op = parent[2].extract_str().into();
	let rhs: Box<Expr> = parse_expr(&parent[3].extract_object()).into();

	match (lhs.as_ref(), op, rhs.as_ref()) {
		(Expr::Int(i1), BinOp::Add, Expr::Int(i2)) => Expr::Int(i1 + i2),
		(Expr::Int(i1), BinOp::Sub, Expr::Int(i2)) => Expr::Int(i1 - i2),
		(Expr::Int(i1), BinOp::Div, Expr::Int(i2)) => Expr::Int(i1 / i2),
		(Expr::Int(i1), BinOp::Mul, Expr::Int(i2)) => Expr::Int(i1 * i2),
		(Expr::Int(i1), BinOp::Rem, Expr::Int(i2)) => Expr::Int(i1 % i2),
		(Expr::Int(i1), BinOp::And, Expr::Int(i2)) => Expr::Int(i1 & i2),
		(Expr::Int(i1), BinOp::Or, Expr::Int(i2)) => Expr::Int(i1 | i2),
		(Expr::Int(i1), BinOp::Eq, Expr::Int(i2)) => Expr::Bool(i1 == i2),
		(Expr::Int(i1), BinOp::Neq, Expr::Int(i2)) => Expr::Bool(i1 != i2),
		(Expr::Int(i1), BinOp::Lt, Expr::Int(i2)) => Expr::Bool(i1 < i2),
		(Expr::Int(i1), BinOp::Lte, Expr::Int(i2)) => Expr::Bool(i1 <= i2),
		(Expr::Int(i1), BinOp::Gt, Expr::Int(i2)) => Expr::Bool(i1 > i2),
		(Expr::Int(i1), BinOp::Gte, Expr::Int(i2)) => Expr::Bool(i1 >= i2),
		(Expr::Bool(b1), BinOp::And, Expr::Bool(b2)) => Expr::Bool(b1 & b2),
		(Expr::Bool(b1), BinOp::Or, Expr::Bool(b2)) => Expr::Bool(b1 | b2),
		(Expr::Bool(b1), BinOp::Eq, Expr::Bool(b2)) => Expr::Bool(b1 == b2),
		(Expr::Bool(b1), BinOp::Neq, Expr::Bool(b2)) => Expr::Bool(b1 != b2),
		(Expr::Bool(b1), BinOp::Lt, Expr::Bool(b2)) => Expr::Bool(b1 < b2),
		(Expr::Bool(b1), BinOp::Lte, Expr::Bool(b2)) => Expr::Bool(b1 <= b2),
		(Expr::Bool(b1), BinOp::Gt, Expr::Bool(b2)) => Expr::Bool(b1 > b2),
		(Expr::Bool(b1), BinOp::Gte, Expr::Bool(b2)) => Expr::Bool(b1 >= b2),
		(Expr::Str(s1), BinOp::Add, Expr::Str(s2)) => Expr::Str(format!("{s1}{s2}")),
		(Expr::Int(i1), BinOp::Add, Expr::Str(s2)) => Expr::Str(format!("{i1}{s2}")),
		(Expr::Str(s1), BinOp::Add, Expr::Int(i2)) => Expr::Str(format!("{s1}{i2}")),
		(Expr::Bool(b1), BinOp::Add, Expr::Str(s2)) => Expr::Str(format!("{b1}{s2}")),
		(Expr::Str(s1), BinOp::Add, Expr::Bool(b2)) => Expr::Str(format!("{s1}{b2}")),
		(_, op, _) => Expr::Binary { lhs, op, rhs },
	}
}

#[inline]
fn parse_let(parent: &[JsonValue], value: Box<Expr>) -> Expr {
	let name = parse_param(&parent[1]);
	let next = parse_expr(&parent[3].extract_object()).into();

	Expr::Let { name, value, next }
}

#[inline]
fn parse_if(parent: &[JsonValue]) -> Expr {
	let condition = parse_expr(&parent[1].extract_object()).into();
	let then = parse_expr(&parent[2].extract_object()).into();
	let otherwise = parse_expr(&parent[3].extract_object()).into();

	Expr::If {
		condition,
		then,
		otherwise,
	}
}

#[inline]
fn parse_tuple(parent: &[JsonValue]) -> Expr {
	let first = parse_expr(&parent[1].extract_object()).into();
	let second = parse_expr(&parent[2].extract_object()).into();

	Expr::Tuple(first, second)
}

#[inline]
fn parse_application(parent: &[JsonValue]) -> Expr {
	let Expr::Variable(funct) = parse_variable(&*parent[1].extract_object()) else {
		unsafe { std::hint::unreachable_unchecked() }
	};

	let args = parent[2]
		.extract_array()
		.iter()
		.map(|x| parse_expr(&JsonValue::extract_object(x)))
		.collect();

	Expr::Application { funct, args }
}

#[inline]
fn parse_abstraction(parent: &[JsonValue]) -> Expr {
	let args = parent[1].extract_array().iter().map(parse_param).collect();

	let body = parse_expr(&parent[2].extract_object()).into();

	Expr::Abstraction { args, body }
}

fn parse_native(expr: &[JsonValue], name: &'static str) -> Expr {
	let value = parse_expr(&expr[1].extract_object());

	Expr::Application {
		funct: Ident::from(name.to_string()),
		args: vec![value],
	}
}

#[cfg(test)]
mod tests {
	use crate::expr::{BinOp, Expr, Ident};

	use super::parse;

	#[test]
	fn parse_fib() {
		let file = parse("test_files/fib.json");
		assert_eq!(
			file.expr,
			Expr::Let {
				name: Ident::from("fib".to_string()),
				value: Expr::Abstraction {
					args: vec![Ident::from("n".to_string())],
					body: Expr::If {
						condition: Expr::Binary {
							lhs: Expr::Variable(Ident::from("n".to_string())).into(),
							op: BinOp::Lt,
							rhs: Expr::Int(2).into()
						}
						.into(),
						then: Expr::Variable(Ident::from("n".to_string())).into(),
						otherwise: Expr::Binary {
							lhs: Expr::Application {
								funct: Ident::from("fib".to_string()),
								args: vec![Expr::Binary {
									lhs: Expr::Variable(Ident::from("n".to_string())).into(),
									op: BinOp::Sub,
									rhs: Expr::Int(1).into()
								}]
							}
							.into(),
							op: BinOp::Add,
							rhs: Expr::Application {
								funct: Ident::from("fib".to_string()),
								args: vec![Expr::Binary {
									lhs: Expr::Variable(Ident::from("n".to_string())).into(),
									op: BinOp::Sub,
									rhs: Expr::Int(2).into()
								}]
							}
							.into(),
						}
						.into()
					}
					.into()
				}
				.into(),
				next: Expr::Application {
					funct: Ident::from("print".to_string()),
					args: vec![Expr::Application {
						funct: Ident::from("fib".to_string()),
						args: vec![Expr::Int(10)]
					}]
				}
				.into()
			}
		)
	}
}
