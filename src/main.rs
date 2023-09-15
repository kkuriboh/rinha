use std::time::SystemTime;

mod compiler;
mod expr;
mod json;
mod parser;

fn main() {
	let now = SystemTime::now();
	parser::parse("test_files/list.json").unwrap();
	println!("{}", now.elapsed().unwrap().as_nanos())
}