use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;

use crate::tir::builtin::BuiltinFn;
use crate::tir::{CastKind, LogicalOp, TirExpr, ValueType};

use super::super::Codegen;

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
        result_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        // Evaluate left side
        let left_val = self.codegen_expr(left);
        let left_truth = if left.ty == ValueType::Bool {
            left_val.into_int_value()
        } else if left.ty == ValueType::Float {
            emit!(self.build_float_compare(
                inkwell::FloatPredicate::ONE,
                left_val.into_float_value(),
                self.f64_type().const_float(0.0),
                "log_left",
            ))
        } else {
            emit!(self.build_int_compare(
                IntPredicate::NE,
                left_val.into_int_value(),
                self.i64_type().const_int(0, false),
                "log_left",
            ))
        };
        let left_bb = emit!(self.get_insert_block());

        let right_bb = self.context.append_basic_block(function, "log_right");
        let merge_bb = self.context.append_basic_block(function, "log_merge");

        match op {
            LogicalOp::And => {
                // If left is falsy, short-circuit; else evaluate right
                emit!(self.build_conditional_branch(left_truth, right_bb, merge_bb));
            }
            LogicalOp::Or => {
                // If left is truthy, short-circuit; else evaluate right
                emit!(self.build_conditional_branch(left_truth, merge_bb, right_bb));
            }
        }

        // Evaluate right in right_bb
        self.builder.position_at_end(right_bb);
        let right_val = self.codegen_expr(right);
        let right_end_bb = emit!(self.get_insert_block());
        emit!(self.build_unconditional_branch(merge_bb));

        // Merge: phi node selects left_val or right_val
        self.builder.position_at_end(merge_bb);
        let llvm_type = self.get_llvm_type(result_ty);
        let phi = emit!(self.build_phi(llvm_type, "log_result"));
        phi.add_incoming(&[(&left_val, left_bb), (&right_val, right_end_bb)]);

        phi.as_basic_value()
    }

    // Individual typed operation functions

    // Integer arithmetic
    pub(crate) fn codegen_int_add(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_add(l, r, "add")).into()
    }

    pub(crate) fn codegen_int_sub(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_sub(l, r, "sub")).into()
    }

    pub(crate) fn codegen_int_mul(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_mul(l, r, "mul")).into()
    }

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

    pub(crate) fn codegen_int_mod(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_signed_rem(l, r, "mod")).into()
    }

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
    pub(crate) fn codegen_float_add(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_add(l, r, "fadd")).into()
    }

    pub(crate) fn codegen_float_sub(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_sub(l, r, "fsub")).into()
    }

    pub(crate) fn codegen_float_mul(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_mul(l, r, "fmul")).into()
    }

    pub(crate) fn codegen_float_div(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_div(l, r, "fdiv")).into()
    }

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

    pub(crate) fn codegen_float_mod(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_rem(l, r, "fmod")).into()
    }

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
    pub(crate) fn codegen_bitwise_and(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_and(l, r, "bitand")).into()
    }

    pub(crate) fn codegen_bitwise_or(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_or(l, r, "bitor")).into()
    }

    pub(crate) fn codegen_bitwise_xor(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_xor(l, r, "bitxor")).into()
    }

    pub(crate) fn codegen_left_shift(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_left_shift(l, r, "lshift")).into()
    }

    pub(crate) fn codegen_right_shift(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_right_shift(l, r, true, "rshift")).into()
    }

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

    pub(crate) fn codegen_int_lt(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_compare(IntPredicate::SLT, l, r, "icmp_lt")).into()
    }

    pub(crate) fn codegen_int_le(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_compare(IntPredicate::SLE, l, r, "icmp_le")).into()
    }

    pub(crate) fn codegen_int_gt(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_compare(IntPredicate::SGT, l, r, "icmp_gt")).into()
    }

    pub(crate) fn codegen_int_ge(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        emit!(self.build_int_compare(IntPredicate::SGE, l, r, "icmp_ge")).into()
    }

    // Float comparisons
    pub(crate) fn codegen_float_eq(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "fcmp_eq")).into()
    }

    pub(crate) fn codegen_float_ne(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_compare(inkwell::FloatPredicate::ONE, l, r, "fcmp_ne")).into()
    }

    pub(crate) fn codegen_float_lt(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_compare(inkwell::FloatPredicate::OLT, l, r, "fcmp_lt")).into()
    }

    pub(crate) fn codegen_float_le(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_compare(inkwell::FloatPredicate::OLE, l, r, "fcmp_le")).into()
    }

    pub(crate) fn codegen_float_gt(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_compare(inkwell::FloatPredicate::OGT, l, r, "fcmp_gt")).into()
    }

    pub(crate) fn codegen_float_ge(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_float_value();
        let r = self.codegen_expr(right).into_float_value();
        emit!(self.build_float_compare(inkwell::FloatPredicate::OGE, l, r, "fcmp_ge")).into()
    }

    // Bool comparisons
    pub(crate) fn codegen_bool_eq(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        let left_i64 = emit!(self.build_int_z_extend(l, self.i64_type(), "b_l_i64"));
        let right_i64 = emit!(self.build_int_z_extend(r, self.i64_type(), "b_r_i64"));
        emit!(self.build_int_compare(IntPredicate::EQ, left_i64, right_i64, "bcmp_eq")).into()
    }

    pub(crate) fn codegen_bool_ne(
        &mut self,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let l = self.codegen_expr(left).into_int_value();
        let r = self.codegen_expr(right).into_int_value();
        let left_i64 = emit!(self.build_int_z_extend(l, self.i64_type(), "b_l_i64"));
        let right_i64 = emit!(self.build_int_z_extend(r, self.i64_type(), "b_r_i64"));
        emit!(self.build_int_compare(IntPredicate::NE, left_i64, right_i64, "bcmp_ne")).into()
    }

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
