use inkwell::module::Linkage;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;
use inkwell::{FloatPredicate, IntPredicate};

use crate::tir::builtin::BuiltinFn;
use crate::tir::{intrinsic_tag, IntrinsicInstance, IntrinsicOp, TirExpr, ValueType};

use super::runtime_fn::{LlvmTy, RuntimeFn};
use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    fn bool_to_runtime_abi(ty: &ValueType) -> ValueType {
        if matches!(ty, ValueType::Bool) {
            ValueType::Int
        } else {
            ty.clone()
        }
    }

    fn bool_from_runtime_abi(
        &self,
        val: BasicValueEnum<'ctx>,
        ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        if matches!(ty, ValueType::Bool) {
            emit!(self.build_int_truncate(
                val.into_int_value(),
                self.context.bool_type(),
                "abi_i64_to_i1"
            ))
            .into()
        } else {
            val
        }
    }

    fn bool_to_runtime_abi_arg(
        &self,
        val: BasicValueEnum<'ctx>,
        ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        if matches!(ty, ValueType::Bool) {
            emit!(self.build_int_z_extend(val.into_int_value(), self.i64_type(), "abi_i1_to_i64"))
                .into()
        } else {
            val
        }
    }

    /// Extract the return value from a call to a function known to return non-void.
    /// This is an LLVM API contract — the function has a non-void return type in IR.
    pub(crate) fn extract_call_value(
        &self,
        call_site: inkwell::values::CallSiteValue<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        call_site.try_as_basic_value().basic().unwrap()
    }

    pub(crate) fn get_or_declare_function(
        &self,
        name: &str,
        param_types: &[ValueType],
        return_type: Option<ValueType>,
    ) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| {
            let llvm_params: Vec<BasicMetadataTypeEnum> = param_types
                .iter()
                .map(|t| self.get_llvm_type(t).into())
                .collect();

            let fn_type = match return_type {
                None => self.context.void_type().fn_type(&llvm_params, false),
                Some(ref ty) => self.get_llvm_type(ty).fn_type(&llvm_params, false),
            };

            self.module.add_function(name, fn_type, None)
        })
    }

    pub(crate) fn get_builtin(&self, builtin: BuiltinFn) -> FunctionValue<'ctx> {
        let param_types: Vec<ValueType> = builtin
            .param_types()
            .iter()
            .map(Self::bool_to_runtime_abi)
            .collect();
        let return_type = builtin.return_type().map(|t| Self::bool_to_runtime_abi(&t));
        self.get_or_declare_function(builtin.symbol(), &param_types, return_type)
    }

    pub(crate) fn resolve_llvm_ty(&self, ty: &LlvmTy) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            LlvmTy::I64 => self.context.i64_type().into(),
            LlvmTy::I32 => self.context.i32_type().into(),
            LlvmTy::Ptr => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }

    pub(crate) fn get_runtime_fn(&self, rt: RuntimeFn) -> FunctionValue<'ctx> {
        let name = rt.symbol();
        if let Some(f) = self.module.get_function(name) {
            return f;
        }

        let params: Vec<BasicMetadataTypeEnum> = rt
            .params()
            .iter()
            .map(|ty| self.resolve_llvm_ty(ty).into())
            .collect();
        let is_variadic = matches!(rt, RuntimeFn::Personality);

        let fn_type = match rt.ret() {
            None => self.context.void_type().fn_type(&params, is_variadic),
            Some(ret) => self.resolve_llvm_ty(&ret).fn_type(&params, is_variadic),
        };

        let linkage = if matches!(rt, RuntimeFn::Personality) {
            Some(Linkage::External)
        } else {
            None
        };

        let func = self.module.add_function(name, fn_type, linkage);

        if matches!(rt, RuntimeFn::CxaRethrow) {
            func.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                self.context.create_enum_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("noreturn"),
                    0,
                ),
            );
        }

        func
    }

    /// Convert `BasicValueEnum` args to `BasicMetadataValueEnum` for `build_call`.
    pub(crate) fn to_meta_args(args: &[BasicValueEnum<'ctx>]) -> Vec<BasicMetadataValueEnum<'ctx>> {
        args.iter().copied().map(Into::into).collect()
    }

    /// Get or declare an LLVM intrinsic function by name.
    pub(crate) fn get_llvm_intrinsic(
        &self,
        name: &str,
        fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        self.module
            .get_function(name)
            .unwrap_or_else(|| self.module.add_function(name, fn_type, None))
    }

    pub(crate) fn bitcast_to_i64(
        &self,
        val: BasicValueEnum<'ctx>,
        elem_ty: &ValueType,
    ) -> inkwell::values::IntValue<'ctx> {
        match elem_ty {
            ValueType::Int => val.into_int_value(),
            ValueType::Bool => {
                emit!(self.build_int_z_extend(val.into_int_value(), self.i64_type(), "b2i64"))
            }
            ValueType::Float => {
                emit!(self.build_bit_cast(val, self.i64_type(), "f2i")).into_int_value()
            }
            _ => emit!(self.build_ptr_to_int(val.into_pointer_value(), self.i64_type(), "p2i")),
        }
    }

    pub(crate) fn bitcast_from_i64(
        &self,
        val: inkwell::values::IntValue<'ctx>,
        elem_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        match elem_ty {
            ValueType::Int => val.into(),
            ValueType::Bool => {
                emit!(self.build_int_truncate(val, self.context.bool_type(), "i64_to_b")).into()
            }
            ValueType::Float => emit!(self.build_bit_cast(val, self.f64_type(), "i2f")),
            _ => emit!(self.build_int_to_ptr(
                val,
                self.context.ptr_type(AddressSpace::default()),
                "i2p"
            ))
            .into(),
        }
    }

    /// Codegen a list of TIR args into basic values.
    pub(crate) fn codegen_call_args(&mut self, args: &[TirExpr]) -> Vec<BasicValueEnum<'ctx>> {
        args.iter().map(|arg| self.codegen_expr(arg)).collect()
    }

    pub(crate) fn register_intrinsic_instances(&mut self, instances: &[IntrinsicInstance]) {
        for inst in instances {
            match inst.op {
                IntrinsicOp::Eq => {
                    self.intrinsic_eq_cases
                        .entry(inst.tag)
                        .or_insert_with(|| inst.ty.clone());
                }
                IntrinsicOp::Lt => {
                    self.intrinsic_lt_cases
                        .entry(inst.tag)
                        .or_insert_with(|| inst.ty.clone());
                }
            }
        }
    }

    pub(crate) fn codegen_intrinsic_cmp(
        &mut self,
        op: IntrinsicOp,
        lhs: &TirExpr,
        rhs: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let lhs_val = self.codegen_expr(lhs);
        let rhs_val = self.codegen_expr(rhs);
        let lhs_slot = self.bitcast_to_i64(lhs_val, &lhs.ty);
        let rhs_slot = self.bitcast_to_i64(rhs_val, &rhs.ty);
        let tag = intrinsic_tag(op, &lhs.ty);
        let symbol = match op {
            IntrinsicOp::Eq => "__tython_intrinsic_eq",
            IntrinsicOp::Lt => "__tython_intrinsic_lt",
        };
        let f = self.get_or_declare_function(
            symbol,
            &[ValueType::Int, ValueType::Int, ValueType::Int],
            Some(ValueType::Int),
        );
        let call = emit!(self.build_call(
            f,
            &[
                self.i64_type().const_int(tag as u64, true).into(),
                lhs_slot.into(),
                rhs_slot.into(),
            ],
            "intrinsic_cmp"
        ));
        let out = self.extract_call_value(call).into_int_value();
        emit!(self.build_int_compare(
            IntPredicate::NE,
            out,
            self.i64_type().const_zero(),
            "intrinsic_cmp_b"
        ))
        .into()
    }

    fn emit_intrinsic_compare_dispatcher(
        &mut self,
        op: IntrinsicOp,
        symbol: &str,
        cases: &[(i64, ValueType)],
    ) {
        let f = self.get_or_declare_function(
            symbol,
            &[ValueType::Int, ValueType::Int, ValueType::Int],
            Some(ValueType::Int),
        );
        if f.get_first_basic_block().is_some() {
            return;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let tag_val = f.get_nth_param(0).unwrap().into_int_value();
        let lhs_slot = f.get_nth_param(1).unwrap().into_int_value();
        let rhs_slot = f.get_nth_param(2).unwrap().into_int_value();

        let default_bb = self.context.append_basic_block(f, "default");
        let mut switch_cases = Vec::new();
        let mut case_blocks = Vec::new();
        for (tag, _) in cases {
            let bb = self.context.append_basic_block(f, "case");
            switch_cases.push((self.i64_type().const_int(*tag as u64, true), bb));
            case_blocks.push(bb);
        }
        emit!(self.build_switch(tag_val, default_bb, &switch_cases));

        for ((_, ty), bb) in cases.iter().zip(case_blocks.iter()) {
            self.builder.position_at_end(*bb);
            let out_b = self.intrinsic_compare_slots(op, ty, lhs_slot, rhs_slot);
            let out = emit!(self.build_int_z_extend(out_b, self.i64_type(), "b_to_i64"));
            emit!(self.build_return(Some(&out)));
        }

        self.builder.position_at_end(default_bb);
        let tag = self.i64_type().const_int(6, false).into();
        let msg = self.codegen_str_literal("unknown intrinsic compare tag");
        self.emit_raise(tag, msg);

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
    }

    pub(crate) fn emit_intrinsic_dispatchers(&mut self) {
        let mut eq_cases = self
            .intrinsic_eq_cases
            .iter()
            .map(|(tag, ty)| (*tag, ty.clone()))
            .collect::<Vec<_>>();
        eq_cases.sort_by_key(|(tag, _)| *tag);
        let mut lt_cases = self
            .intrinsic_lt_cases
            .iter()
            .map(|(tag, ty)| (*tag, ty.clone()))
            .collect::<Vec<_>>();
        lt_cases.sort_by_key(|(tag, _)| *tag);
        self.emit_intrinsic_compare_dispatcher(IntrinsicOp::Eq, "__tython_intrinsic_eq", &eq_cases);
        self.emit_intrinsic_compare_dispatcher(IntrinsicOp::Lt, "__tython_intrinsic_lt", &lt_cases);
        // Hash dispatcher uses the same eq_tag values — needed by set/dict by_tag operations
        self.emit_intrinsic_hash_dispatcher("__tython_intrinsic_hash", &eq_cases);
    }

    fn emit_intrinsic_hash_dispatcher(&mut self, symbol: &str, cases: &[(i64, ValueType)]) {
        let f = self.get_or_declare_function(
            symbol,
            &[ValueType::Int, ValueType::Int],
            Some(ValueType::Int),
        );
        if f.get_first_basic_block().is_some() {
            return;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let tag_val = f.get_nth_param(0).unwrap().into_int_value();
        let value_slot = f.get_nth_param(1).unwrap().into_int_value();

        let default_bb = self.context.append_basic_block(f, "default");
        let mut switch_cases = Vec::new();
        let mut case_blocks = Vec::new();
        for (tag, _) in cases {
            let bb = self.context.append_basic_block(f, "case");
            switch_cases.push((self.i64_type().const_int(*tag as u64, true), bb));
            case_blocks.push(bb);
        }
        emit!(self.build_switch(tag_val, default_bb, &switch_cases));

        for ((_, ty), bb) in cases.iter().zip(case_blocks.iter()) {
            self.builder.position_at_end(*bb);
            let out = self.intrinsic_hash_slot(ty, value_slot);
            emit!(self.build_return(Some(&out)));
        }

        // Default: return the raw value (identity/pointer hash)
        self.builder.position_at_end(default_bb);
        emit!(self.build_return(Some(&value_slot)));

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
    }

    fn intrinsic_hash_slot(
        &mut self,
        ty: &ValueType,
        value_slot: inkwell::values::IntValue<'ctx>,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        match ty {
            ValueType::Int | ValueType::Bool => value_slot.into(),
            ValueType::Float => {
                // bitcast f64 to i64 for hashing
                value_slot.into()
            }
            ValueType::Str => {
                let str_ptr = self.bitcast_from_i64(value_slot, &ValueType::Str);
                let hash_fn = self.get_builtin(BuiltinFn::StrHash);
                let call = emit!(self.build_call(hash_fn, &[str_ptr.into()], "str_hash"));
                self.extract_call_value(call)
            }
            ValueType::Class(class_name) => {
                let hash_name = format!("{}$__hash__", class_name);
                if let Some(hash_fn) = self.module.get_function(&hash_name) {
                    let class_ty = ValueType::Class(class_name.clone());
                    let obj = self.bitcast_from_i64(value_slot, &class_ty);
                    let call = emit!(self.build_call(hash_fn, &[obj.into()], "cls_hash"));
                    self.extract_call_value(call)
                } else {
                    // No __hash__ → identity hash (pointer value)
                    value_slot.into()
                }
            }
            _ => {
                // Bytes, ByteArray, List, Tuple, etc. — use raw value
                value_slot.into()
            }
        }
    }

    fn intrinsic_compare_slots(
        &mut self,
        op: IntrinsicOp,
        ty: &ValueType,
        lhs_slot: inkwell::values::IntValue<'ctx>,
        rhs_slot: inkwell::values::IntValue<'ctx>,
    ) -> inkwell::values::IntValue<'ctx> {
        match ty {
            ValueType::Int | ValueType::Bool => {
                let pred = match op {
                    IntrinsicOp::Eq => IntPredicate::EQ,
                    IntrinsicOp::Lt => IntPredicate::SLT,
                };
                emit!(self.build_int_compare(pred, lhs_slot, rhs_slot, "intrinsic_int_cmp"))
            }
            ValueType::Float => {
                let lhs = emit!(self.build_bit_cast(lhs_slot, self.f64_type(), "lhs_f"))
                    .into_float_value();
                let rhs = emit!(self.build_bit_cast(rhs_slot, self.f64_type(), "rhs_f"))
                    .into_float_value();
                let pred = match op {
                    IntrinsicOp::Eq => FloatPredicate::OEQ,
                    IntrinsicOp::Lt => FloatPredicate::OLT,
                };
                emit!(self.build_float_compare(pred, lhs, rhs, "intrinsic_float_cmp"))
            }
            ValueType::Str => match op {
                IntrinsicOp::Eq => {
                    let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Str);
                    let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Str);
                    let eq_fn = self.get_builtin(BuiltinFn::StrEq);
                    let call = emit!(self.build_call(
                        eq_fn,
                        &[lhs.into(), rhs.into()],
                        "intrinsic_str_eq"
                    ));
                    let v = self.extract_call_value(call).into_int_value();
                    emit!(self.build_int_compare(
                        IntPredicate::NE,
                        v,
                        self.i64_type().const_zero(),
                        "intrinsic_str_eq_b"
                    ))
                }
                IntrinsicOp::Lt => {
                    let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Str);
                    let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Str);
                    let cmp_fn = self.get_builtin(BuiltinFn::StrCmp);
                    let call = emit!(self.build_call(
                        cmp_fn,
                        &[lhs.into(), rhs.into()],
                        "intrinsic_str_cmp"
                    ));
                    let v = self.extract_call_value(call).into_int_value();
                    emit!(self.build_int_compare(
                        IntPredicate::SLT,
                        v,
                        self.i64_type().const_zero(),
                        "intrinsic_str_lt"
                    ))
                }
            },
            ValueType::Bytes => match op {
                IntrinsicOp::Eq => {
                    let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Bytes);
                    let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Bytes);
                    let eq_fn = self.get_builtin(BuiltinFn::BytesEq);
                    let call = emit!(self.build_call(
                        eq_fn,
                        &[lhs.into(), rhs.into()],
                        "intrinsic_bytes_eq"
                    ));
                    let v = self.extract_call_value(call).into_int_value();
                    emit!(self.build_int_compare(
                        IntPredicate::NE,
                        v,
                        self.i64_type().const_zero(),
                        "intrinsic_bytes_eq_b"
                    ))
                }
                IntrinsicOp::Lt => {
                    let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Bytes);
                    let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Bytes);
                    let cmp_fn = self.get_builtin(BuiltinFn::BytesCmp);
                    let call = emit!(self.build_call(
                        cmp_fn,
                        &[lhs.into(), rhs.into()],
                        "intrinsic_bytes_cmp"
                    ));
                    let v = self.extract_call_value(call).into_int_value();
                    emit!(self.build_int_compare(
                        IntPredicate::SLT,
                        v,
                        self.i64_type().const_zero(),
                        "intrinsic_bytes_lt"
                    ))
                }
            },
            ValueType::ByteArray => match op {
                IntrinsicOp::Eq => {
                    let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::ByteArray);
                    let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::ByteArray);
                    let eq_fn = self.get_builtin(BuiltinFn::ByteArrayEq);
                    let call =
                        emit!(self.build_call(eq_fn, &[lhs.into(), rhs.into()], "intrinsic_ba_eq"));
                    let v = self.extract_call_value(call).into_int_value();
                    emit!(self.build_int_compare(
                        IntPredicate::NE,
                        v,
                        self.i64_type().const_zero(),
                        "intrinsic_ba_eq_b"
                    ))
                }
                IntrinsicOp::Lt => {
                    let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::ByteArray);
                    let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::ByteArray);
                    let cmp_fn = self.get_builtin(BuiltinFn::ByteArrayCmp);
                    let call = emit!(self.build_call(
                        cmp_fn,
                        &[lhs.into(), rhs.into()],
                        "intrinsic_ba_cmp"
                    ));
                    let v = self.extract_call_value(call).into_int_value();
                    emit!(self.build_int_compare(
                        IntPredicate::SLT,
                        v,
                        self.i64_type().const_zero(),
                        "intrinsic_ba_lt"
                    ))
                }
            },
            ValueType::Class(class_name) => {
                let class_ty = ValueType::Class(class_name.clone());
                match op {
                    IntrinsicOp::Eq => {
                        let eq_name = format!("{}$__eq__", class_name);
                        if let Some(eq_fn) = self.module.get_function(&eq_name) {
                            let lhs = self.bitcast_from_i64(lhs_slot, &class_ty);
                            let rhs = self.bitcast_from_i64(rhs_slot, &class_ty);
                            let call = emit!(self.build_call(
                                eq_fn,
                                &[lhs.into(), rhs.into()],
                                "intrinsic_cls_eq"
                            ));
                            self.extract_call_value(call).into_int_value()
                        } else {
                            // Python's default object.__eq__ fallback is identity.
                            emit!(self.build_int_compare(
                                IntPredicate::EQ,
                                lhs_slot,
                                rhs_slot,
                                "intrinsic_cls_eq_identity"
                            ))
                        }
                    }
                    IntrinsicOp::Lt => {
                        let lhs = self.bitcast_from_i64(lhs_slot, &class_ty);
                        let rhs = self.bitcast_from_i64(rhs_slot, &class_ty);
                        let lt_name = format!("{}$__lt__", class_name);
                        let lt_fn = self.get_or_declare_function(
                            &lt_name,
                            &[class_ty.clone(), class_ty],
                            Some(ValueType::Bool),
                        );
                        let call = emit!(self.build_call(
                            lt_fn,
                            &[lhs.into(), rhs.into()],
                            "intrinsic_cls_lt"
                        ));
                        self.extract_call_value(call).into_int_value()
                    }
                }
            }
            ValueType::List(inner) => {
                let list_ty = ValueType::List(Box::new((**inner).clone()));
                let lhs = self.bitcast_from_i64(lhs_slot, &list_ty);
                let rhs = self.bitcast_from_i64(rhs_slot, &list_ty);
                let child_tag = intrinsic_tag(op, inner);
                let f = self.get_builtin(match op {
                    IntrinsicOp::Eq => BuiltinFn::ListEqByTag,
                    IntrinsicOp::Lt => BuiltinFn::ListLtByTag,
                });
                let call = emit!(self.build_call(
                    f,
                    &[
                        lhs.into(),
                        rhs.into(),
                        self.i64_type().const_int(child_tag as u64, true).into(),
                    ],
                    "intrinsic_list_cmp"
                ));
                let out = self.extract_call_value(call).into_int_value();
                emit!(self.build_int_compare(
                    IntPredicate::NE,
                    out,
                    self.i64_type().const_zero(),
                    "intrinsic_list_cmp_b"
                ))
            }
            ValueType::Tuple(fields) if matches!(op, IntrinsicOp::Eq) => {
                let tuple_ty = ValueType::Tuple(fields.clone());
                let lhs = self
                    .bitcast_from_i64(lhs_slot, &tuple_ty)
                    .into_pointer_value();
                let rhs = self
                    .bitcast_from_i64(rhs_slot, &tuple_ty)
                    .into_pointer_value();
                let tuple_struct = self.get_or_create_tuple_struct(fields);
                let mut all_eq = self.bool_type().const_int(1, false);
                for (idx, field_ty) in fields.iter().enumerate() {
                    let lhs_field_ptr = emit!(self.build_struct_gep(
                        tuple_struct,
                        lhs,
                        idx as u32,
                        "intrinsic_tuple_lhs_ptr"
                    ));
                    let rhs_field_ptr = emit!(self.build_struct_gep(
                        tuple_struct,
                        rhs,
                        idx as u32,
                        "intrinsic_tuple_rhs_ptr"
                    ));
                    let lhs_val = emit!(self.build_load(
                        self.get_llvm_type(field_ty),
                        lhs_field_ptr,
                        "intrinsic_tuple_lhs"
                    ));
                    let rhs_val = emit!(self.build_load(
                        self.get_llvm_type(field_ty),
                        rhs_field_ptr,
                        "intrinsic_tuple_rhs"
                    ));
                    let lhs_field_slot = self.bitcast_to_i64(lhs_val, field_ty);
                    let rhs_field_slot = self.bitcast_to_i64(rhs_val, field_ty);
                    let feq = self.intrinsic_compare_slots(
                        IntrinsicOp::Eq,
                        field_ty,
                        lhs_field_slot,
                        rhs_field_slot,
                    );
                    all_eq = emit!(self.build_and(all_eq, feq, "intrinsic_tuple_and"));
                }
                all_eq
            }
            _ => {
                let pred = match op {
                    IntrinsicOp::Eq => IntPredicate::EQ,
                    IntrinsicOp::Lt => IntPredicate::SLT,
                };
                emit!(self.build_int_compare(pred, lhs_slot, rhs_slot, "intrinsic_ptr_cmp"))
            }
        }
    }

    /// Codegen a call to a user-defined function, returning its value if non-void.
    pub(crate) fn codegen_named_call(
        &mut self,
        func: &str,
        args: &[TirExpr],
        return_type: Option<&ValueType>,
    ) -> Option<BasicValueEnum<'ctx>> {
        let arg_types: Vec<ValueType> = args.iter().map(|a| a.ty.clone()).collect();
        let function = self.get_or_declare_function(func, &arg_types, return_type.cloned());
        let arg_values = self.codegen_call_args(args);
        let call_site = self.build_call_maybe_invoke(function, &arg_values, "call", true);
        return_type.map(|_| self.extract_call_value(call_site))
    }

    /// Codegen a call to a builtin (runtime) function.
    ///
    /// Handles container-element bitcasting conventions automatically:
    /// - `ListPop`/`ListGet` return an i64 slot that is bitcast to the element type.
    /// - `DictGet`/`DictPop`/`SetPop` return an i64 slot that is bitcast.
    /// - `ListAppend`/`ListRemove`/`ListInsert`/`ListContains`/`ListIndex`/`ListCount`
    ///   take an element as the **last** argument which is bitcast *to* i64.
    pub(crate) fn codegen_builtin_call(
        &mut self,
        func: BuiltinFn,
        args: &[TirExpr],
        result_ty: Option<&ValueType>,
    ) -> Option<BasicValueEnum<'ctx>> {
        let function = self.get_builtin(func);

        // DictGet/DictPop variants need both:
        // - key (arg1) bitcasted to i64
        // - returned slot bitcasted from i64 to the value type
        if matches!(
            func,
            BuiltinFn::DictGet
                | BuiltinFn::DictPop
                | BuiltinFn::DictGetByTag
                | BuiltinFn::DictPopByTag
                | BuiltinFn::DictGetDefaultByTag
                | BuiltinFn::DictPopDefaultByTag
                | BuiltinFn::DictSetDefaultByTag
        ) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1
                    || i == 2
                        && matches!(
                            func,
                            BuiltinFn::DictGetDefaultByTag
                                | BuiltinFn::DictPopDefaultByTag
                                | BuiltinFn::DictSetDefaultByTag
                        )
                {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(self.bool_to_runtime_abi_arg(val, &arg.ty).into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            let i64_val = self.extract_call_value(call).into_int_value();
            return Some(self.bitcast_from_i64(i64_val, result_ty.unwrap()));
        }

        // TupleGetItem — use codegen_tuple_get_dynamic logic
        if matches!(func, BuiltinFn::TupleGetItem) {
            let tuple = &args[0];
            let index = &args[1];
            let ValueType::Tuple(elem_types) = &tuple.ty else {
                panic!("ICE: TupleGetItem on non-tuple type");
            };
            return Some(self.codegen_tuple_get_dynamic(
                tuple,
                index,
                elem_types.len(),
                elem_types,
                result_ty.unwrap(),
            ));
        }

        // List ops returning an element stored as i64 — bitcast result
        if matches!(
            func,
            BuiltinFn::ListPop | BuiltinFn::ListGet | BuiltinFn::SetPop
        ) {
            let arg_values = self.codegen_call_args(args);
            let call =
                emit!(self.build_call(function, &Self::to_meta_args(&arg_values), "builtin_call"));
            let i64_val = self.extract_call_value(call).into_int_value();
            return Some(self.bitcast_from_i64(i64_val, result_ty.unwrap()));
        }

        // Tag-based list ops where element is arg1 and tag is arg2.
        if matches!(
            func,
            BuiltinFn::ListContainsByTag
                | BuiltinFn::ListIndexByTag
                | BuiltinFn::ListCountByTag
                | BuiltinFn::ListRemoveByTag
        ) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(self.bool_to_runtime_abi_arg(val, &arg.ty).into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty
                .map(|ty| self.bool_from_runtime_abi(self.extract_call_value(call), ty));
        }

        // List ops where the last arg is an element — bitcast it to i64
        if matches!(
            func,
            BuiltinFn::ListContains
                | BuiltinFn::ListIndex
                | BuiltinFn::ListCount
                | BuiltinFn::ListAppend
                | BuiltinFn::ListRemove
                | BuiltinFn::ListInsert
        ) {
            let last = args.len() - 1;
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == last {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(self.bool_to_runtime_abi_arg(val, &arg.ty).into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty
                .map(|ty| self.bool_from_runtime_abi(self.extract_call_value(call), ty));
        }

        // Dict ops with key in position 1; set/get/pop bitcast that key.
        if matches!(
            func,
            BuiltinFn::DictContains
                | BuiltinFn::DictGet
                | BuiltinFn::DictPop
                | BuiltinFn::DictContainsByTag
                | BuiltinFn::DictDelByTag
        ) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(self.bool_to_runtime_abi_arg(val, &arg.ty).into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty
                .map(|ty| self.bool_from_runtime_abi(self.extract_call_value(call), ty));
        }

        // DictSet variants bitcast key (arg1) and value (arg2).
        if matches!(func, BuiltinFn::DictSet | BuiltinFn::DictSetByTag) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 || i == 2 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(self.bool_to_runtime_abi_arg(val, &arg.ty).into());
                }
            }
            emit!(self.build_call(function, &call_args, "builtin_call"));
            return None;
        }

        // dict.fromkeys(keys, value, key_eq_tag) bitcasts value (arg1) to i64.
        if matches!(func, BuiltinFn::DictFromKeysByTag) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(self.bool_to_runtime_abi_arg(val, &arg.ty).into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty
                .map(|ty| self.bool_from_runtime_abi(self.extract_call_value(call), ty));
        }

        // Set ops with element arg in position 1.
        if matches!(
            func,
            BuiltinFn::SetContains
                | BuiltinFn::SetAdd
                | BuiltinFn::SetRemove
                | BuiltinFn::SetDiscard
                | BuiltinFn::SetContainsByTag
                | BuiltinFn::SetAddByTag
                | BuiltinFn::SetRemoveByTag
                | BuiltinFn::SetDiscardByTag
        ) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(self.bool_to_runtime_abi_arg(val, &arg.ty).into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty
                .map(|ty| self.bool_from_runtime_abi(self.extract_call_value(call), ty));
        }

        // General case — no bitcasting
        let mut arg_values = Vec::with_capacity(args.len());
        let param_types = func.param_types();
        for (i, arg) in args.iter().enumerate() {
            let v = self.codegen_expr(arg);
            arg_values.push(self.bool_to_runtime_abi_arg(v, &param_types[i]));
        }
        let call =
            emit!(self.build_call(function, &Self::to_meta_args(&arg_values), "builtin_call"));
        result_ty.map(|ty| self.bool_from_runtime_abi(self.extract_call_value(call), ty))
    }
}
