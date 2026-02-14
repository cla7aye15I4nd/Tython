use inkwell::values::BasicValueEnum;

use crate::tir::{TirExpr, TirExprKind};

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_expr(&mut self, expr: &TirExpr) -> BasicValueEnum<'ctx> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => self.i64_type().const_int(*val as u64, false).into(),
            TirExprKind::FloatLiteral(val) => self.f64_type().const_float(*val).into(),
            TirExprKind::BoolLiteral(val) => self.bool_type().const_int(*val as u64, false).into(),
            TirExprKind::StrLiteral(s) => self.codegen_str_literal(s),
            TirExprKind::BytesLiteral(bytes) => self.codegen_bytes_literal(bytes),
            TirExprKind::Var(name) => self.codegen_var_load(name, &expr.ty),

            // Integer arithmetic
            TirExprKind::IntAdd(left, right) => self.codegen_int_add(left, right),
            TirExprKind::IntSub(left, right) => self.codegen_int_sub(left, right),
            TirExprKind::IntMul(left, right) => self.codegen_int_mul(left, right),
            TirExprKind::IntFloorDiv(left, right) => self.codegen_int_floor_div(left, right),
            TirExprKind::IntMod(left, right) => self.codegen_int_mod(left, right),
            TirExprKind::IntPow(left, right) => self.codegen_int_pow(left, right),

            // Float arithmetic
            TirExprKind::FloatAdd(left, right) => self.codegen_float_add(left, right),
            TirExprKind::FloatSub(left, right) => self.codegen_float_sub(left, right),
            TirExprKind::FloatMul(left, right) => self.codegen_float_mul(left, right),
            TirExprKind::FloatDiv(left, right) => self.codegen_float_div(left, right),
            TirExprKind::FloatFloorDiv(left, right) => self.codegen_float_floor_div(left, right),
            TirExprKind::FloatMod(left, right) => self.codegen_float_mod(left, right),
            TirExprKind::FloatPow(left, right) => self.codegen_float_pow(left, right),

            // Bitwise operations
            TirExprKind::BitAnd(left, right) => self.codegen_bitwise_and(left, right),
            TirExprKind::BitOr(left, right) => self.codegen_bitwise_or(left, right),
            TirExprKind::BitXor(left, right) => self.codegen_bitwise_xor(left, right),
            TirExprKind::LShift(left, right) => self.codegen_left_shift(left, right),
            TirExprKind::RShift(left, right) => self.codegen_right_shift(left, right),

            // Unary operations
            TirExprKind::IntNeg(operand) => self.codegen_int_neg(operand),
            TirExprKind::FloatNeg(operand) => self.codegen_float_neg(operand),
            TirExprKind::Not(operand) => self.codegen_not(operand),
            TirExprKind::BitNot(operand) => self.codegen_bit_not(operand),

            // Integer comparisons
            TirExprKind::IntEq(left, right) => self.codegen_int_eq(left, right),
            TirExprKind::IntNotEq(left, right) => self.codegen_int_ne(left, right),
            TirExprKind::IntLt(left, right) => self.codegen_int_lt(left, right),
            TirExprKind::IntLtEq(left, right) => self.codegen_int_le(left, right),
            TirExprKind::IntGt(left, right) => self.codegen_int_gt(left, right),
            TirExprKind::IntGtEq(left, right) => self.codegen_int_ge(left, right),

            // Float comparisons
            TirExprKind::FloatEq(left, right) => self.codegen_float_eq(left, right),
            TirExprKind::FloatNotEq(left, right) => self.codegen_float_ne(left, right),
            TirExprKind::FloatLt(left, right) => self.codegen_float_lt(left, right),
            TirExprKind::FloatLtEq(left, right) => self.codegen_float_le(left, right),
            TirExprKind::FloatGt(left, right) => self.codegen_float_gt(left, right),
            TirExprKind::FloatGtEq(left, right) => self.codegen_float_ge(left, right),

            // Bool comparisons
            TirExprKind::BoolEq(left, right) => self.codegen_bool_eq(left, right),
            TirExprKind::BoolNotEq(left, right) => self.codegen_bool_ne(left, right),

            // Logical operations
            TirExprKind::LogicalAnd(left, right) => self.codegen_logical_and(left, right, &expr.ty),
            TirExprKind::LogicalOr(left, right) => self.codegen_logical_or(left, right, &expr.ty),

            TirExprKind::Call { func, args } => {
                self.codegen_named_call(func, args, Some(&expr.ty)).unwrap()
            }
            TirExprKind::ExternalCall { func, args } => self
                .codegen_builtin_call(*func, args, Some(&expr.ty))
                .unwrap(),
            TirExprKind::IntrinsicCmp { op, lhs, rhs } => self.codegen_intrinsic_cmp(*op, lhs, rhs),
            TirExprKind::Cast { kind, arg } => self.codegen_cast(kind, arg),
            TirExprKind::Construct {
                class_name,
                init_mangled_name,
                args,
            } => self.codegen_construct(class_name, init_mangled_name, args),
            TirExprKind::GetField {
                object,
                field_index,
            } => self.codegen_get_field(object, *field_index, &expr.ty),
            TirExprKind::TupleLiteral {
                elements,
                element_types,
            } => self.codegen_tuple_literal(elements, element_types),
            TirExprKind::ListLiteral {
                element_type,
                elements,
            } => self.codegen_list_literal(element_type, elements),
        }
    }
}
