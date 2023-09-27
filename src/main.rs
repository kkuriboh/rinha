mod codegen;
mod expr;
mod json;
mod parser;

fn main() {
	#[cfg(debug_assertions)]
	let file_path = {
		let mut args = std::env::args();
		args.nth(1).unwrap()
	};
	#[cfg(not(debug_assertions))]
	let file_path = env!("FILE_PATH");

	let file = parser::parse(file_path);

	let mut code = codegen::Codegen::new(file.expr).transpile();
	code.push_str("\nHVM_MAIN_CALL = Main");

	// #[cfg(debug_assertions)]
	std::fs::write("main.hvm", code.as_bytes()).unwrap();

	let tids = hvm::runtime::default_heap_tids();

	let file = hvm::language::syntax::read_file(&code).unwrap();

	let book = hvm::language::rulebook::gen_rulebook(&file);
	let mut prog = hvm::runtime::Program::new();
	prog.add_book(&book);
	// prog.add_function(
	// 	"STD.stringify".to_owned(),
	// 	hvm::runtime::Function::Compiled {
	// 		smap: Box::new([true]),
	// 		visit: |_| false,
	// 		apply: |ctx| {
	// 			let arg0 = hvm::runtime::get_loc(ctx.term, 0);
	// 			if let Some(term) =
	// 				hvm::language::readback::as_string(ctx.heap, ctx.prog, &[ctx.tid], arg0)
	// 			{
	// 				let text = hvm::runtime::make_string(ctx.heap, ctx.tid, &term);
	// 				hvm::runtime::link(ctx.heap, *ctx.host, text);
	// 				hvm::runtime::collect(
	// 					ctx.heap,
	// 					&ctx.prog.aris,
	// 					ctx.tid,
	// 					hvm::runtime::load_ptr(ctx.heap, arg0),
	// 				);
	// 				hvm::runtime::free(ctx.heap, ctx.tid, hvm::get_loc(ctx.term, 0), 1);
	// 			};
	//
	// 			return true;
	// 		},
	// 	},
	// );

	let heap = hvm::runtime::new_heap(hvm::runtime::default_heap_size(), tids);
	let tids = hvm::runtime::new_tids(tids);

	hvm::runtime::link(
		&heap,
		0,
		hvm::runtime::Fun(*book.name_to_id.get("HVM_MAIN_CALL").unwrap(), 0),
	);

	let host = 0;

	hvm::runtime::normalize(&heap, &prog, &tids, host, false);

	#[cfg(debug_assertions)]
	let code = format!("{}", hvm::language::readback::as_term(&heap, &prog, host));

	hvm::runtime::collect(
		&heap,
		&prog.aris,
		tids[0],
		hvm::runtime::load_ptr(&heap, host),
	);
	hvm::runtime::free(&heap, 0, 0, 1);

	#[cfg(debug_assertions)]
	println!("{code}");
}
