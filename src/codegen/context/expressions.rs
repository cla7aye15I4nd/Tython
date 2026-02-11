use inkwell::types::BasicType;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;
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
                let global = emit!(self.build_global_string_ptr(s, "str_data"));
                let data_ptr = global.as_pointer_value();
                let len = self.i64_type().const_int(s.len() as u64, false);
                let str_new_fn = self.get_or_declare_str_new();
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
                let bytes_new_fn = self.get_or_declare_bytes_new();
                let call = emit!(self.build_call(
                    bytes_new_fn,
                    &[data_ptr.into(), len.into()],
                    "bytes_new"
                ));
                self.extract_call_value(call)
            }

            TirExprKind::Var(name) => {
                let ptr = self.variables[name.as_str()];
                emit!(self.build_load(self.get_llvm_type(&expr.ty), ptr, name))
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
                                emit!(self.build_float_add(l, r, "fadd"))
                            }
                            FloatArithOp::Sub => {
                                emit!(self.build_float_sub(l, r, "fsub"))
                            }
                            FloatArithOp::Mul => {
                                emit!(self.build_float_mul(l, r, "fmul"))
                            }
                            FloatArithOp::Div => {
                                emit!(self.build_float_div(l, r, "fdiv"))
                            }
                            FloatArithOp::Mod => {
                                emit!(self.build_float_rem(l, r, "fmod"))
                            }
                            FloatArithOp::FloorDiv => {
                                let div = emit!(self.build_float_div(l, r, "fdiv"));
                                let floor_fn = self
                                    .module
                                    .get_function("llvm.floor.f64")
                                    .unwrap_or_else(|| {
                                        let f64_type = self.context.f64_type();
                                        let fn_type = f64_type.fn_type(&[f64_type.into()], false);
                                        self.module.add_function("llvm.floor.f64", fn_type, None)
                                    });
                                let call =
                                    emit!(self.build_call(floor_fn, &[div.into()], "floordiv"));
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
                            // No Div variant — Python `/` always returns float.
                            IntArithOp::Mod => {
                                emit!(self.build_int_signed_rem(l, r, "mod"))
                            }
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
                                let pow_fn = self.get_or_declare_function(
                                    "__tython_pow_int",
                                    &[ValueType::Int, ValueType::Int],
                                    Some(ValueType::Int),
                                );
                                let call =
                                    emit!(self.build_call(pow_fn, &[l.into(), r.into()], "ipow"));
                                self.extract_call_value(call).into_int_value()
                            }
                        };
                        result.into()
                    }

                    TypedBinOp::Bitwise(bitwise_op) => {
                        let left_int = left_val.into_int_value();
                        let right_int = right_val.into_int_value();

                        let result = match bitwise_op {
                            BitwiseBinOp::BitAnd => {
                                emit!(self.build_and(left_int, right_int, "bitand"))
                            }
                            BitwiseBinOp::BitOr => {
                                emit!(self.build_or(left_int, right_int, "bitor"))
                            }
                            BitwiseBinOp::BitXor => {
                                emit!(self.build_xor(left_int, right_int, "bitxor"))
                            }
                            BitwiseBinOp::LShift => {
                                emit!(self.build_left_shift(left_int, right_int, "lshift"))
                            }
                            BitwiseBinOp::RShift => {
                                emit!(self.build_right_shift(left_int, right_int, true, "rshift"))
                            }
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
                let call_site = self.build_call_maybe_invoke(function, &arg_metadata, "call", true);
                self.extract_call_value(call_site)
            }

            TirExprKind::ExternalCall { func, args } => {
                use crate::tir::builtin::BuiltinFn;
                if matches!(func, BuiltinFn::ListPop | BuiltinFn::ListGet) {
                    // Return value is i64 slot — bitcast from i64 to actual type
                    let function = self.get_or_declare_function(
                        func.symbol(),
                        &func.param_types(),
                        func.return_type(),
                    );
                    let arg_metadata = self.codegen_call_args(args);
                    let call_site = emit!(self.build_call(
                        function,
                        &Self::to_meta_args(&arg_metadata),
                        "list_get_elem"
                    ));
                    let i64_val = self.extract_call_value(call_site).into_int_value();
                    return self.bitcast_from_i64(i64_val, &expr.ty);
                }

                if matches!(
                    func,
                    BuiltinFn::ListContains | BuiltinFn::ListIndex | BuiltinFn::ListCount
                ) {
                    // args = [list, value] — value needs bitcast to i64
                    let function = self.get_or_declare_function(
                        func.symbol(),
                        &func.param_types(),
                        func.return_type(),
                    );
                    let list_val = self.codegen_expr(&args[0]);
                    let elem_val = self.codegen_expr(&args[1]);
                    let i64_val = self.bitcast_to_i64(elem_val, &args[1].ty);
                    let call_site = emit!(self.build_call(
                        function,
                        &[list_val.into(), i64_val.into()],
                        "list_elem_op"
                    ));
                    return self.extract_call_value(call_site);
                }

                let function = self.get_or_declare_function(
                    func.symbol(),
                    &func.param_types(),
                    func.return_type(),
                );
                let arg_metadata = self.codegen_call_args(args);
                let call_site = emit!(self.build_call(
                    function,
                    &Self::to_meta_args(&arg_metadata),
                    "ext_call"
                ));
                self.extract_call_value(call_site)
            }

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
                let struct_type = self.class_types[class_name.as_str()];

                // Allocate heap memory for the struct
                let size = struct_type.size_of().unwrap();
                let size_i64 = emit!(self.build_int_cast(size, self.i64_type(), "size_i64"));
                let malloc_fn = self.get_or_declare_malloc();
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
                let struct_type = self.class_types[class_name.as_str()];

                let field_ptr = emit!(self.build_struct_gep(
                    struct_type,
                    obj_ptr,
                    *field_index as u32,
                    "field_ptr"
                ));

                let field_llvm_type = self.get_llvm_type(&expr.ty);
                emit!(self.build_load(field_llvm_type, field_ptr, "field_val"))
            }

            TirExprKind::MethodCall {
                object,
                method_mangled_name,
                args,
            } => {
                let self_val = self.codegen_expr(object);

                // Build full arg list: [self, ...args]
                let mut all_vals: Vec<inkwell::values::BasicValueEnum> = vec![self_val];
                all_vals.extend(self.codegen_call_args(args));

                // Declare/get method function
                let mut param_types = vec![object.ty.clone()];
                param_types.extend(args.iter().map(|a| a.ty.clone()));
                let method_fn = self.get_or_declare_function(
                    method_mangled_name,
                    &param_types,
                    Some(expr.ty.clone()),
                );

                let call_site =
                    self.build_call_maybe_invoke(method_fn, &all_vals, "method_call", true);

                self.extract_call_value(call_site)
            }

            TirExprKind::TupleLiteral {
                elements,
                element_types,
            } => {
                let struct_type = self.get_or_create_tuple_struct(element_types);
                let size = struct_type.size_of().unwrap();
                let size_i64 = emit!(self.build_int_cast(size, self.i64_type(), "tuple_size_i64"));
                let malloc_fn = self.get_or_declare_malloc();
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
                    let empty_fn = self.get_or_declare_function(
                        "__tython_list_empty",
                        &[],
                        Some(ValueType::List(Box::new(element_type.clone()))),
                    );
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
                    let list_new_fn = self.get_or_declare_list_new();
                    let call = emit!(self.build_call(
                        list_new_fn,
                        &[array_alloca.into(), len_val.into()],
                        "list_new",
                    ));
                    self.extract_call_value(call)
                }
            }

            TirExprKind::FuncRef { mangled_name } => {
                let (params, return_type) = expr.ty.unwrap_function();
                let func = self.get_or_declare_function(
                    mangled_name,
                    params,
                    return_type.as_ref().map(|b| *b.clone()),
                );
                func.as_global_value().as_pointer_value().into()
            }

            TirExprKind::IndirectCall { callee, args } => {
                let callee_ptr = self.codegen_expr(callee).into_pointer_value();
                let (param_types_vt, return_type_vt) = callee.ty.unwrap_function();

                let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types_vt
                    .iter()
                    .map(|t| self.get_llvm_type(t).into())
                    .collect();

                let rt = return_type_vt
                    .as_ref()
                    .expect("ICE: void IndirectCall in expr context");
                let fn_type = self.get_llvm_type(rt).fn_type(&llvm_params, false);

                let arg_metadata = self.codegen_call_args(args);

                let call_site = emit!(self.build_indirect_call(
                    fn_type,
                    callee_ptr,
                    &Self::to_meta_args(&arg_metadata),
                    "indirect_call"
                ));

                self.extract_call_value(call_site)
            }
        }
    }
}
