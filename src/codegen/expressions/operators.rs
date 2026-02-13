use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;

use crate::tir::builtin::BuiltinFn;
use crate::tir::{
    CastKind, FloatArithOp, IntArithOp, LogicalOp, OrderedCmpOp, TirExpr, TypedBinOp, UnaryOpKind,
    ValueType,
};

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_bin_op(
        &mut self,
        op: &TypedBinOp,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let left_val = self.codegen_expr(left);
        let right_val = self.codegen_expr(right);

        match op {
            TypedBinOp::FloatArith(float_op) => {
                let l = left_val.into_float_value();
                let r = right_val.into_float_value();

                let result = match float_op {
                    FloatArithOp::Add => emit!(self.build_float_add(l, r, "fadd")),
                    FloatArithOp::Sub => emit!(self.build_float_sub(l, r, "fsub")),
                    FloatArithOp::Mul => emit!(self.build_float_mul(l, r, "fmul")),
                    FloatArithOp::Div => emit!(self.build_float_div(l, r, "fdiv")),
                    FloatArithOp::Mod => emit!(self.build_float_rem(l, r, "fmod")),
                    FloatArithOp::FloorDiv => {
                        let div = emit!(self.build_float_div(l, r, "fdiv"));
                        let f64_ty = self.f64_type();
                        let floor_fn = self.get_llvm_intrinsic(
                            "llvm.floor.f64",
                            f64_ty.fn_type(&[f64_ty.into()], false),
                        );
                        let call = emit!(self.build_call(floor_fn, &[div.into()], "floordiv"));
                        self.extract_call_value(call).into_float_value()
                    }
                    FloatArithOp::Pow => {
                        let f64_ty = self.f64_type();
                        let pow_fn = self.get_llvm_intrinsic(
                            "llvm.pow.f64",
                            f64_ty.fn_type(&[f64_ty.into(), f64_ty.into()], false),
                        );
                        let call = emit!(self.build_call(pow_fn, &[l.into(), r.into()], "pow"));
                        self.extract_call_value(call).into_float_value()
                    }
                };
                result.into()
            }

            TypedBinOp::IntArith(int_op) => {
                let l = left_val.into_int_value();
                let r = right_val.into_int_value();

                let result = match int_op {
                    IntArithOp::Add => emit!(self.build_int_add(l, r, "add")),
                    IntArithOp::Sub => emit!(self.build_int_sub(l, r, "sub")),
                    IntArithOp::Mul => emit!(self.build_int_mul(l, r, "mul")),
                    IntArithOp::Mod => emit!(self.build_int_signed_rem(l, r, "mod")),
                    IntArithOp::FloorDiv => {
                        let div = emit!(self.build_int_signed_div(l, r, "div_tmp"));
                        let rem = emit!(self.build_int_signed_rem(l, r, "rem_tmp"));
                        let zero = self.i64_type().const_int(0, false);
                        let rem_nonzero =
                            emit!(self.build_int_compare(IntPredicate::NE, rem, zero, "rem_nz"));
                        let xor_val = emit!(self.build_xor(l, r, "xor_signs"));
                        let signs_differ = emit!(self.build_int_compare(
                            IntPredicate::SLT,
                            xor_val,
                            zero,
                            "signs_diff",
                        ));
                        let need_adjust =
                            emit!(self.build_and(rem_nonzero, signs_differ, "need_adj"));
                        let adjust =
                            emit!(self.build_int_z_extend(need_adjust, self.i64_type(), "adj_ext"));
                        emit!(self.build_int_sub(div, adjust, "floordiv"))
                    }
                    IntArithOp::Pow => {
                        let pow_fn = self.get_builtin(BuiltinFn::PowInt);
                        let call = emit!(self.build_call(pow_fn, &[l.into(), r.into()], "ipow"));
                        self.extract_call_value(call).into_int_value()
                    }
                };
                result.into()
            }

            TypedBinOp::Bitwise(bitwise_op) => {
                let l = left_val.into_int_value();
                let r = right_val.into_int_value();
                dispatch!(self, bitwise_op,
                    crate::tir::BitwiseBinOp::BitAnd  => build_and(l, r, "bitand"),
                    crate::tir::BitwiseBinOp::BitOr   => build_or(l, r, "bitor"),
                    crate::tir::BitwiseBinOp::BitXor  => build_xor(l, r, "bitxor"),
                    crate::tir::BitwiseBinOp::LShift  => build_left_shift(l, r, "lshift"),
                    crate::tir::BitwiseBinOp::RShift  => build_right_shift(l, r, true, "rshift"),
                )
                .into()
            }
        }
    }

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
                arg_val.into_int_value(),
                self.f64_type(),
                "btof",
            ))
            .into(),

            CastKind::IntToBool => {
                let cmp = self.build_int_truthiness_check(arg_val.into_int_value(), "itob");
                emit!(self.build_int_z_extend(cmp, self.i64_type(), "zext_bool")).into()
            }

            CastKind::FloatToBool => {
                let cmp = self.build_float_truthiness_check(arg_val.into_float_value(), "ftob");
                emit!(self.build_int_z_extend(cmp, self.i64_type(), "zext_bool")).into()
            }

            CastKind::BoolToInt => arg_val, // same representation
        }
    }

    pub(crate) fn codegen_compare(
        &mut self,
        op: &OrderedCmpOp,
        left: &TirExpr,
        right: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let left_val = self.codegen_expr(left);
        let right_val = self.codegen_expr(right);

        let cmp_result = if left.ty == ValueType::Float {
            emit!(self.build_float_compare(
                Self::float_predicate(op),
                left_val.into_float_value(),
                right_val.into_float_value(),
                "fcmp",
            ))
        } else if left.ty.is_ref_type() {
            // Pointer comparison for reference types (is/is not)
            let left_int = emit!(self.build_ptr_to_int(
                left_val.into_pointer_value(),
                self.i64_type(),
                "ptr_to_int_l",
            ));
            let right_int = emit!(self.build_ptr_to_int(
                right_val.into_pointer_value(),
                self.i64_type(),
                "ptr_to_int_r",
            ));
            emit!(self.build_int_compare(Self::int_predicate(op), left_int, right_int, "ptrcmp"))
        } else {
            emit!(self.build_int_compare(
                Self::int_predicate(op),
                left_val.into_int_value(),
                right_val.into_int_value(),
                "cmp",
            ))
        };

        emit!(self.build_int_z_extend(cmp_result, self.i64_type(), "zext_bool")).into()
    }

    pub(crate) fn codegen_unary(
        &mut self,
        op: &UnaryOpKind,
        operand: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let operand_val = self.codegen_expr(operand);
        match op {
            UnaryOpKind::Neg => {
                if operand.ty == ValueType::Float {
                    let zero = self.f64_type().const_float(0.0);
                    emit!(self.build_float_sub(zero, operand_val.into_float_value(), "fneg")).into()
                } else {
                    let zero = self.i64_type().const_int(0, false);
                    emit!(self.build_int_sub(zero, operand_val.into_int_value(), "neg")).into()
                }
            }
            UnaryOpKind::Pos => operand_val,
            UnaryOpKind::Not => {
                let truth =
                    self.build_truthiness_check_for_value(operand_val, &operand.ty, "not_truth");
                let inverted = emit!(self.build_not(truth, "not"));
                emit!(self.build_int_z_extend(inverted, self.i64_type(), "not_zext")).into()
            }
            UnaryOpKind::BitNot => {
                let val = operand_val.into_int_value();
                let all_ones = self.i64_type().const_all_ones();
                emit!(self.build_xor(val, all_ones, "bitnot")).into()
            }
        }
    }

    pub(crate) fn codegen_logical(
        &mut self,
        op: &LogicalOp,
        left: &TirExpr,
        right: &TirExpr,
        result_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        // Evaluate left side
        let left_val = self.codegen_expr(left);
        let left_truth = self.build_truthiness_check_for_value(left_val, &left.ty, "log_left");
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
}
