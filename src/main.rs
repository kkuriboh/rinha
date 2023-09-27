use std::{collections::HashMap, fs::File};

mod codegen;
mod expr;
mod json;
mod parser;

pub type Context<'ctx, T> = HashMap<&'ctx str, T>;

fn main() {
	let now = std::time::SystemTime::now();
	#[cfg(debug_assertions)]
	let file_path = {
		let mut args = std::env::args();
		args.nth(1).unwrap()
	};
	#[cfg(not(debug_assertions))]
	let file_path = env!("FILE_PATH");

	let file = parser::parse(file_path);

	codegen::Codegen::new(file.expr, File::create("main.hvm").unwrap()).transpile();
	println!("{}", now.elapsed().unwrap().as_micros());
}
