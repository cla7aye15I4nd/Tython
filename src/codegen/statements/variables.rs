use crate::tir::{CallTarget, TirExpr, ValueType};

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_let_stmt(&mut self, name: &str, ty: &ValueType, value: &TirExpr) {
        let value_llvm = self.codegen_expr(value);

        if let Some(&existing_ptr) = self.variables.get(name) {
            emit!(self.build_store(existing_ptr, value_llvm));
        } else {
            let alloca = self.build_entry_block_alloca(self.get_llvm_type(ty), name);
            emit!(self.build_store(alloca, value_llvm));
            self.variables.insert(name.to_string(), alloca);
        }
    }

    pub(crate) fn codegen_return_stmt(&mut self, expr_opt: &Option<TirExpr>) {
        if let Some(expr) = expr_opt {
            let value = self.codegen_expr(expr);
            emit!(self.build_return(Some(&value)));
        } else {
            emit!(self.build_return(None));
        }
    }

    pub(crate) fn codegen_void_call_stmt(&mut self, target: &CallTarget, args: &[TirExpr]) {
        match target {
            CallTarget::Named(func_name) => {
                self.codegen_named_call(func_name, args, None);
            }
            CallTarget::Builtin(builtin_fn) => {
                self.codegen_builtin_call(*builtin_fn, args, None);
            }
        }
    }

    pub(crate) fn codegen_set_field_stmt(
        &mut self,
        object: &TirExpr,
        class_name: &str,
        field_index: usize,
        value: &TirExpr,
    ) {
        let obj_ptr = self.codegen_expr(object).into_pointer_value();
        let struct_type = self.struct_types[class_name];

        let field_ptr =
            emit!(self.build_struct_gep(struct_type, obj_ptr, field_index as u32, "field_ptr"));

        let val = self.codegen_expr(value);
        emit!(self.build_store(field_ptr, val));
    }

    pub(crate) fn codegen_list_set_stmt(
        &mut self,
        list: &TirExpr,
        index: &TirExpr,
        value: &TirExpr,
    ) {
        let list_val = self.codegen_expr(list);
        let index_val = self.codegen_expr(index);
        let elem_val = self.codegen_expr(value);
        let i64_val = self.bitcast_to_i64(elem_val, &value.ty);
        let list_set_fn = self.get_runtime_fn(RuntimeFn::ListSet);
        emit!(self.build_call(
            list_set_fn,
            &[list_val.into(), index_val.into(), i64_val.into()],
            "list_set",
        ));
    }
}
