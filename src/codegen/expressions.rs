use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;
use inkwell::IntPredicate;

use crate::tir::builtin::BuiltinFn;
use crate::tir::{
    BitwiseBinOp, CastKind, FloatArithOp, IntArithOp, LogicalOp, TirExpr, TirExprKind, TypedBinOp,
    UnaryOpKind, ValueType,
};

use super::runtime_fn::RuntimeFn;
use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(super) fn codegen_expr(&mut self, expr: &TirExpr) -> BasicValueEnum<'ctx> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => self.i64_type().const_int(*val as u64, false).into(),

            TirExprKind::FloatLiteral(val) => self.f64_type().const_float(*val).into(),

            TirExprKind::StrLiteral(s) => {
                let global = emit!(self.build_global_string_ptr(s, "str_data"));
                let data_ptr = global.as_pointer_value();
                let len = self.i64_type().const_int(s.len() as u64, false);
                let str_new_fn = self.get_runtime_fn(RuntimeFn::StrNew);
                let call =
                    emit!(self.build_call(str_new_fn, &[data_ptr.into(), len.into()], "str_new"));
                self.extract_call_value(call)
            }

            TirExprKind::BytesLiteral(bytes) => {
                let byte_values: Vec<_> = bytes
                    .iter()
                    .map(|b| self.context.i8_type().const_int(*b as u64, false))
                    .collect();
                let array_val = self.context.i8_type().const_array(&byte_values);
                let array_type = self.context.i8_type().array_type(bytes.len() as u32);
                let global = self.module.add_global(array_type, None, "bytes_data");
                global.set_initializer(&array_val);
                global.set_constant(true);
                let data_ptr = global.as_pointer_value();
                let len = self.i64_type().const_int(bytes.len() as u64, false);
                let bytes_new_fn = self.get_runtime_fn(RuntimeFn::BytesNew);
                let call = emit!(self.build_call(
                    bytes_new_fn,
                    &[data_ptr.into(), len.into()],
                    "bytes_new"
                ));
                self.extract_call_value(call)
            }

            TirExprKind::Var(name) => {
                emit!(self.build_load(
                    self.get_llvm_type(&expr.ty),
                    self.variables[name.as_str()],
                    name
                ))
            }

            TirExprKind::BinOp { op, left, right } => {
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
                                let call =
                                    emit!(self.build_call(floor_fn, &[div.into()], "floordiv"));
                                self.extract_call_value(call).into_float_value()
                            }
                            FloatArithOp::Pow => {
                                let f64_ty = self.f64_type();
                                let pow_fn = self.get_llvm_intrinsic(
                                    "llvm.pow.f64",
                                    f64_ty.fn_type(&[f64_ty.into(), f64_ty.into()], false),
                                );
                                let call =
                                    emit!(self.build_call(pow_fn, &[l.into(), r.into()], "pow"));
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
                                let rem_nonzero = emit!(self.build_int_compare(
                                    IntPredicate::NE,
                                    rem,
                                    zero,
                                    "rem_nz"
                                ));
                                let xor_val = emit!(self.build_xor(l, r, "xor_signs"));
                                let signs_differ = emit!(self.build_int_compare(
                                    IntPredicate::SLT,
                                    xor_val,
                                    zero,
                                    "signs_diff",
                                ));
                                let need_adjust =
                                    emit!(self.build_and(rem_nonzero, signs_differ, "need_adj"));
                                let adjust = emit!(self.build_int_z_extend(
                                    need_adjust,
                                    self.i64_type(),
                                    "adj_ext"
                                ));
                                emit!(self.build_int_sub(div, adjust, "floordiv"))
                            }
                            IntArithOp::Pow => {
                                let pow_fn = self.get_builtin(BuiltinFn::PowInt);
                                let call =
                                    emit!(self.build_call(pow_fn, &[l.into(), r.into()], "ipow"));
                                self.extract_call_value(call).into_int_value()
                            }
                        };
                        result.into()
                    }

                    TypedBinOp::Bitwise(bitwise_op) => {
                        let l = left_val.into_int_value();
                        let r = right_val.into_int_value();
                        dispatch!(self, bitwise_op,
                            BitwiseBinOp::BitAnd  => build_and(l, r, "bitand"),
                            BitwiseBinOp::BitOr   => build_or(l, r, "bitor"),
                            BitwiseBinOp::BitXor  => build_xor(l, r, "bitxor"),
                            BitwiseBinOp::LShift  => build_left_shift(l, r, "lshift"),
                            BitwiseBinOp::RShift  => build_right_shift(l, r, true, "rshift"),
                        )
                        .into()
                    }
                }
            }

            TirExprKind::Call { func, args } => {
                self.codegen_named_call(func, args, Some(&expr.ty)).unwrap()
            }

            TirExprKind::ExternalCall { func, args } => self
                .codegen_builtin_call(*func, args, Some(&expr.ty))
                .unwrap(),

            TirExprKind::Cast { kind, arg } => {
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
                        let cmp =
                            self.build_float_truthiness_check(arg_val.into_float_value(), "ftob");
                        emit!(self.build_int_z_extend(cmp, self.i64_type(), "zext_bool")).into()
                    }

                    CastKind::BoolToInt => arg_val, // same representation
                }
            }

            TirExprKind::Compare { op, left, right } => {
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
                    emit!(self.build_int_compare(
                        Self::int_predicate(op),
                        left_int,
                        right_int,
                        "ptrcmp"
                    ))
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

            TirExprKind::UnaryOp { op, operand } => {
                let operand_val = self.codegen_expr(operand);
                match op {
                    UnaryOpKind::Neg => {
                        if operand.ty == ValueType::Float {
                            let zero = self.f64_type().const_float(0.0);
                            emit!(self.build_float_sub(
                                zero,
                                operand_val.into_float_value(),
                                "fneg"
                            ))
                            .into()
                        } else {
                            let zero = self.i64_type().const_int(0, false);
                            emit!(self.build_int_sub(zero, operand_val.into_int_value(), "neg"))
                                .into()
                        }
                    }
                    UnaryOpKind::Pos => operand_val,
                    UnaryOpKind::Not => {
                        let truth = self.build_truthiness_check_for_value(
                            operand_val,
                            &operand.ty,
                            "not_truth",
                        );
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

            TirExprKind::LogicalOp { op, left, right } => {
                let function = emit!(self.get_insert_block()).get_parent().unwrap();

                // Evaluate left side
                let left_val = self.codegen_expr(left);
                let left_truth =
                    self.build_truthiness_check_for_value(left_val, &left.ty, "log_left");
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
                let llvm_type = self.get_llvm_type(&expr.ty);
                let phi = emit!(self.build_phi(llvm_type, "log_result"));
                phi.add_incoming(&[(&left_val, left_bb), (&right_val, right_end_bb)]);

                phi.as_basic_value()
            }

            // ── class expressions ────────────────────────────────────
            TirExprKind::Construct {
                class_name,
                init_mangled_name,
                args,
            } => {
                let struct_type = self.struct_types[class_name.as_str()];

                // Allocate heap memory for the struct
                let size = struct_type.size_of().unwrap();
                let size_i64 = emit!(self.build_int_cast(size, self.i64_type(), "size_i64"));
                let malloc_fn = self.get_runtime_fn(RuntimeFn::Malloc);
                let call_site = emit!(self.build_call(malloc_fn, &[size_i64.into()], "malloc"));
                let ptr = self.extract_call_value(call_site).into_pointer_value();

                // Build full arg list: [self_ptr, ...args]
                let mut init_args: Vec<inkwell::values::BasicValueEnum> = vec![ptr.into()];
                init_args.extend(self.codegen_call_args(args));

                // Declare/get __init__ function
                let mut param_types = vec![ValueType::Class(class_name.clone())];
                param_types.extend(args.iter().map(|a| a.ty.clone()));
                let init_fn = self.get_or_declare_function(init_mangled_name, &param_types, None);

                self.build_call_maybe_invoke(init_fn, &init_args, "init", true);

                ptr.into()
            }

            TirExprKind::GetField {
                object,
                class_name,
                field_index,
            } => {
                let obj_ptr = self.codegen_expr(object).into_pointer_value();
                let struct_type = self.struct_types[class_name.as_str()];

                let field_ptr = emit!(self.build_struct_gep(
                    struct_type,
                    obj_ptr,
                    *field_index as u32,
                    "field_ptr"
                ));

                let field_llvm_type = self.get_llvm_type(&expr.ty);
                emit!(self.build_load(field_llvm_type, field_ptr, "field_val"))
            }

            TirExprKind::TupleLiteral {
                elements,
                element_types,
            } => {
                let struct_type = self.get_or_create_tuple_struct(element_types);
                let size = struct_type.size_of().unwrap();
                let size_i64 = emit!(self.build_int_cast(size, self.i64_type(), "tuple_size_i64"));
                let malloc_fn = self.get_runtime_fn(RuntimeFn::Malloc);
                let call_site =
                    emit!(self.build_call(malloc_fn, &[size_i64.into()], "tuple_malloc"));
                let tuple_ptr = self.extract_call_value(call_site).into_pointer_value();

                for (i, elem) in elements.iter().enumerate() {
                    let field_ptr = emit!(self.build_struct_gep(
                        struct_type,
                        tuple_ptr,
                        i as u32,
                        "tuple_field_ptr"
                    ));
                    let elem_val = self.codegen_expr(elem);
                    emit!(self.build_store(field_ptr, elem_val));
                }
                tuple_ptr.into()
            }

            TirExprKind::TupleGet {
                tuple,
                index,
                element_types,
            } => {
                let tuple_ptr = self.codegen_expr(tuple).into_pointer_value();
                let struct_type = self.get_or_create_tuple_struct(element_types);
                let field_ptr = emit!(self.build_struct_gep(
                    struct_type,
                    tuple_ptr,
                    *index as u32,
                    "tuple_get_ptr"
                ));
                emit!(self.build_load(self.get_llvm_type(&expr.ty), field_ptr, "tuple_get"))
            }

            TirExprKind::TupleGetDynamic {
                tuple,
                index,
                len,
                element_types,
            } => {
                let tuple_ptr = self.codegen_expr(tuple).into_pointer_value();
                let idx_val = self.codegen_expr(index).into_int_value();
                let struct_type = self.get_or_create_tuple_struct(element_types);

                let result_alloca = self
                    .build_entry_block_alloca(self.get_llvm_type(&expr.ty), "tuple_dyn_get_tmp");

                let default_val: BasicValueEnum<'ctx> = match &expr.ty {
                    ValueType::Int | ValueType::Bool => self.i64_type().const_zero().into(),
                    ValueType::Float => self.f64_type().const_float(0.0).into(),
                    _ => self
                        .context
                        .ptr_type(AddressSpace::default())
                        .const_null()
                        .into(),
                };
                emit!(self.build_store(result_alloca, default_val));

                let len_i64 = self.i64_type().const_int(*len as u64, false);
                let is_neg = emit!(self.build_int_compare(
                    IntPredicate::SLT,
                    idx_val,
                    self.i64_type().const_zero(),
                    "tuple_idx_neg",
                ));
                let neg_adjusted =
                    emit!(self.build_int_add(idx_val, len_i64, "tuple_idx_norm_neg"));
                let norm_idx =
                    emit!(self.build_select(is_neg, neg_adjusted, idx_val, "tuple_idx_norm"))
                        .into_int_value();

                let function = emit!(self.get_insert_block()).get_parent().unwrap();
                let default_bb = self
                    .context
                    .append_basic_block(function, "tuple_idx_default");
                let merge_bb = self.context.append_basic_block(function, "tuple_idx_merge");

                let mut case_bbs = Vec::with_capacity(*len);
                for i in 0..*len {
                    case_bbs.push(
                        self.context
                            .append_basic_block(function, &format!("tuple_idx_case_{}", i)),
                    );
                }
                let switch_cases: Vec<_> = case_bbs
                    .iter()
                    .enumerate()
                    .map(|(i, bb)| (self.i64_type().const_int(i as u64, false), *bb))
                    .collect();
                emit!(self.build_switch(norm_idx, default_bb, &switch_cases));

                for (i, case_bb) in case_bbs.iter().enumerate() {
                    self.builder.position_at_end(*case_bb);
                    let field_ptr = emit!(self.build_struct_gep(
                        struct_type,
                        tuple_ptr,
                        i as u32,
                        "tuple_dyn_get_ptr"
                    ));
                    let field_val = emit!(self.build_load(
                        self.get_llvm_type(&expr.ty),
                        field_ptr,
                        "tuple_dyn_get"
                    ));
                    emit!(self.build_store(result_alloca, field_val));
                    emit!(self.build_unconditional_branch(merge_bb));
                }

                self.builder.position_at_end(default_bb);
                emit!(self.build_unconditional_branch(merge_bb));

                self.builder.position_at_end(merge_bb);
                emit!(self.build_load(
                    self.get_llvm_type(&expr.ty),
                    result_alloca,
                    "tuple_dyn_get_out",
                ))
            }

            TirExprKind::ListLiteral {
                element_type,
                elements,
            } => {
                if elements.is_empty() {
                    let empty_fn = self.get_builtin(BuiltinFn::ListEmpty);
                    let call = emit!(self.build_call(empty_fn, &[], "list_empty"));
                    self.extract_call_value(call)
                } else {
                    let len = elements.len();
                    let i64_ty = self.i64_type();
                    let array_ty = i64_ty.array_type(len as u32);
                    let array_alloca = self.build_entry_block_alloca(array_ty.into(), "list_data");

                    for (i, elem) in elements.iter().enumerate() {
                        let val = self.codegen_expr(elem);
                        let i64_val = self.bitcast_to_i64(val, element_type);
                        let zero = self.context.i32_type().const_int(0, false);
                        let idx = self.context.i32_type().const_int(i as u64, false);
                        let elem_ptr = unsafe {
                            emit!(self.build_in_bounds_gep(
                                array_ty,
                                array_alloca,
                                &[zero, idx],
                                "elem_ptr",
                            ))
                        };
                        emit!(self.build_store(elem_ptr, i64_val));
                    }

                    let len_val = i64_ty.const_int(len as u64, false);
                    let list_new_fn = self.get_runtime_fn(RuntimeFn::ListNew);
                    let call = emit!(self.build_call(
                        list_new_fn,
                        &[array_alloca.into(), len_val.into()],
                        "list_new",
                    ));
                    self.extract_call_value(call)
                }
            }
        }
    }
}
