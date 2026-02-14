use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};

use crate::tir::builtin::BuiltinFn;
use crate::tir::{intrinsic_tag, IntrinsicInstance, IntrinsicOp, TirExpr, ValueType};

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
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
            _ => {
                let pred = match op {
                    IntrinsicOp::Eq => IntPredicate::EQ,
                    IntrinsicOp::Lt => IntPredicate::SLT,
                };
                emit!(self.build_int_compare(pred, lhs_slot, rhs_slot, "intrinsic_ptr_cmp"))
            }
        }
    }
}
