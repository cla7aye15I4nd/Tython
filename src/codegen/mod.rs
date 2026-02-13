use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::StructType;
use inkwell::values::PointerValue;
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Shorthand for `self.builder.method(...).unwrap()`.
/// Usage: `emit!(self.build_int_add(l, r, "add"))`
macro_rules! emit {
    ($s:ident . $method:ident ( $($arg:expr),* $(,)? )) => {
        $s.builder.$method( $($arg),* ).unwrap()
    };
}

// Note: predicate_map macro removed - we now use TypedCompare which encodes
// both the comparison operator and operand type, eliminating the need for
// runtime type dispatch in codegen.

/// Push loop context, codegen body statements, pop, branch to continue target.
macro_rules! loop_body {
    ($self:ident, $continue_bb:expr, $break_bb:expr, $body:expr) => {{
        $self.loop_stack.push(($continue_bb, $break_bb));
        for s in $body {
            $self.codegen_stmt(s);
        }
        $self.loop_stack.pop();
        $self.branch_if_unterminated($continue_bb);
    }};
}

/// Codegen an optional else block, then branch to after_bb.
macro_rules! else_body {
    ($self:ident, $else_bb:expr, $stmts:expr, $after_bb:expr) => {
        if let Some(else_bb) = $else_bb {
            $self.builder.position_at_end(else_bb);
            for s in $stmts {
                $self.codegen_stmt(s);
            }
            $self.branch_if_unterminated($after_bb);
        }
    };
}

mod exceptions;
mod expressions;
mod helpers_calls;
mod helpers_flow;
mod helpers_types;
mod runtime_fn;
mod statements;

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    global_variables: HashMap<String, PointerValue<'ctx>>,
    loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    struct_types: HashMap<String, StructType<'ctx>>,
    /// > 0 when inside a try/except or ForIter — calls use `invoke` instead of `call`.
    try_depth: usize,
    /// Stack of unwind destinations for nested try/ForIter blocks.
    unwind_dest_stack: Vec<BasicBlock<'ctx>>,
    /// Saved exception state for bare `raise` inside except handlers.
    /// (type_tag_alloca, message_ptr_alloca)
    reraise_state: Option<(PointerValue<'ctx>, PointerValue<'ctx>)>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native target");

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &triple,
                "",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .expect("Failed to create target machine");

        let module = context.create_module("__main__");
        module.set_triple(&triple);
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            global_variables: HashMap::new(),
            loop_stack: Vec::new(),
            struct_types: HashMap::new(),
            try_depth: 0,
            unwind_dest_stack: Vec::new(),
            reraise_state: None,
        }
    }

    const RUNTIME_BC_NAIVE: &'static str = env!("RUNTIME_BC_PATH_NAIVE");
    const RUNTIME_BC_BOEHM: &'static str = env!("RUNTIME_BC_PATH_BOEHM");

    pub fn link(&self, output_path: &Path) {
        let bc_path = output_path.with_extension("o");

        self.module.write_bitcode_to_path(&bc_path);

        // Select runtime based on TYTHON_GC environment variable
        let gc_type = std::env::var("TYTHON_GC").unwrap_or_else(|_| "boehm".to_string());
        let runtime_bc = match gc_type.as_str() {
            "naive" => Self::RUNTIME_BC_NAIVE,
            "boehm" => Self::RUNTIME_BC_BOEHM,
            _ => {
                eprintln!(
                    "Warning: Unknown TYTHON_GC value '{}', defaulting to 'boehm'",
                    gc_type
                );
                Self::RUNTIME_BC_BOEHM
            }
        };

        let mut cmd = Command::new("clang++");
        cmd.arg("-static")
            .arg("-flto")
            .arg("-O2")
            .arg("-o")
            .arg(output_path)
            .arg(&bc_path)
            .arg(runtime_bc);

        // Add libraries AFTER object files (linking order matters)
        cmd.arg("-lm");

        // Add Boehm GC library if using boehm
        if gc_type == "boehm" {
            cmd.arg("-lgc");
        }

        cmd.arg("-lpthread").arg("-ldl");

        let output = cmd.output().expect("Failed to execute clang++");
        if !output.status.success() {
            eprintln!("Linker error:\n{}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to link with runtime");
        }
    }
}
