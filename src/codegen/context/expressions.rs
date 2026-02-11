use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;

use crate::tir::{
    BitwiseBinOp, CastKind, FloatArithOp, IntArithOp, LogicalOp, TirExpr, TirExprKind, TypedBinOp,
    UnaryOpKind, ValueType,
};

use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(super) fn codegen_expr(&mut self, expr: &TirExpr) -> BasicValueEnum<'ctx> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => self.i64_type().const_int(*val as u64, false).into(),

            TirExprKind::FloatLiteral(val) => self.f64_type().const_float(*val).into(),

            TirExprKind::StrLiteral(s) => {
                let global = self.builder.build_global_string_ptr(s, "str_data").unwrap();
                let data_ptr = global.as_pointer_value();
                let len = self.i64_type().const_int(s.len() as u64, false);
                let str_new_fn = self.get_or_declare_str_new();
                let call = self
                    .builder
                    .build_call(str_new_fn, &[data_ptr.into(), len.into()], "str_new")
                    .unwrap();
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
                let bytes_new_fn = self.get_or_declare_bytes_new();
                let call = self
                    .builder
                    .build_call(bytes_new_fn, &[data_ptr.into(), len.into()], "bytes_new")
                    .unwrap();
                self.extract_call_value(call)
            }

            TirExprKind::Var(name) => {
                let ptr = self.variables[name.as_str()];
                self.builder
                    .build_load(self.get_llvm_type(&expr.ty), ptr, name)
                    .unwrap()
            }

            TirExprKind::BinOp { op, left, right } => {
                let left_val = self.codegen_expr(left);
                let right_val = self.codegen_expr(right);

                match op {
                    TypedBinOp::FloatArith(float_op) => {
                        let l = left_val.into_float_value();
                        let r = right_val.into_float_value();

                        let result = match float_op {
                            FloatArithOp::Add => {
                                self.builder.build_float_add(l, r, "fadd").unwrap()
                            }
                            FloatArithOp::Sub => {
                                self.builder.build_float_sub(l, r, "fsub").unwrap()
                            }
                            FloatArithOp::Mul => {
                                self.builder.build_float_mul(l, r, "fmul").unwrap()
                            }
                            FloatArithOp::Div => {
                                self.builder.build_float_div(l, r, "fdiv").unwrap()
                            }
                            FloatArithOp::Mod => {
                                self.builder.build_float_rem(l, r, "fmod").unwrap()
                            }
                            FloatArithOp::FloorDiv => {
                                let div = self.builder.build_float_div(l, r, "fdiv").unwrap();
                                let floor_fn = self
                                    .module
                                    .get_function("llvm.floor.f64")
                                    .unwrap_or_else(|| {
                                        let f64_type = self.context.f64_type();
                                        let fn_type = f64_type.fn_type(&[f64_type.into()], false);
                                        self.module.add_function("llvm.floor.f64", fn_type, None)
                                    });
                                let call = self
                                    .builder
                                    .build_call(floor_fn, &[div.into()], "floordiv")
                                    .unwrap();
                                self.extract_call_value(call).into_float_value()
                            }
                            FloatArithOp::Pow => {
                                let pow_fn =
                                    self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                                        let f64_type = self.context.f64_type();
                                        let fn_type = f64_type
                                            .fn_type(&[f64_type.into(), f64_type.into()], false);
                                        self.module.add_function("llvm.pow.f64", fn_type, None)
                                    });
                                let call = self
                                    .builder
                                    .build_call(pow_fn, &[l.into(), r.into()], "pow")
                                    .unwrap();
                                self.extract_call_value(call).into_float_value()
                            }
                        };
                        result.into()
                    }

                    TypedBinOp::IntArith(int_op) => {
                        let l = left_val.into_int_value();
                        let r = right_val.into_int_value();

                        let result = match int_op {
                            IntArithOp::Add => self.builder.build_int_add(l, r, "add").unwrap(),
                            IntArithOp::Sub => self.builder.build_int_sub(l, r, "sub").unwrap(),
                            IntArithOp::Mul => self.builder.build_int_mul(l, r, "mul").unwrap(),
                            // No Div variant — Python `/` always returns float.
                            IntArithOp::Mod => {
                                self.builder.build_int_signed_rem(l, r, "mod").unwrap()
                            }
                            IntArithOp::FloorDiv => {
                                let div =
                                    self.builder.build_int_signed_div(l, r, "div_tmp").unwrap();
                                let rem =
                                    self.builder.build_int_signed_rem(l, r, "rem_tmp").unwrap();
                                let zero = self.i64_type().const_int(0, false);
                                let rem_nonzero = self
                                    .builder
                                    .build_int_compare(IntPredicate::NE, rem, zero, "rem_nz")
                                    .unwrap();
                                let xor_val = self.builder.build_xor(l, r, "xor_signs").unwrap();
                                let signs_differ = self
                                    .builder
                                    .build_int_compare(
                                        IntPredicate::SLT,
                                        xor_val,
                                        zero,
                                        "signs_diff",
                                    )
                                    .unwrap();
                                let need_adjust = self
                                    .builder
                                    .build_and(rem_nonzero, signs_differ, "need_adj")
                                    .unwrap();
                                let adjust = self
                                    .builder
                                    .build_int_z_extend(need_adjust, self.i64_type(), "adj_ext")
                                    .unwrap();
                                self.builder.build_int_sub(div, adjust, "floordiv").unwrap()
                            }
                            IntArithOp::Pow => {
                                let pow_fn = self.get_or_declare_function(
                                    "__tython_pow_int",
                                    &[ValueType::Int, ValueType::Int],
                                    Some(ValueType::Int),
                                );
                                let call = self
                                    .builder
                                    .build_call(pow_fn, &[l.into(), r.into()], "ipow")
                                    .unwrap();
                                self.extract_call_value(call).into_int_value()
                            }
                        };
                        result.into()
                    }

                    TypedBinOp::Bitwise(bitwise_op) => {
                        let left_int = left_val.into_int_value();
                        let right_int = right_val.into_int_value();

                        let result = match bitwise_op {
                            BitwiseBinOp::BitAnd => self
                                .builder
                                .build_and(left_int, right_int, "bitand")
                                .unwrap(),
                            BitwiseBinOp::BitOr => {
                                self.builder.build_or(left_int, right_int, "bitor").unwrap()
                            }
                            BitwiseBinOp::BitXor => self
                                .builder
                                .build_xor(left_int, right_int, "bitxor")
                                .unwrap(),
                            BitwiseBinOp::LShift => self
                                .builder
                                .build_left_shift(left_int, right_int, "lshift")
                                .unwrap(),
                            BitwiseBinOp::RShift => self
                                .builder
                                .build_right_shift(left_int, right_int, true, "rshift")
                                .unwrap(),
                        };
                        result.into()
                    }
                }
            }

            TirExprKind::Call { func, args } => {
                let arg_types: Vec<ValueType> = args.iter().map(|a| a.ty.clone()).collect();
                let function =
                    self.get_or_declare_function(func, &arg_types, Some(expr.ty.clone()));
                let arg_metadata = self.codegen_call_args(args);
                let call_site = self
                    .builder
                    .build_call(function, &arg_metadata, "call")
                    .unwrap();
                self.extract_call_value(call_site)
            }

            TirExprKind::ExternalCall { func, args } => {
                let function = self.get_or_declare_function(
                    func.symbol(),
                    &func.param_types(),
                    func.return_type(),
                );
                let arg_metadata = self.codegen_call_args(args);
                let call_site = self
                    .builder
                    .build_call(function, &arg_metadata, "ext_call")
                    .unwrap();
                self.extract_call_value(call_site)
            }

            TirExprKind::Cast { kind, arg } => {
                let arg_val = self.codegen_expr(arg);
                match kind {
                    CastKind::FloatToInt => self
                        .builder
                        .build_float_to_signed_int(
                            arg_val.into_float_value(),
                            self.i64_type(),
                            "ftoi",
                        )
                        .unwrap()
                        .into(),

                    CastKind::IntToFloat => self
                        .builder
                        .build_signed_int_to_float(
                            arg_val.into_int_value(),
                            self.f64_type(),
                            "itof",
                        )
                        .unwrap()
                        .into(),

                    CastKind::BoolToFloat => self
                        .builder
                        .build_signed_int_to_float(
                            arg_val.into_int_value(),
                            self.f64_type(),
                            "btof",
                        )
                        .unwrap()
                        .into(),

                    CastKind::IntToBool => {
                        let cmp = self.build_int_truthiness_check(arg_val.into_int_value(), "itob");
                        self.builder
                            .build_int_z_extend(cmp, self.i64_type(), "zext_bool")
                            .unwrap()
                            .into()
                    }

                    CastKind::FloatToBool => {
                        let cmp =
                            self.build_float_truthiness_check(arg_val.into_float_value(), "ftob");
                        self.builder
                            .build_int_z_extend(cmp, self.i64_type(), "zext_bool")
                            .unwrap()
                            .into()
                    }

                    CastKind::BoolToInt => arg_val, // same representation
                }
            }

            TirExprKind::Compare { op, left, right } => {
                let left_val = self.codegen_expr(left);
                let right_val = self.codegen_expr(right);

                let cmp_result = if left.ty == ValueType::Float {
                    self.builder
                        .build_float_compare(
                            Self::float_predicate(op),
                            left_val.into_float_value(),
                            right_val.into_float_value(),
                            "fcmp",
                        )
                        .unwrap()
                } else {
                    self.builder
                        .build_int_compare(
                            Self::int_predicate(op),
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "cmp",
                        )
                        .unwrap()
                };

                self.builder
                    .build_int_z_extend(cmp_result, self.i64_type(), "zext_bool")
                    .unwrap()
                    .into()
            }

            TirExprKind::UnaryOp { op, operand } => {
                let operand_val = self.codegen_expr(operand);
                match op {
                    UnaryOpKind::Neg => {
                        if operand.ty == ValueType::Float {
                            let zero = self.f64_type().const_float(0.0);
                            self.builder
                                .build_float_sub(zero, operand_val.into_float_value(), "fneg")
                                .unwrap()
                                .into()
                        } else {
                            let zero = self.i64_type().const_int(0, false);
                            self.builder
                                .build_int_sub(zero, operand_val.into_int_value(), "neg")
                                .unwrap()
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
                        let inverted = self.builder.build_not(truth, "not").unwrap();
                        self.builder
                            .build_int_z_extend(inverted, self.i64_type(), "not_zext")
                            .unwrap()
                            .into()
                    }
                    UnaryOpKind::BitNot => {
                        let val = operand_val.into_int_value();
                        let all_ones = self.i64_type().const_all_ones();
                        self.builder
                            .build_xor(val, all_ones, "bitnot")
                            .unwrap()
                            .into()
                    }
                }
            }

            TirExprKind::LogicalOp { op, left, right } => {
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                // Evaluate left side
                let left_val = self.codegen_expr(left);
                let left_truth =
                    self.build_truthiness_check_for_value(left_val, &left.ty, "log_left");
                let left_bb = self.builder.get_insert_block().unwrap();

                let right_bb = self.context.append_basic_block(function, "log_right");
                let merge_bb = self.context.append_basic_block(function, "log_merge");

                match op {
                    LogicalOp::And => {
                        // If left is falsy, short-circuit; else evaluate right
                        self.builder
                            .build_conditional_branch(left_truth, right_bb, merge_bb)
                            .unwrap();
                    }
                    LogicalOp::Or => {
                        // If left is truthy, short-circuit; else evaluate right
                        self.builder
                            .build_conditional_branch(left_truth, merge_bb, right_bb)
                            .unwrap();
                    }
                }

                // Evaluate right in right_bb
                self.builder.position_at_end(right_bb);
                let right_val = self.codegen_expr(right);
                let right_end_bb = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                // Merge: phi node selects left_val or right_val
                self.builder.position_at_end(merge_bb);
                let llvm_type = self.get_llvm_type(&expr.ty);
                let phi = self.builder.build_phi(llvm_type, "log_result").unwrap();
                phi.add_incoming(&[(&left_val, left_bb), (&right_val, right_end_bb)]);

                phi.as_basic_value()
            }

            // ── class expressions ────────────────────────────────────
            TirExprKind::Construct {
                class_name,
                init_mangled_name,
                args,
            } => {
                let struct_type = self.class_types[class_name.as_str()];

                // Allocate heap memory for the struct
                let size = struct_type.size_of().unwrap();
                let size_i64 = self
                    .builder
                    .build_int_cast(size, self.i64_type(), "size_i64")
                    .unwrap();
                let malloc_fn = self.get_or_declare_malloc();
                let call_site = self
                    .builder
                    .build_call(malloc_fn, &[size_i64.into()], "malloc")
                    .unwrap();
                let ptr = self.extract_call_value(call_site).into_pointer_value();

                // Build full arg list: [self_ptr, ...args]
                let mut init_args: Vec<inkwell::values::BasicMetadataValueEnum> = vec![ptr.into()];
                init_args.extend(self.codegen_call_args(args));

                // Declare/get __init__ function
                let mut param_types = vec![ValueType::Class(class_name.clone())];
                param_types.extend(args.iter().map(|a| a.ty.clone()));
                let init_fn = self.get_or_declare_function(init_mangled_name, &param_types, None);

                self.builder
                    .build_call(init_fn, &init_args, "init")
                    .unwrap();

                ptr.into()
            }

            TirExprKind::GetField {
                object,
                class_name,
                field_index,
            } => {
                let obj_ptr = self.codegen_expr(object).into_pointer_value();
                let struct_type = self.class_types[class_name.as_str()];

                let field_ptr = self
                    .builder
                    .build_struct_gep(struct_type, obj_ptr, *field_index as u32, "field_ptr")
                    .unwrap();

                let field_llvm_type = self.get_llvm_type(&expr.ty);
                self.builder
                    .build_load(field_llvm_type, field_ptr, "field_val")
                    .unwrap()
            }

            TirExprKind::MethodCall {
                object,
                method_mangled_name,
                args,
            } => {
                let self_val = self.codegen_expr(object);

                // Build full arg list: [self, ...args]
                let mut all_meta: Vec<inkwell::values::BasicMetadataValueEnum> =
                    vec![self_val.into()];
                all_meta.extend(self.codegen_call_args(args));

                // Declare/get method function
                let mut param_types = vec![object.ty.clone()];
                param_types.extend(args.iter().map(|a| a.ty.clone()));
                let method_fn = self.get_or_declare_function(
                    method_mangled_name,
                    &param_types,
                    Some(expr.ty.clone()),
                );

                let call_site = self
                    .builder
                    .build_call(method_fn, &all_meta, "method_call")
                    .unwrap();

                self.extract_call_value(call_site)
            }

            TirExprKind::ListLiteral {
                element_type,
                elements,
            } => {
                if elements.is_empty() {
                    let empty_fn = self.get_or_declare_function(
                        "__tython_list_empty",
                        &[],
                        Some(ValueType::List(Box::new(element_type.clone()))),
                    );
                    let call = self
                        .builder
                        .build_call(empty_fn, &[], "list_empty")
                        .unwrap();
                    self.extract_call_value(call)
                } else {
                    let len = elements.len();
                    let i64_ty = self.i64_type();
                    let array_ty = i64_ty.array_type(len as u32);
                    let array_alloca = self.builder.build_alloca(array_ty, "list_data").unwrap();

                    for (i, elem) in elements.iter().enumerate() {
                        let val = self.codegen_expr(elem);
                        let i64_val = self.bitcast_to_i64(val, element_type);
                        let zero = self.context.i32_type().const_int(0, false);
                        let idx = self.context.i32_type().const_int(i as u64, false);
                        let elem_ptr = unsafe {
                            self.builder
                                .build_in_bounds_gep(
                                    array_ty,
                                    array_alloca,
                                    &[zero, idx],
                                    "elem_ptr",
                                )
                                .unwrap()
                        };
                        self.builder.build_store(elem_ptr, i64_val).unwrap();
                    }

                    let len_val = i64_ty.const_int(len as u64, false);
                    let list_new_fn = self.get_or_declare_list_new();
                    let call = self
                        .builder
                        .build_call(
                            list_new_fn,
                            &[array_alloca.into(), len_val.into()],
                            "list_new",
                        )
                        .unwrap();
                    self.extract_call_value(call)
                }
            }

            TirExprKind::ListGet { list, index } => {
                let list_val = self.codegen_expr(list);
                let index_val = self.codegen_expr(index);
                let list_get_fn = self.get_or_declare_list_get();
                let call = self
                    .builder
                    .build_call(
                        list_get_fn,
                        &[list_val.into(), index_val.into()],
                        "list_get",
                    )
                    .unwrap();
                let i64_val = self.extract_call_value(call).into_int_value();
                self.bitcast_from_i64(i64_val, &expr.ty)
            }
        }
    }
}
