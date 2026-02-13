use inkwell::values::BasicValueEnum;

use crate::tir::{TirExpr, TirExprKind};

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_expr(&mut self, expr: &TirExpr) -> BasicValueEnum<'ctx> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => self.i64_type().const_int(*val as u64, false).into(),
            TirExprKind::FloatLiteral(val) => self.f64_type().const_float(*val).into(),
            TirExprKind::StrLiteral(s) => self.codegen_str_literal(s),
            TirExprKind::BytesLiteral(bytes) => self.codegen_bytes_literal(bytes),
            TirExprKind::Var(name) => self.codegen_var_load(name, &expr.ty),
            TirExprKind::BinOp { op, left, right } => self.codegen_bin_op(op, left, right),
            TirExprKind::Call { func, args } => {
                self.codegen_named_call(func, args, Some(&expr.ty)).unwrap()
            }
            TirExprKind::ExternalCall { func, args } => self
                .codegen_builtin_call(*func, args, Some(&expr.ty))
                .unwrap(),
            TirExprKind::Cast { kind, arg } => self.codegen_cast(kind, arg),
            TirExprKind::Compare { op, left, right } => self.codegen_compare(op, left, right),
            TirExprKind::UnaryOp { op, operand } => self.codegen_unary(op, operand),
            TirExprKind::LogicalOp { op, left, right } => {
                self.codegen_logical(op, left, right, &expr.ty)
            }
            TirExprKind::Construct {
                class_name,
                init_mangled_name,
                args,
            } => self.codegen_construct(class_name, init_mangled_name, args),
            TirExprKind::GetField {
                object,
                class_name,
                field_index,
            } => self.codegen_get_field(object, class_name, *field_index, &expr.ty),
            TirExprKind::TupleLiteral {
                elements,
                element_types,
            } => self.codegen_tuple_literal(elements, element_types),
            TirExprKind::TupleGet {
                tuple,
                index,
                element_types,
            } => self.codegen_tuple_get(tuple, *index, element_types, &expr.ty),
            TirExprKind::TupleGetDynamic {
                tuple,
                index,
                len,
                element_types,
            } => self.codegen_tuple_get_dynamic(tuple, index, *len, element_types, &expr.ty),
            TirExprKind::ListLiteral {
                element_type,
                elements,
            } => self.codegen_list_literal(element_type, elements),
        }
    }
}
