use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::module::Linkage;
use inkwell::types::BasicType;
use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};

use crate::tir::builtin::BuiltinFn;
use crate::tir::{
    intrinsic_tag, CmpIntrinsicOp, IntrinsicInstance, IntrinsicOp, TirExpr, TirExprKind, ValueType,
};

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
                IntrinsicOp::Str => {
                    self.intrinsic_str_cases
                        .entry(inst.tag)
                        .or_insert_with(|| inst.ty.clone());
                }
            }
        }
    }

    fn tag_suffix(tag: i64) -> String {
        format!("{:016x}", tag as u64)
    }

    fn cmp_kernel_symbol(op: CmpIntrinsicOp, tag: i64) -> String {
        let op_name = match op {
            CmpIntrinsicOp::Eq => "eq",
            CmpIntrinsicOp::Lt => "lt",
        };
        format!("__tython_intrinsic_{}${}", op_name, Self::tag_suffix(tag))
    }

    fn hash_kernel_symbol(tag: i64) -> String {
        format!("__tython_intrinsic_hash${}", Self::tag_suffix(tag))
    }

    fn str_kernel_symbol(tag: i64) -> String {
        format!("__tython_intrinsic_str${}", Self::tag_suffix(tag))
    }

    fn eq_ops_global_symbol(tag: i64) -> String {
        format!("__tython_eq_ops${}", Self::tag_suffix(tag))
    }

    fn lt_ops_global_symbol(tag: i64) -> String {
        format!("__tython_lt_ops${}", Self::tag_suffix(tag))
    }

    fn str_ops_global_symbol(tag: i64) -> String {
        format!("__tython_str_ops${}", Self::tag_suffix(tag))
    }

    fn mark_always_inline(&self, f: inkwell::values::FunctionValue<'ctx>) {
        let kind = Attribute::get_named_enum_kind_id("alwaysinline");
        f.add_attribute(
            AttributeLoc::Function,
            self.context.create_enum_attribute(kind, 0),
        );
    }

    fn get_or_declare_internal_fn(
        &self,
        symbol: &str,
        fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> inkwell::values::FunctionValue<'ctx> {
        self.module.get_function(symbol).unwrap_or_else(|| {
            self.module
                .add_function(symbol, fn_type, Some(Linkage::Internal))
        })
    }

    fn emit_intrinsic_cmp_kernel(
        &mut self,
        op: CmpIntrinsicOp,
        tag: i64,
        ty: &ValueType,
    ) -> inkwell::values::FunctionValue<'ctx> {
        let symbol = Self::cmp_kernel_symbol(op, tag);
        let fn_type = self
            .i64_type()
            .fn_type(&[self.i64_type().into(), self.i64_type().into()], false);
        let f = self.get_or_declare_internal_fn(&symbol, fn_type);
        self.mark_always_inline(f);
        if f.get_first_basic_block().is_some() {
            return f;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let lhs_slot = f.get_nth_param(0).unwrap().into_int_value();
        let rhs_slot = f.get_nth_param(1).unwrap().into_int_value();
        let out_b = self.intrinsic_compare_slots(op, ty, lhs_slot, rhs_slot);
        let out = emit!(self.build_int_z_extend(out_b, self.i64_type(), "b_to_i64"));
        emit!(self.build_return(Some(&out)));

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
        f
    }

    fn emit_intrinsic_hash_kernel(
        &mut self,
        tag: i64,
        ty: &ValueType,
    ) -> inkwell::values::FunctionValue<'ctx> {
        let symbol = Self::hash_kernel_symbol(tag);
        let fn_type = self.i64_type().fn_type(&[self.i64_type().into()], false);
        let f = self.get_or_declare_internal_fn(&symbol, fn_type);
        self.mark_always_inline(f);
        if f.get_first_basic_block().is_some() {
            return f;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let value_slot = f.get_nth_param(0).unwrap().into_int_value();
        let out = self.intrinsic_hash_slot(ty, value_slot).into_int_value();
        emit!(self.build_return(Some(&out)));

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
        f
    }

    fn emit_intrinsic_str_kernel(
        &mut self,
        tag: i64,
        ty: &ValueType,
    ) -> inkwell::values::FunctionValue<'ctx> {
        let symbol = Self::str_kernel_symbol(tag);
        let fn_type = self
            .get_llvm_type(&ValueType::Str)
            .fn_type(&[self.i64_type().into()], false);
        let f = self.get_or_declare_internal_fn(&symbol, fn_type);
        self.mark_always_inline(f);
        if f.get_first_basic_block().is_some() {
            return f;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let obj_slot = f.get_nth_param(0).unwrap().into_int_value();
        let out = self.intrinsic_str_slot(ty, obj_slot);
        emit!(self.build_return(Some(&out)));

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
        f
    }

    fn emit_eq_ops_global(
        &mut self,
        tag: i64,
        ty: &ValueType,
    ) -> inkwell::values::GlobalValue<'ctx> {
        let symbol = Self::eq_ops_global_symbol(tag);
        if let Some(g) = self.module.get_global(&symbol) {
            return g;
        }

        let eq_fn = self.emit_intrinsic_cmp_kernel(CmpIntrinsicOp::Eq, tag, ty);
        let hash_fn = self.emit_intrinsic_hash_kernel(tag, ty);

        let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
        let ops_ty = self
            .context
            .struct_type(&[ptr_ty.into(), ptr_ty.into()], false);
        let init = ops_ty.const_named_struct(&[
            eq_fn.as_global_value().as_pointer_value().into(),
            hash_fn.as_global_value().as_pointer_value().into(),
        ]);

        let g = self.module.add_global(ops_ty, None, &symbol);
        g.set_linkage(Linkage::Internal);
        g.set_constant(true);
        g.set_initializer(&init);
        g
    }

    fn emit_lt_ops_global(
        &mut self,
        tag: i64,
        ty: &ValueType,
    ) -> inkwell::values::GlobalValue<'ctx> {
        let symbol = Self::lt_ops_global_symbol(tag);
        if let Some(g) = self.module.get_global(&symbol) {
            return g;
        }

        let lt_fn = self.emit_intrinsic_cmp_kernel(CmpIntrinsicOp::Lt, tag, ty);
        let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
        let ops_ty = self.context.struct_type(&[ptr_ty.into()], false);
        let init = ops_ty.const_named_struct(&[lt_fn.as_global_value().as_pointer_value().into()]);

        let g = self.module.add_global(ops_ty, None, &symbol);
        g.set_linkage(Linkage::Internal);
        g.set_constant(true);
        g.set_initializer(&init);
        g
    }

    fn emit_str_ops_global(
        &mut self,
        tag: i64,
        ty: &ValueType,
    ) -> inkwell::values::GlobalValue<'ctx> {
        let symbol = Self::str_ops_global_symbol(tag);
        if let Some(g) = self.module.get_global(&symbol) {
            return g;
        }

        let str_fn = self.emit_intrinsic_str_kernel(tag, ty);
        let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
        let ops_ty = self.context.struct_type(&[ptr_ty.into()], false);
        let init = ops_ty.const_named_struct(&[str_fn.as_global_value().as_pointer_value().into()]);

        let g = self.module.add_global(ops_ty, None, &symbol);
        g.set_linkage(Linkage::Internal);
        g.set_constant(true);
        g.set_initializer(&init);
        g
    }

    fn emit_ops_handle_bridge(
        &mut self,
        symbol: &str,
        cases: &[(i64, ValueType)],
        op: IntrinsicOp,
    ) {
        let f = self.get_or_declare_function(symbol, &[ValueType::Int], Some(ValueType::Int));
        if f.get_first_basic_block().is_some() {
            return;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let tag_val = f.get_nth_param(0).unwrap().into_int_value();
        let default_bb = self.context.append_basic_block(f, "default");
        let mut switch_cases = Vec::new();
        let mut case_blocks = Vec::new();
        for (tag, _) in cases {
            let bb = self.context.append_basic_block(f, "case");
            switch_cases.push((self.i64_type().const_int(*tag as u64, true), bb));
            case_blocks.push(bb);
        }
        emit!(self.build_switch(tag_val, default_bb, &switch_cases));

        for ((tag, ty), bb) in cases.iter().zip(case_blocks.iter()) {
            self.builder.position_at_end(*bb);
            let handle = match op {
                IntrinsicOp::Eq => self
                    .emit_eq_ops_global(*tag, ty)
                    .as_pointer_value()
                    .const_to_int(self.i64_type()),
                IntrinsicOp::Lt => self
                    .emit_lt_ops_global(*tag, ty)
                    .as_pointer_value()
                    .const_to_int(self.i64_type()),
                IntrinsicOp::Str => self
                    .emit_str_ops_global(*tag, ty)
                    .as_pointer_value()
                    .const_to_int(self.i64_type()),
            };
            emit!(self.build_return(Some(&handle)));
        }

        self.builder.position_at_end(default_bb);
        let exc_tag = self.i64_type().const_int(6, false).into();
        let msg = self.codegen_str_literal("unknown intrinsic ops tag");
        self.emit_raise(exc_tag, msg);

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
    }

    fn intrinsic_type_for_tag(&self, op: IntrinsicOp, tag: i64) -> Option<ValueType> {
        match op {
            IntrinsicOp::Eq => self.intrinsic_eq_cases.get(&tag).cloned(),
            IntrinsicOp::Lt => self.intrinsic_lt_cases.get(&tag).cloned(),
            IntrinsicOp::Str => self.intrinsic_str_cases.get(&tag).cloned(),
        }
    }

    pub(crate) fn codegen_intrinsic_ops_handle(
        &mut self,
        op: IntrinsicOp,
        tag_expr: &TirExpr,
    ) -> inkwell::values::IntValue<'ctx> {
        if let TirExprKind::IntLiteral(tag) = &tag_expr.kind {
            let tag = *tag;
            if let Some(ty) = self.intrinsic_type_for_tag(op, tag) {
                return match op {
                    IntrinsicOp::Eq => self
                        .emit_eq_ops_global(tag, &ty)
                        .as_pointer_value()
                        .const_to_int(self.i64_type()),
                    IntrinsicOp::Lt => self
                        .emit_lt_ops_global(tag, &ty)
                        .as_pointer_value()
                        .const_to_int(self.i64_type()),
                    IntrinsicOp::Str => self
                        .emit_str_ops_global(tag, &ty)
                        .as_pointer_value()
                        .const_to_int(self.i64_type()),
                };
            }
        }

        let tag_val = self.codegen_expr(tag_expr).into_int_value();
        let bridge_name = match op {
            IntrinsicOp::Eq => "__tython_eq_ops_from_tag",
            IntrinsicOp::Lt => "__tython_lt_ops_from_tag",
            IntrinsicOp::Str => "__tython_str_ops_from_tag",
        };
        let bridge =
            self.get_or_declare_function(bridge_name, &[ValueType::Int], Some(ValueType::Int));
        let call = emit!(self.build_call(bridge, &[tag_val.into()], "ops_from_tag"));
        self.extract_call_value(call).into_int_value()
    }

    pub(crate) fn codegen_intrinsic_cmp(
        &mut self,
        op: CmpIntrinsicOp,
        lhs: &TirExpr,
        rhs: &TirExpr,
    ) -> BasicValueEnum<'ctx> {
        let lhs_val = self.codegen_expr(lhs);
        let rhs_val = self.codegen_expr(rhs);
        let lhs_slot = self.bitcast_to_i64(lhs_val, &lhs.ty);
        let rhs_slot = self.bitcast_to_i64(rhs_val, &rhs.ty);
        let tag = intrinsic_tag(op.into(), &lhs.ty);
        let f = self.emit_intrinsic_cmp_kernel(op, tag, &lhs.ty);
        let call = emit!(self.build_call(f, &[lhs_slot.into(), rhs_slot.into(),], "intrinsic_cmp"));
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
        op: CmpIntrinsicOp,
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

        for ((tag, ty), bb) in cases.iter().zip(case_blocks.iter()) {
            self.builder.position_at_end(*bb);
            let kernel = self.emit_intrinsic_cmp_kernel(op, *tag, ty);
            let call = emit!(self.build_call(
                kernel,
                &[lhs_slot.into(), rhs_slot.into()],
                "intrinsic_cmp_kernel"
            ));
            let out = self.extract_call_value(call).into_int_value();
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
        let mut str_cases = self
            .intrinsic_str_cases
            .iter()
            .map(|(tag, ty)| (*tag, ty.clone()))
            .collect::<Vec<_>>();
        str_cases.sort_by_key(|(tag, _)| *tag);

        for (tag, ty) in &eq_cases {
            let _ = self.emit_eq_ops_global(*tag, ty);
        }
        for (tag, ty) in &lt_cases {
            let _ = self.emit_lt_ops_global(*tag, ty);
        }
        for (tag, ty) in &str_cases {
            let _ = self.emit_str_ops_global(*tag, ty);
        }
        self.emit_ops_handle_bridge("__tython_eq_ops_from_tag", &eq_cases, IntrinsicOp::Eq);
        self.emit_ops_handle_bridge("__tython_lt_ops_from_tag", &lt_cases, IntrinsicOp::Lt);
        self.emit_ops_handle_bridge("__tython_str_ops_from_tag", &str_cases, IntrinsicOp::Str);

        self.emit_intrinsic_compare_dispatcher(
            CmpIntrinsicOp::Eq,
            "__tython_intrinsic_eq",
            &eq_cases,
        );
        self.emit_intrinsic_compare_dispatcher(
            CmpIntrinsicOp::Lt,
            "__tython_intrinsic_lt",
            &lt_cases,
        );
        // Hash dispatcher uses the same eq_tag values — needed by set/dict by_tag operations
        self.emit_intrinsic_hash_dispatcher("__tython_intrinsic_hash", &eq_cases);
        self.emit_intrinsic_str_dispatcher("__tython_intrinsic_str", &str_cases);
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

        for ((tag, ty), bb) in cases.iter().zip(case_blocks.iter()) {
            self.builder.position_at_end(*bb);
            let kernel = self.emit_intrinsic_hash_kernel(*tag, ty);
            let call =
                emit!(self.build_call(kernel, &[value_slot.into()], "intrinsic_hash_kernel"));
            let out = self.extract_call_value(call).into_int_value();
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
        op: CmpIntrinsicOp,
        ty: &ValueType,
        lhs_slot: inkwell::values::IntValue<'ctx>,
        rhs_slot: inkwell::values::IntValue<'ctx>,
    ) -> inkwell::values::IntValue<'ctx> {
        match ty {
            ValueType::Int | ValueType::Bool => {
                let pred = match op {
                    CmpIntrinsicOp::Eq => IntPredicate::EQ,
                    CmpIntrinsicOp::Lt => IntPredicate::SLT,
                };
                emit!(self.build_int_compare(pred, lhs_slot, rhs_slot, "intrinsic_int_cmp"))
            }
            ValueType::Float => {
                let lhs = emit!(self.build_bit_cast(lhs_slot, self.f64_type(), "lhs_f"))
                    .into_float_value();
                let rhs = emit!(self.build_bit_cast(rhs_slot, self.f64_type(), "rhs_f"))
                    .into_float_value();
                let pred = match op {
                    CmpIntrinsicOp::Eq => FloatPredicate::OEQ,
                    CmpIntrinsicOp::Lt => FloatPredicate::OLT,
                };
                emit!(self.build_float_compare(pred, lhs, rhs, "intrinsic_float_cmp"))
            }
            ValueType::Str => match op {
                CmpIntrinsicOp::Eq => {
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
                CmpIntrinsicOp::Lt => {
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
                CmpIntrinsicOp::Eq => {
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
                CmpIntrinsicOp::Lt => {
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
                CmpIntrinsicOp::Eq => {
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
                CmpIntrinsicOp::Lt => {
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
                    CmpIntrinsicOp::Eq => {
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
                    CmpIntrinsicOp::Lt => {
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
                let child_tag = intrinsic_tag(op.into(), inner);
                let child_ops_handle = match op {
                    CmpIntrinsicOp::Eq => self
                        .emit_eq_ops_global(child_tag, inner)
                        .as_pointer_value()
                        .const_to_int(self.i64_type()),
                    CmpIntrinsicOp::Lt => self
                        .emit_lt_ops_global(child_tag, inner)
                        .as_pointer_value()
                        .const_to_int(self.i64_type()),
                };
                let f = self.get_builtin(match op {
                    CmpIntrinsicOp::Eq => BuiltinFn::ListEqByTag,
                    CmpIntrinsicOp::Lt => BuiltinFn::ListLtByTag,
                });
                let call = emit!(self.build_call(
                    f,
                    &[lhs.into(), rhs.into(), child_ops_handle.into(),],
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
                    CmpIntrinsicOp::Eq => IntPredicate::EQ,
                    CmpIntrinsicOp::Lt => IntPredicate::SLT,
                };
                emit!(self.build_int_compare(pred, lhs_slot, rhs_slot, "intrinsic_ptr_cmp"))
            }
        }
    }

    fn emit_intrinsic_str_dispatcher(&mut self, symbol: &str, cases: &[(i64, ValueType)]) {
        // __tython_intrinsic_str(tag: i64, obj: i64) -> ptr (char*)
        let f = self.get_or_declare_function(
            symbol,
            &[ValueType::Int, ValueType::Int],
            Some(ValueType::Str),
        );
        if f.get_first_basic_block().is_some() {
            return;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let tag_val = f.get_nth_param(0).unwrap().into_int_value();
        let obj_slot = f.get_nth_param(1).unwrap().into_int_value();

        let default_bb = self.context.append_basic_block(f, "default");
        let mut switch_cases = Vec::new();
        let mut case_blocks = Vec::new();
        for (tag, _) in cases {
            let bb = self.context.append_basic_block(f, "case");
            switch_cases.push((self.i64_type().const_int(*tag as u64, true), bb));
            case_blocks.push(bb);
        }
        emit!(self.build_switch(tag_val, default_bb, &switch_cases));

        for ((tag, ty), bb) in cases.iter().zip(case_blocks.iter()) {
            self.builder.position_at_end(*bb);
            let kernel = self.emit_intrinsic_str_kernel(*tag, ty);
            let call = emit!(self.build_call(kernel, &[obj_slot.into()], "intrinsic_str_kernel"));
            let out = self.extract_call_value(call);
            emit!(self.build_return(Some(&out)));
        }

        // Default: raise on unknown tag
        self.builder.position_at_end(default_bb);
        let tag = self.i64_type().const_int(6, false).into();
        let msg = self.codegen_str_literal("unknown intrinsic str tag");
        self.emit_raise(tag, msg);

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
    }

    fn intrinsic_str_slot(
        &mut self,
        ty: &ValueType,
        obj_slot: inkwell::values::IntValue<'ctx>,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        match ty {
            ValueType::Int => {
                let val = self.bitcast_from_i64(obj_slot, &ValueType::Int);
                let f = self.get_builtin(BuiltinFn::StrFromInt);
                let call = emit!(self.build_call(f, &[val.into()], "str_from_int"));
                self.extract_call_value(call)
            }
            ValueType::Float => {
                let val = self.bitcast_from_i64(obj_slot, &ValueType::Float);
                let f = self.get_builtin(BuiltinFn::StrFromFloat);
                let call = emit!(self.build_call(f, &[val.into()], "str_from_float"));
                self.extract_call_value(call)
            }
            ValueType::Bool => {
                let val = self.bitcast_from_i64(obj_slot, &ValueType::Bool);
                let f = self.get_builtin(BuiltinFn::StrFromBool);
                let call = emit!(self.build_call(f, &[val.into()], "str_from_bool"));
                self.extract_call_value(call)
            }
            ValueType::Str => {
                let val = self.bitcast_from_i64(obj_slot, &ValueType::Str);
                let f = self.get_builtin(BuiltinFn::ReprStr);
                let call = emit!(self.build_call(f, &[val.into()], "repr_str"));
                self.extract_call_value(call)
            }
            ValueType::Bytes => {
                let val = self.bitcast_from_i64(obj_slot, &ValueType::Bytes);
                let f = self.get_builtin(BuiltinFn::StrFromBytes);
                let call = emit!(self.build_call(f, &[val.into()], "str_from_bytes"));
                self.extract_call_value(call)
            }
            ValueType::ByteArray => {
                let val = self.bitcast_from_i64(obj_slot, &ValueType::ByteArray);
                let f = self.get_builtin(BuiltinFn::StrFromByteArray);
                let call = emit!(self.build_call(f, &[val.into()], "str_from_bytearray"));
                self.extract_call_value(call)
            }
            ValueType::List(inner) => {
                let list_ty = ValueType::List(Box::new((**inner).clone()));
                let val = self.bitcast_from_i64(obj_slot, &list_ty);
                let child_tag = intrinsic_tag(IntrinsicOp::Str, inner);
                let child_ops_handle = self
                    .emit_str_ops_global(child_tag, inner)
                    .as_pointer_value()
                    .const_to_int(self.i64_type());
                let f = self.get_builtin(BuiltinFn::ListStrByTag);
                let call = emit!(self.build_call(
                    f,
                    &[val.into(), child_ops_handle.into(),],
                    "list_str_by_tag"
                ));
                self.extract_call_value(call)
            }
            ValueType::Dict(key, value) => {
                let dict_ty =
                    ValueType::Dict(Box::new((**key).clone()), Box::new((**value).clone()));
                let val = self.bitcast_from_i64(obj_slot, &dict_ty);
                let key_tag = intrinsic_tag(IntrinsicOp::Str, key);
                let value_tag = intrinsic_tag(IntrinsicOp::Str, value);
                let key_ops_handle = self
                    .emit_str_ops_global(key_tag, key)
                    .as_pointer_value()
                    .const_to_int(self.i64_type());
                let value_ops_handle = self
                    .emit_str_ops_global(value_tag, value)
                    .as_pointer_value()
                    .const_to_int(self.i64_type());
                let f = self.get_builtin(BuiltinFn::DictStrByTag);
                let call = emit!(self.build_call(
                    f,
                    &[val.into(), key_ops_handle.into(), value_ops_handle.into(),],
                    "dict_str_by_tag"
                ));
                self.extract_call_value(call)
            }
            ValueType::Set(inner) => {
                let set_ty = ValueType::Set(Box::new((**inner).clone()));
                let val = self.bitcast_from_i64(obj_slot, &set_ty);
                let child_tag = intrinsic_tag(IntrinsicOp::Str, inner);
                let child_ops_handle = self
                    .emit_str_ops_global(child_tag, inner)
                    .as_pointer_value()
                    .const_to_int(self.i64_type());
                let f = self.get_builtin(BuiltinFn::SetStrByTag);
                let call = emit!(self.build_call(
                    f,
                    &[val.into(), child_ops_handle.into(),],
                    "set_str_by_tag"
                ));
                self.extract_call_value(call)
            }
            ValueType::Class(class_name) => {
                let class_ty = ValueType::Class(class_name.clone());
                let repr_name = format!("{}$__repr__", class_name);
                if let Some(repr_fn) = self.module.get_function(&repr_name) {
                    let obj = self.bitcast_from_i64(obj_slot, &class_ty);
                    let call = emit!(self.build_call(repr_fn, &[obj.into()], "cls_repr"));
                    self.extract_call_value(call)
                } else {
                    // Fallback: return a placeholder string
                    self.codegen_str_literal(&format!("<{} object>", class_name))
                }
            }
            _ => {
                // Fallback for Function or other types
                self.codegen_str_literal("<object>")
            }
        }
    }
}
