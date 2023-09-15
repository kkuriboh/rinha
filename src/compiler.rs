#![allow(unused)]

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Module};

use crate::parser::File;

struct Jit {
	builder_context: FunctionBuilderContext,
	ctx: codegen::Context,
	data_description: DataDescription,
	module: JITModule,
}

impl Default for Jit {
	fn default() -> Self {
		let mut flag_builder = settings::builder();
		flag_builder.set("use_colocated_libcalls", "false").unwrap();
		flag_builder.set("is_pic", "false").unwrap();
		let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
			panic!("host machine is not supported: {}", msg);
		});
		let isa = isa_builder
			.finish(settings::Flags::new(flag_builder))
			.unwrap();
		let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
		let module = JITModule::new(builder);

		Self {
			builder_context: FunctionBuilderContext::new(),
			ctx: module.make_context(),
			data_description: DataDescription::new(),
			module,
		}
	}
}
