use inkwell::{values::BasicValueEnum, IntPredicate};

use crate::tir::builtin::BuiltinFn;
use crate::tir::{CastKind, LogicalOp, TirExpr, ValueType};

use super::super::Codegen;

// Unified macro for generating binary operation functions
macro_rules! binop_fn {
    // Integer binary operations: add, sub, mul, etc.
    (int $fn_name:ident, $op:ident, $name:expr) => {
        pub(crate) fn $fn_name(&mut self, left: &TirExpr, right: &TirExpr) -> BasicValueEnum<'ctx> {
            let l = self.codegen_expr(left).into_int_value();
            let r = self.codegen_expr(right).into_int_value();
            emit!(self.$op(l, r, $name)).into()
        }
    };

    // Float binary operations: add, sub, mul, etc.
    (float $fn_name:ident, $op:ident, $name:expr) => {
        pub(crate) fn $fn_name(&mut self, left: &TirExpr, right: &TirExpr) -> BasicValueEnum<'ctx> {
            let l = self.codegen_expr(left).into_float_value();
            let r = self.codegen_expr(right).into_float_value();
            emit!(self.$op(l, r, $name)).into()
        }
    };

    // Integer comparisons: lt, le, gt, ge
    (int_cmp $fn_name:ident, $pred:expr, $name:expr) => {
        pub(crate) fn $fn_name(&mut self, left: &TirExpr, right: &TirExpr) -> BasicValueEnum<'ctx> {
            let l = self.codegen_expr(left).into_int_value();
            let r = self.codegen_expr(right).into_int_value();
            emit!(self.build_int_compare($pred, l, r, $name)).into()
        }
    };

    // Float comparisons: eq, ne, lt, le, gt, ge
    (float_cmp $fn_name:ident, $pred:expr, $name:expr) => {
        pub(crate) fn $fn_name(&mut self, left: &TirExpr, right: &TirExpr) -> BasicValueEnum<'ctx> {
            let l = self.codegen_expr(left).into_float_value();
            let r = self.codegen_expr(right).into_float_value();
            emit!(self.build_float_compare($pred, l, r, $name)).into()
        }
    };

    // Bool comparisons (extends to i64 first): eq, ne
    (bool_cmp $fn_name:ident, $pred:expr, $name:expr) => {
        pub(crate) fn $fn_name(&mut self, left: &TirExpr, right: &TirExpr) -> BasicValueEnum<'ctx> {
            let l = self.codegen_expr(left).into_int_value();
            let r = self.codegen_expr(right).into_int_value();
            let left_i64 = emit!(self.build_int_z_extend(l, self.i64_type(), "b_l_i64"));
            let right_i64 = emit!(self.build_int_z_extend(r, self.i64_type(), "b_r_i64"));
            emit!(self.build_int_compare($pred, left_i64, right_i64, $name)).into()
        }
    };

    // Shift operations with extra parameter
    (shift $fn_name:ident, $op:ident, $signed:expr, $name:expr) => {
        pub(crate) fn $fn_name(&mut self, left: &TirExpr, right: &TirExpr) -> BasicValueEnum<'ctx> {
            let l = self.codegen_expr(left).into_int_value();
            let r = self.codegen_expr(right).into_int_value();
            emit!(self.$op(l, r, $signed, $name)).into()
        }
    };
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_cast(&mut self, kind: &CastKind, arg: &TirExpr) -> BasicValueEnum<'ctx> {
        let arg_val = self.codegen_expr(arg);
        match kind {
            CastKind::FloatToInt => emit!(self.build_float_to_signed_int(
                arg_val.into_float_value(),
                self.i64_type(),
                "ftoi",
            ))
            .into(),

            CastKind::IntToFloat => emit!(self.build_signed_int_to_float(
                arg_val.into_int_value(),
                self.f64_type(),
                "itof",
            ))
            .into(),

            CastKind::BoolToFloat => emit!(self.build_signed_int_to_float(
                emit!(self.build_int_z_extend(arg_val.into_int_value(), self.i64_type(), "b2i64")),
                self.f64_type(),
                "btof",
            ))
            .into(),

            CastKind::IntToBool => {
                let cmp = emit!(self.build_int_compare(
                    IntPredicate::NE,
                    arg_val.into_int_value(),
                    self.i64_type().const_int(0, false),
                    "itob",
                ));
                cmp.into()
            }

            CastKind::FloatToBool => {
                let cmp = emit!(self.build_float_compare(
                    inkwell::FloatPredicate::ONE,
                    arg_val.into_float_value(),
                    self.f64_type().const_float(0.0),
                    "ftob",
                ));
                cmp.into()
            }

            CastKind::BoolToInt => {
                emit!(self.build_int_z_extend(arg_val.into_int_value(), self.i64_type(), "btoi"))
                    .into()
            }
        }
    }

    fn codegen_logical(
        &mut self,
        op: &LogicalOp,
        left: &TirExpr,
        right: &TirExpr,
        _result_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        // Both operands are already bool (i1) after TIR lowering
        let left_val = self.codegen_expr(left);
        let left_bool = left_val.into_int_value();
        let left_bb = emit!(self.get_insert_block());

        let right_bb = self.context.append_basic_block(function, "log_right");
        let merge_bb = self.context.append_basic_block(function, "log_merge");

        match op {
            LogicalOp::And => {
                // If left is false, short-circuit; else evaluate right
                emit!(self.build_conditional_branch(left_bool, right_bb, merge_bb));
            }
            LogicalOp::Or => {
                // If left is true, short-circuit; else evaluate right
                emit!(self.build_conditional_branch(left_bool, merge_bb, right_bb));
            }
        }

        // Evaluate right in right_bb
        self.builder.position_at_end(right_bb);
        let right_val = self.codegen_expr(right);
        let right_end_bb = emit!(self.get_insert_block());
        emit!(self.build_unconditional_branch(merge_bb));

        // Merge: phi node selects between two bool (i1) values
        self.builder.position_at_end(merge_bb);
        let phi = emit!(self.build_phi(self.bool_type(), "log_result"));
        phi.add_incoming(&[(&left_val, left_bb), (&right_val, right_end_bb)]);

        phi.as_basic_value()
    }

    // Integer arithmetic
    binop_fn!(int codegen_int_add, build_int_add, "add");
    binop_fn!(int codegen_int_sub, build_int_sub, "sub");
    binop_fn!(int codegen_int_mul, build_int_mul, "mul");

    pub(crate) fn codegen_int_floor_div(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        let div = emit!(self.build_int_signed_div(l, r, "div_tmp"));
        let rem = emit!(self.build_int_signed_rem(l, r, "rem_tmp"));
        let zero = self.i64_type().const_int(0, false);
        let rem_nonzero = emit!(self.build_int_compare(IntPredicate::NE, rem, zero, "rem_nz"));
        let xor_val = emit!(self.build_xor(l, r, "xor_signs"));
        let signs_differ =
            emit!(self.build_int_compare(IntPredicate::SLT, xor_val, zero, "signs_diff"));
        let need_adjust = emit!(self.build_and(rem_nonzero, signs_differ, "need_adj"));
        let adjust = emit!(self.build_int_z_extend(need_adjust, self.i64_type(), "adj_ext"));
        emit!(self.build_int_sub(div, adjust, "floordiv")).into()
    }

    binop_fn!(int codegen_int_mod, build_int_signed_rem, "mod");

    pub(crate) fn codegen_int_pow(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        let pow_fn = self.get_builtin(BuiltinFn::PowInt);
        let call = emit!(self.build_call(pow_fn, &[l.into(), r.into()], "ipow"));
        self.extract_call_value(call).into_int_value().into()
    }

    // Float arithmetic
    binop_fn!(float codegen_float_add, build_float_add, "fadd");
    binop_fn!(float codegen_float_sub, build_float_sub, "fsub");
    binop_fn!(float codegen_float_mul, build_float_mul, "fmul");
    binop_fn!(float codegen_float_div, build_float_div, "fdiv");

    pub(crate) fn codegen_float_floor_div(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        let div = emit!(self.build_float_div(l, r, "fdiv"));
        let f64_ty = self.f64_type();
        let floor_fn =
            self.get_llvm_intrinsic("llvm.floor.f64", f64_ty.fn_type(&[f64_ty.into()], false));
        let call = emit!(self.build_call(floor_fn, &[div.into()], "floordiv"));
        self.extract_call_value(call).into_float_value().into()
    }

    binop_fn!(float codegen_float_mod, build_float_rem, "fmod");

    pub(crate) fn codegen_float_pow(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        let f64_ty = self.f64_type();
        let pow_fn = self.get_llvm_intrinsic(
            "llvm.pow.f64",
            f64_ty.fn_type(&[f64_ty.into(), f64_ty.into()], false),
        );
        let call = emit!(self.build_call(pow_fn, &[l.into(), r.into()], "pow"));
        self.extract_call_value(call).into_float_value().into()
    }

    // Bitwise operations
    binop_fn!(int codegen_bitwise_and, build_and, "bitand");
    binop_fn!(int codegen_bitwise_or, build_or, "bitor");
    binop_fn!(int codegen_bitwise_xor, build_xor, "bitxor");
    binop_fn!(int codegen_left_shift, build_left_shift, "lshift");
    binop_fn!(shift codegen_right_shift, build_right_shift, true, "rshift");

    // Unary operations
    pub(crate) fn codegen_int_neg(&mut self, operand: &TirExpr) -> BasicValueEnum<'ctx> {
        let val = self.codegen_expr(operand).into_int_value();
        let zero = self.i64_type().const_int(0, false);
        emit!(self.build_int_sub(zero, val, "neg")).into()
    }

    pub(crate) fn codegen_float_neg(&mut self, operand: &TirExpr) -> BasicValueEnum<'ctx> {
        let val = self.codegen_expr(operand).into_float_value();
        let zero = self.f64_type().const_float(0.0);
        emit!(self.build_float_sub(zero, val, "fneg")).into()
    }

    pub(crate) fn codegen_not(&mut self, operand: &TirExpr) -> BasicValueEnum<'ctx> {
        let val = self.codegen_expr(operand).into_int_value();
        emit!(self.build_not(val, "not")).into()
    }

    pub(crate) fn codegen_bit_not(&mut self, operand: &TirExpr) -> BasicValueEnum<'ctx> {
        let val = self.codegen_expr(operand).into_int_value();
        let all_ones = self.i64_type().const_all_ones();
        emit!(self.build_xor(val, all_ones, "bitnot")).into()
    }

    // Integer comparisons
    pub(crate) fn codegen_int_eq(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let left_val = self.codegen_expr(left);
        let right_val = self.codegen_expr(right);
        if left.ty.is_ref_type() {
            let left_int = emit!(self.build_ptr_to_int(
                left_val.into_pointer_value(),
                self.i64_type(),
                "ptr_to_int_l"
            ));
            let right_int = emit!(self.build_ptr_to_int(
                right_val.into_pointer_value(),
                self.i64_type(),
                "ptr_to_int_r"
            ));
            emit!(self.build_int_compare(IntPredicate::EQ, left_int, right_int, "ptrcmp_eq")).into()
        } else {
            emit!(self.build_int_compare(
                IntPredicate::EQ,
                left_val.into_int_value(),
                right_val.into_int_value(),
                "icmp_eq"
            ))
            .into()
        }
    }

    pub(crate) fn codegen_int_ne(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let left_val = self.codegen_expr(left);
        let right_val = self.codegen_expr(right);
        if left.ty.is_ref_type() {
            let left_int = emit!(self.build_ptr_to_int(
                left_val.into_pointer_value(),
                self.i64_type(),
                "ptr_to_int_l"
            ));
            let right_int = emit!(self.build_ptr_to_int(
                right_val.into_pointer_value(),
                self.i64_type(),
                "ptr_to_int_r"
            ));
            emit!(self.build_int_compare(IntPredicate::NE, left_int, right_int, "ptrcmp_ne")).into()
        } else {
            emit!(self.build_int_compare(
                IntPredicate::NE,
                left_val.into_int_value(),
                right_val.into_int_value(),
                "icmp_ne"
            ))
            .into()
        }
    }

    binop_fn!(int_cmp codegen_int_lt, IntPredicate::SLT, "icmp_lt");
    binop_fn!(int_cmp codegen_int_le, IntPredicate::SLE, "icmp_le");
    binop_fn!(int_cmp codegen_int_gt, IntPredicate::SGT, "icmp_gt");
    binop_fn!(int_cmp codegen_int_ge, IntPredicate::SGE, "icmp_ge");

    // Float comparisons
    binop_fn!(float_cmp codegen_float_eq, inkwell::FloatPredicate::OEQ, "fcmp_eq");
    binop_fn!(float_cmp codegen_float_ne, inkwell::FloatPredicate::ONE, "fcmp_ne");
    binop_fn!(float_cmp codegen_float_lt, inkwell::FloatPredicate::OLT, "fcmp_lt");
    binop_fn!(float_cmp codegen_float_le, inkwell::FloatPredicate::OLE, "fcmp_le");
    binop_fn!(float_cmp codegen_float_gt, inkwell::FloatPredicate::OGT, "fcmp_gt");
    binop_fn!(float_cmp codegen_float_ge, inkwell::FloatPredicate::OGE, "fcmp_ge");

    // Bool comparisons
    binop_fn!(bool_cmp codegen_bool_eq, IntPredicate::EQ, "bcmp_eq");
    binop_fn!(bool_cmp codegen_bool_ne, IntPredicate::NE, "bcmp_ne");

    // Logical operations
    pub(crate) fn codegen_logical_and(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
        result_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        self.codegen_logical(&LogicalOp::And, left, right, result_ty)
    }

    pub(crate) fn codegen_logical_or(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
        result_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        self.codegen_logical(&LogicalOp::Or, left, right, result_ty)
    }
}
