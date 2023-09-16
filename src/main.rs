mod compiler;
mod expr;
mod json;
mod parser;

fn main() {
	let mut args = std::env::args();
	let file_path = unsafe { args.nth(1).unwrap_unchecked() };
	let now = std::time::SystemTime::now();
	parser::parse(file_path).unwrap();
	let after = unsafe { now.elapsed().unwrap_unchecked().as_nanos() };
	println!("{after}")
}
