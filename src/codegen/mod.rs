pub mod context;
pub mod expr;
pub mod function;

use anyhow::Result;
use inkwell::values::ValueKind;

use context::CodegenContext;

/// Add a C-compatible `main()` wrapper that calls the mangled entry point.
/// `entry_main_name` is the mangled name, e.g. "main$$main$".
pub fn add_c_main_wrapper(codegen_ctx: &mut CodegenContext, entry_main_name: &str) -> Result<()> {
    let entry_fn = codegen_ctx
        .module
        .get_function(entry_main_name)
        .ok_or_else(|| anyhow::anyhow!("{} function not found", entry_main_name))?;

    // Create C-compatible main function: int main()
    let c_main_type = codegen_ctx.context.i32_type().fn_type(&[], false);
    let c_main = codegen_ctx.module.add_function("main", c_main_type, None);

    let entry = codegen_ctx.context.append_basic_block(c_main, "entry");
    codegen_ctx.builder.position_at_end(entry);

    let result = codegen_ctx
        .builder
        .build_call(entry_fn, &[], "call_main")
        .map_err(|e| anyhow::anyhow!("Failed to build call: {:?}", e))?;
    let return_val = match result.try_as_basic_value() {
        ValueKind::Basic(val) => val,
        ValueKind::Instruction(_) => anyhow::bail!("Main function must return a value"),
    };

    // Cast i64 to i32 for C main
    let i32_result = codegen_ctx
        .builder
        .build_int_cast(
            return_val.into_int_value(),
            codegen_ctx.context.i32_type(),
            "cast_to_i32",
        )
        .map_err(|e| anyhow::anyhow!("Failed to build cast: {:?}", e))?;

    codegen_ctx
        .builder
        .build_return(Some(&i32_result))
        .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;

    Ok(())
}
