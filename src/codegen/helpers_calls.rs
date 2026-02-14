use inkwell::module::Linkage;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;
use inkwell::{FloatPredicate, IntPredicate};
use std::hash::{Hash, Hasher};

use crate::tir::builtin::BuiltinFn;
use crate::tir::{TirExpr, ValueType};

use super::runtime_fn::{LlvmTy, RuntimeFn};
use super::Codegen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MonoListOp {
    Eq,
    Lt,
    Contains,
    Index,
    Count,
    Remove,
    Sort,
    Sorted,
}

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

    fn list_elem_type_from_first_arg(args: &[TirExpr]) -> Option<&ValueType> {
        let first = args.first()?;
        let ValueType::List(inner) = &first.ty else {
            return None;
        };
        Some(inner.as_ref())
    }

    fn requires_mono_list_eq_ops(elem_ty: &ValueType) -> bool {
        matches!(elem_ty, ValueType::List(_) | ValueType::Class(_))
    }

    fn sort_fastpath_builtin(func: BuiltinFn, elem_ty: &ValueType) -> Option<BuiltinFn> {
        match (func, elem_ty) {
            (BuiltinFn::ListSortAny, ValueType::Int | ValueType::Bool) => {
                Some(BuiltinFn::ListSortInt)
            }
            (BuiltinFn::ListSortAny, ValueType::Float) => Some(BuiltinFn::ListSortFloat),
            (BuiltinFn::ListSortAny, ValueType::Str) => Some(BuiltinFn::ListSortStr),
            (BuiltinFn::ListSortAny, ValueType::Bytes) => Some(BuiltinFn::ListSortBytes),
            (BuiltinFn::ListSortAny, ValueType::ByteArray) => Some(BuiltinFn::ListSortByteArray),
            (BuiltinFn::SortedAny, ValueType::Int | ValueType::Bool) => Some(BuiltinFn::SortedInt),
            (BuiltinFn::SortedAny, ValueType::Float) => Some(BuiltinFn::SortedFloat),
            (BuiltinFn::SortedAny, ValueType::Str) => Some(BuiltinFn::SortedStr),
            (BuiltinFn::SortedAny, ValueType::Bytes) => Some(BuiltinFn::SortedBytes),
            (BuiltinFn::SortedAny, ValueType::ByteArray) => Some(BuiltinFn::SortedByteArray),
            _ => None,
        }
    }

    fn mono_list_type_key(ty: &ValueType) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        ty.hash(&mut hasher);
        let hash = hasher.finish();
        let base = ty.to_string();
        let sanitized = base
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>();
        format!("{}_{:x}", sanitized, hash)
    }

    fn mono_list_symbol(op: MonoListOp, elem_ty: &ValueType) -> String {
        let op_name = match op {
            MonoListOp::Eq => "eq",
            MonoListOp::Lt => "lt",
            MonoListOp::Contains => "contains",
            MonoListOp::Index => "index",
            MonoListOp::Count => "count",
            MonoListOp::Remove => "remove",
            MonoListOp::Sort => "sort",
            MonoListOp::Sorted => "sorted",
        };
        format!(
            "__tython_mono_list_{}_{}",
            op_name,
            Self::mono_list_type_key(elem_ty)
        )
    }

    fn mono_list_signature(
        op: MonoListOp,
        elem_ty: &ValueType,
    ) -> (Vec<ValueType>, Option<ValueType>) {
        let list_ty = ValueType::List(Box::new(elem_ty.clone()));
        match op {
            MonoListOp::Eq | MonoListOp::Lt => {
                (vec![list_ty.clone(), list_ty], Some(ValueType::Bool))
            }
            MonoListOp::Contains => (vec![list_ty, elem_ty.clone()], Some(ValueType::Bool)),
            MonoListOp::Index => (vec![list_ty, elem_ty.clone()], Some(ValueType::Int)),
            MonoListOp::Count => (vec![list_ty, elem_ty.clone()], Some(ValueType::Int)),
            MonoListOp::Remove => (vec![list_ty, elem_ty.clone()], None),
            MonoListOp::Sort => (vec![list_ty], None),
            MonoListOp::Sorted => (vec![list_ty.clone()], Some(list_ty)),
        }
    }

    fn ensure_mono_list_helper(
        &mut self,
        op: MonoListOp,
        elem_ty: &ValueType,
    ) -> FunctionValue<'ctx> {
        if let ValueType::List(inner) = elem_ty {
            match op {
                MonoListOp::Eq
                | MonoListOp::Contains
                | MonoListOp::Index
                | MonoListOp::Count
                | MonoListOp::Remove => {
                    self.ensure_mono_list_helper(MonoListOp::Eq, inner);
                }
                MonoListOp::Lt => {
                    self.ensure_mono_list_helper(MonoListOp::Lt, inner);
                }
                MonoListOp::Sort | MonoListOp::Sorted => {
                    self.ensure_mono_list_helper(MonoListOp::Lt, inner);
                }
            }
        }

        let symbol = Self::mono_list_symbol(op, elem_ty);
        let (params, ret) = Self::mono_list_signature(op, elem_ty);
        let function = self.get_or_declare_function(&symbol, &params, ret);
        if function.get_first_basic_block().is_some() {
            return function;
        }

        let saved_block = self.builder.get_insert_block();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.codegen_mono_list_helper_body(function, op, elem_ty);

        if let Some(bb) = saved_block {
            self.builder.position_at_end(bb);
        }
        function
    }

    fn mono_elem_eq_from_slots(
        &mut self,
        lhs_slot: inkwell::values::IntValue<'ctx>,
        rhs_slot: inkwell::values::IntValue<'ctx>,
        elem_ty: &ValueType,
    ) -> inkwell::values::IntValue<'ctx> {
        match elem_ty {
            ValueType::Int | ValueType::Bool => {
                emit!(self.build_int_compare(IntPredicate::EQ, lhs_slot, rhs_slot, "mono_eq_i"))
            }
            ValueType::Float => {
                let lhs = emit!(self.build_bit_cast(lhs_slot, self.f64_type(), "lhs_f"))
                    .into_float_value();
                let rhs = emit!(self.build_bit_cast(rhs_slot, self.f64_type(), "rhs_f"))
                    .into_float_value();
                emit!(self.build_float_compare(FloatPredicate::OEQ, lhs, rhs, "mono_eq_f"))
            }
            ValueType::Str => {
                let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Str);
                let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Str);
                let eq_fn = self.get_builtin(BuiltinFn::StrEq);
                let call = emit!(self.build_call(eq_fn, &[lhs.into(), rhs.into()], "mono_eq_str"));
                let v = self.extract_call_value(call).into_int_value();
                emit!(self.build_int_compare(
                    IntPredicate::NE,
                    v,
                    self.i64_type().const_zero(),
                    "mono_eq_str_b"
                ))
            }
            ValueType::Bytes => {
                let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Bytes);
                let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Bytes);
                let eq_fn = self.get_builtin(BuiltinFn::BytesEq);
                let call =
                    emit!(self.build_call(eq_fn, &[lhs.into(), rhs.into()], "mono_eq_bytes"));
                let v = self.extract_call_value(call).into_int_value();
                emit!(self.build_int_compare(
                    IntPredicate::NE,
                    v,
                    self.i64_type().const_zero(),
                    "mono_eq_bytes_b"
                ))
            }
            ValueType::ByteArray => {
                let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::ByteArray);
                let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::ByteArray);
                let eq_fn = self.get_builtin(BuiltinFn::ByteArrayEq);
                let call = emit!(self.build_call(eq_fn, &[lhs.into(), rhs.into()], "mono_eq_ba"));
                let v = self.extract_call_value(call).into_int_value();
                emit!(self.build_int_compare(
                    IntPredicate::NE,
                    v,
                    self.i64_type().const_zero(),
                    "mono_eq_ba_b"
                ))
            }
            ValueType::Class(class_name) => {
                let class_ty = ValueType::Class(class_name.clone());
                let lhs = self.bitcast_from_i64(lhs_slot, &class_ty);
                let rhs = self.bitcast_from_i64(rhs_slot, &class_ty);
                let eq_name = format!("{}$__eq__", class_name);
                let eq_fn = self.get_or_declare_function(
                    &eq_name,
                    &[class_ty.clone(), class_ty],
                    Some(ValueType::Bool),
                );
                let call = emit!(self.build_call(eq_fn, &[lhs.into(), rhs.into()], "mono_eq_cls"));
                self.extract_call_value(call).into_int_value()
            }
            ValueType::List(inner) => {
                let list_ty = ValueType::List(Box::new((**inner).clone()));
                let lhs = self.bitcast_from_i64(lhs_slot, &list_ty);
                let rhs = self.bitcast_from_i64(rhs_slot, &list_ty);
                let eq_fn = self.ensure_mono_list_helper(MonoListOp::Eq, inner);
                let call = emit!(self.build_call(eq_fn, &[lhs.into(), rhs.into()], "mono_eq_list"));
                self.extract_call_value(call).into_int_value()
            }
            ValueType::Tuple(fields) => {
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
                        "mono_eq_tuple_lhs_ptr"
                    ));
                    let rhs_field_ptr = emit!(self.build_struct_gep(
                        tuple_struct,
                        rhs,
                        idx as u32,
                        "mono_eq_tuple_rhs_ptr"
                    ));
                    let lhs_field_val = emit!(self.build_load(
                        self.get_llvm_type(field_ty),
                        lhs_field_ptr,
                        "mono_eq_tuple_lhs"
                    ));
                    let rhs_field_val = emit!(self.build_load(
                        self.get_llvm_type(field_ty),
                        rhs_field_ptr,
                        "mono_eq_tuple_rhs"
                    ));
                    let lhs_field = self.bitcast_to_i64(lhs_field_val, field_ty);
                    let rhs_field = self.bitcast_to_i64(rhs_field_val, field_ty);
                    let field_eq = self.mono_elem_eq_from_slots(lhs_field, rhs_field, field_ty);
                    all_eq = emit!(self.build_and(all_eq, field_eq, "mono_eq_tuple_and"));
                }
                all_eq
            }
            _ => emit!(self.build_int_compare(IntPredicate::EQ, lhs_slot, rhs_slot, "mono_eq_ptr")),
        }
    }

    fn mono_elem_lt_from_slots(
        &mut self,
        lhs_slot: inkwell::values::IntValue<'ctx>,
        rhs_slot: inkwell::values::IntValue<'ctx>,
        elem_ty: &ValueType,
    ) -> inkwell::values::IntValue<'ctx> {
        match elem_ty {
            ValueType::Int | ValueType::Bool => {
                emit!(self.build_int_compare(IntPredicate::SLT, lhs_slot, rhs_slot, "mono_lt_i"))
            }
            ValueType::Float => {
                let lhs = emit!(self.build_bit_cast(lhs_slot, self.f64_type(), "lhs_f"))
                    .into_float_value();
                let rhs = emit!(self.build_bit_cast(rhs_slot, self.f64_type(), "rhs_f"))
                    .into_float_value();
                emit!(self.build_float_compare(FloatPredicate::OLT, lhs, rhs, "mono_lt_f"))
            }
            ValueType::Str => {
                let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Str);
                let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Str);
                let cmp_fn = self.get_builtin(BuiltinFn::StrCmp);
                let call =
                    emit!(self.build_call(cmp_fn, &[lhs.into(), rhs.into()], "mono_lt_str_cmp"));
                let v = self.extract_call_value(call).into_int_value();
                emit!(self.build_int_compare(
                    IntPredicate::SLT,
                    v,
                    self.i64_type().const_zero(),
                    "mono_lt_str"
                ))
            }
            ValueType::Bytes => {
                let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::Bytes);
                let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::Bytes);
                let cmp_fn = self.get_builtin(BuiltinFn::BytesCmp);
                let call =
                    emit!(self.build_call(cmp_fn, &[lhs.into(), rhs.into()], "mono_lt_bytes_cmp"));
                let v = self.extract_call_value(call).into_int_value();
                emit!(self.build_int_compare(
                    IntPredicate::SLT,
                    v,
                    self.i64_type().const_zero(),
                    "mono_lt_bytes"
                ))
            }
            ValueType::ByteArray => {
                let lhs = self.bitcast_from_i64(lhs_slot, &ValueType::ByteArray);
                let rhs = self.bitcast_from_i64(rhs_slot, &ValueType::ByteArray);
                let cmp_fn = self.get_builtin(BuiltinFn::ByteArrayCmp);
                let call =
                    emit!(self.build_call(cmp_fn, &[lhs.into(), rhs.into()], "mono_lt_ba_cmp"));
                let v = self.extract_call_value(call).into_int_value();
                emit!(self.build_int_compare(
                    IntPredicate::SLT,
                    v,
                    self.i64_type().const_zero(),
                    "mono_lt_ba"
                ))
            }
            ValueType::Class(class_name) => {
                let class_ty = ValueType::Class(class_name.clone());
                let lhs = self.bitcast_from_i64(lhs_slot, &class_ty);
                let rhs = self.bitcast_from_i64(rhs_slot, &class_ty);
                let lt_name = format!("{}$__lt__", class_name);
                let lt_fn = self.get_or_declare_function(
                    &lt_name,
                    &[class_ty.clone(), class_ty],
                    Some(ValueType::Bool),
                );
                let call = emit!(self.build_call(lt_fn, &[lhs.into(), rhs.into()], "mono_lt_cls"));
                self.extract_call_value(call).into_int_value()
            }
            ValueType::List(inner) => {
                let list_ty = ValueType::List(Box::new((**inner).clone()));
                let lhs = self.bitcast_from_i64(lhs_slot, &list_ty);
                let rhs = self.bitcast_from_i64(rhs_slot, &list_ty);
                let lt_fn = self.ensure_mono_list_helper(MonoListOp::Lt, inner);
                let call = emit!(self.build_call(lt_fn, &[lhs.into(), rhs.into()], "mono_lt_list"));
                self.extract_call_value(call).into_int_value()
            }
            _ => {
                emit!(self.build_int_compare(IntPredicate::SLT, lhs_slot, rhs_slot, "mono_lt_ptr"))
            }
        }
    }

    fn codegen_mono_list_eq_helper(&mut self, function: FunctionValue<'ctx>, elem_ty: &ValueType) {
        let a = function.get_nth_param(0).unwrap().into_pointer_value();
        let b = function.get_nth_param(1).unwrap().into_pointer_value();

        let idx = emit!(self.build_alloca(self.i64_type(), "idx"));
        emit!(self.build_store(idx, self.i64_type().const_zero()));

        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let list_get_fn = self.get_builtin(BuiltinFn::ListGet);

        let len_a_call = emit!(self.build_call(list_len_fn, &[a.into()], "len_a"));
        let len_b_call = emit!(self.build_call(list_len_fn, &[b.into()], "len_b"));
        let len_a = self.extract_call_value(len_a_call).into_int_value();
        let len_b = self.extract_call_value(len_b_call).into_int_value();

        let parent = function;
        let ret_false = self.context.append_basic_block(parent, "ret_false");
        let loop_head = self.context.append_basic_block(parent, "loop_head");
        let loop_body = self.context.append_basic_block(parent, "loop_body");
        let loop_inc = self.context.append_basic_block(parent, "loop_inc");
        let ret_true = self.context.append_basic_block(parent, "ret_true");

        let len_ne = emit!(self.build_int_compare(IntPredicate::NE, len_a, len_b, "len_ne"));
        emit!(self.build_conditional_branch(len_ne, ret_false, loop_head));

        self.builder.position_at_end(loop_head);
        let i = emit!(self.build_load(self.i64_type(), idx, "i")).into_int_value();
        let cond = emit!(self.build_int_compare(IntPredicate::SLT, i, len_a, "loop_cond"));
        emit!(self.build_conditional_branch(cond, loop_body, ret_true));

        self.builder.position_at_end(loop_body);
        let a_slot_call = emit!(self.build_call(list_get_fn, &[a.into(), i.into()], "a_slot"));
        let b_slot_call = emit!(self.build_call(list_get_fn, &[b.into(), i.into()], "b_slot"));
        let a_slot = self.extract_call_value(a_slot_call).into_int_value();
        let b_slot = self.extract_call_value(b_slot_call).into_int_value();
        let elem_eq = self.mono_elem_eq_from_slots(a_slot, b_slot, elem_ty);
        emit!(self.build_conditional_branch(elem_eq, loop_inc, ret_false));

        self.builder.position_at_end(loop_inc);
        let i_next = emit!(self.build_int_add(i, self.i64_type().const_int(1, false), "i_next"));
        emit!(self.build_store(idx, i_next));
        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(ret_false);
        let false_v = self.bool_type().const_zero();
        emit!(self.build_return(Some(&false_v)));

        self.builder.position_at_end(ret_true);
        let true_v = self.bool_type().const_int(1, false);
        emit!(self.build_return(Some(&true_v)));
    }

    fn codegen_mono_list_lt_helper(&mut self, function: FunctionValue<'ctx>, elem_ty: &ValueType) {
        let a = function.get_nth_param(0).unwrap().into_pointer_value();
        let b = function.get_nth_param(1).unwrap().into_pointer_value();

        let idx = emit!(self.build_alloca(self.i64_type(), "idx"));
        let lt_tmp = emit!(self.build_alloca(self.bool_type(), "lt_tmp"));
        emit!(self.build_store(idx, self.i64_type().const_zero()));

        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let list_get_fn = self.get_builtin(BuiltinFn::ListGet);

        let len_a_call = emit!(self.build_call(list_len_fn, &[a.into()], "len_a"));
        let len_b_call = emit!(self.build_call(list_len_fn, &[b.into()], "len_b"));
        let len_a = self.extract_call_value(len_a_call).into_int_value();
        let len_b = self.extract_call_value(len_b_call).into_int_value();
        let a_lt_b = emit!(self.build_int_compare(IntPredicate::SLT, len_a, len_b, "a_lt_b"));
        let min_len = emit!(self.build_select(a_lt_b, len_a, len_b, "min_len")).into_int_value();

        let parent = function;
        let loop_head = self.context.append_basic_block(parent, "loop_head");
        let loop_body = self.context.append_basic_block(parent, "loop_body");
        let loop_inc = self.context.append_basic_block(parent, "loop_inc");
        let diff_cmp = self.context.append_basic_block(parent, "diff_cmp");
        let ret_len_cmp = self.context.append_basic_block(parent, "ret_len_cmp");

        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(loop_head);
        let i = emit!(self.build_load(self.i64_type(), idx, "i")).into_int_value();
        let cond = emit!(self.build_int_compare(IntPredicate::SLT, i, min_len, "loop_cond"));
        emit!(self.build_conditional_branch(cond, loop_body, ret_len_cmp));

        self.builder.position_at_end(loop_body);
        let a_slot_call = emit!(self.build_call(list_get_fn, &[a.into(), i.into()], "a_slot"));
        let b_slot_call = emit!(self.build_call(list_get_fn, &[b.into(), i.into()], "b_slot"));
        let a_slot = self.extract_call_value(a_slot_call).into_int_value();
        let b_slot = self.extract_call_value(b_slot_call).into_int_value();
        let a_lt_b = self.mono_elem_lt_from_slots(a_slot, b_slot, elem_ty);
        emit!(self.build_store(lt_tmp, a_lt_b));
        let b_lt_a = self.mono_elem_lt_from_slots(b_slot, a_slot, elem_ty);
        let differs = emit!(self.build_or(a_lt_b, b_lt_a, "elem_differs"));
        emit!(self.build_conditional_branch(differs, diff_cmp, loop_inc));

        self.builder.position_at_end(loop_inc);
        let i_next = emit!(self.build_int_add(i, self.i64_type().const_int(1, false), "i_next"));
        emit!(self.build_store(idx, i_next));
        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(diff_cmp);
        let elem_lt = emit!(self.build_load(self.bool_type(), lt_tmp, "elem_lt")).into_int_value();
        emit!(self.build_return(Some(&elem_lt)));

        self.builder.position_at_end(ret_len_cmp);
        let len_cmp = emit!(self.build_int_compare(IntPredicate::SLT, len_a, len_b, "len_cmp"));
        emit!(self.build_return(Some(&len_cmp)));
    }

    fn codegen_mono_list_contains_helper(
        &mut self,
        function: FunctionValue<'ctx>,
        elem_ty: &ValueType,
    ) {
        let list = function.get_nth_param(0).unwrap().into_pointer_value();
        let target = function.get_nth_param(1).unwrap();
        let target_slot = self.bitcast_to_i64(target, elem_ty);

        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let list_get_fn = self.get_builtin(BuiltinFn::ListGet);
        let len_call = emit!(self.build_call(list_len_fn, &[list.into()], "len"));
        let len = self.extract_call_value(len_call).into_int_value();

        let idx = emit!(self.build_alloca(self.i64_type(), "idx"));
        emit!(self.build_store(idx, self.i64_type().const_zero()));

        let parent = function;
        let loop_head = self.context.append_basic_block(parent, "loop_head");
        let loop_body = self.context.append_basic_block(parent, "loop_body");
        let loop_inc = self.context.append_basic_block(parent, "loop_inc");
        let ret_true = self.context.append_basic_block(parent, "ret_true");
        let ret_false = self.context.append_basic_block(parent, "ret_false");

        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(loop_head);
        let i = emit!(self.build_load(self.i64_type(), idx, "i")).into_int_value();
        let cond = emit!(self.build_int_compare(IntPredicate::SLT, i, len, "loop_cond"));
        emit!(self.build_conditional_branch(cond, loop_body, ret_false));

        self.builder.position_at_end(loop_body);
        let slot_call = emit!(self.build_call(list_get_fn, &[list.into(), i.into()], "slot"));
        let slot = self.extract_call_value(slot_call).into_int_value();
        let elem_eq = self.mono_elem_eq_from_slots(slot, target_slot, elem_ty);
        emit!(self.build_conditional_branch(elem_eq, ret_true, loop_inc));

        self.builder.position_at_end(loop_inc);
        let i_next = emit!(self.build_int_add(i, self.i64_type().const_int(1, false), "i_next"));
        emit!(self.build_store(idx, i_next));
        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(ret_true);
        let true_v = self.bool_type().const_int(1, false);
        emit!(self.build_return(Some(&true_v)));

        self.builder.position_at_end(ret_false);
        let false_v = self.bool_type().const_zero();
        emit!(self.build_return(Some(&false_v)));
    }

    fn codegen_mono_list_index_helper(
        &mut self,
        function: FunctionValue<'ctx>,
        elem_ty: &ValueType,
    ) {
        let list = function.get_nth_param(0).unwrap().into_pointer_value();
        let target = function.get_nth_param(1).unwrap();
        let target_slot = self.bitcast_to_i64(target, elem_ty);

        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let list_get_fn = self.get_builtin(BuiltinFn::ListGet);
        let len_call = emit!(self.build_call(list_len_fn, &[list.into()], "len"));
        let len = self.extract_call_value(len_call).into_int_value();

        let idx = emit!(self.build_alloca(self.i64_type(), "idx"));
        emit!(self.build_store(idx, self.i64_type().const_zero()));

        let parent = function;
        let loop_head = self.context.append_basic_block(parent, "loop_head");
        let loop_body = self.context.append_basic_block(parent, "loop_body");
        let ret_found = self.context.append_basic_block(parent, "ret_found");
        let do_inc = self.context.append_basic_block(parent, "do_inc");
        let not_found = self.context.append_basic_block(parent, "not_found");

        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(loop_head);
        let i = emit!(self.build_load(self.i64_type(), idx, "i")).into_int_value();
        let cond = emit!(self.build_int_compare(IntPredicate::SLT, i, len, "loop_cond"));
        emit!(self.build_conditional_branch(cond, loop_body, not_found));

        self.builder.position_at_end(loop_body);
        let slot_call = emit!(self.build_call(list_get_fn, &[list.into(), i.into()], "slot"));
        let slot = self.extract_call_value(slot_call).into_int_value();
        let elem_eq = self.mono_elem_eq_from_slots(slot, target_slot, elem_ty);
        emit!(self.build_conditional_branch(elem_eq, ret_found, do_inc));

        self.builder.position_at_end(ret_found);
        emit!(self.build_return(Some(&i)));

        self.builder.position_at_end(do_inc);
        let i_next = emit!(self.build_int_add(i, self.i64_type().const_int(1, false), "i_next"));
        emit!(self.build_store(idx, i_next));
        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(not_found);
        let tag = self.i64_type().const_int(3, false).into();
        let msg = self.codegen_str_literal("x not in list");
        self.emit_raise(tag, msg);
    }

    fn codegen_mono_list_count_helper(
        &mut self,
        function: FunctionValue<'ctx>,
        elem_ty: &ValueType,
    ) {
        let list = function.get_nth_param(0).unwrap().into_pointer_value();
        let target = function.get_nth_param(1).unwrap();
        let target_slot = self.bitcast_to_i64(target, elem_ty);

        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let list_get_fn = self.get_builtin(BuiltinFn::ListGet);
        let len_call = emit!(self.build_call(list_len_fn, &[list.into()], "len"));
        let len = self.extract_call_value(len_call).into_int_value();

        let idx = emit!(self.build_alloca(self.i64_type(), "idx"));
        let cnt = emit!(self.build_alloca(self.i64_type(), "cnt"));
        emit!(self.build_store(idx, self.i64_type().const_zero()));
        emit!(self.build_store(cnt, self.i64_type().const_zero()));

        let parent = function;
        let loop_head = self.context.append_basic_block(parent, "loop_head");
        let loop_body = self.context.append_basic_block(parent, "loop_body");
        let maybe_inc = self.context.append_basic_block(parent, "maybe_inc");
        let loop_inc = self.context.append_basic_block(parent, "loop_inc");
        let done = self.context.append_basic_block(parent, "done");

        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(loop_head);
        let i = emit!(self.build_load(self.i64_type(), idx, "i")).into_int_value();
        let cond = emit!(self.build_int_compare(IntPredicate::SLT, i, len, "loop_cond"));
        emit!(self.build_conditional_branch(cond, loop_body, done));

        self.builder.position_at_end(loop_body);
        let slot_call = emit!(self.build_call(list_get_fn, &[list.into(), i.into()], "slot"));
        let slot = self.extract_call_value(slot_call).into_int_value();
        let elem_eq = self.mono_elem_eq_from_slots(slot, target_slot, elem_ty);
        emit!(self.build_conditional_branch(elem_eq, maybe_inc, loop_inc));

        self.builder.position_at_end(maybe_inc);
        let c = emit!(self.build_load(self.i64_type(), cnt, "c")).into_int_value();
        let c_next = emit!(self.build_int_add(c, self.i64_type().const_int(1, false), "c_next"));
        emit!(self.build_store(cnt, c_next));
        emit!(self.build_unconditional_branch(loop_inc));

        self.builder.position_at_end(loop_inc);
        let i_next = emit!(self.build_int_add(i, self.i64_type().const_int(1, false), "i_next"));
        emit!(self.build_store(idx, i_next));
        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(done);
        let out = emit!(self.build_load(self.i64_type(), cnt, "out")).into_int_value();
        emit!(self.build_return(Some(&out)));
    }

    fn codegen_mono_list_remove_helper(
        &mut self,
        function: FunctionValue<'ctx>,
        elem_ty: &ValueType,
    ) {
        let list = function.get_nth_param(0).unwrap().into_pointer_value();
        let value = function.get_nth_param(1).unwrap();

        let index_fn = self.ensure_mono_list_helper(MonoListOp::Index, elem_ty);
        let idx_call = emit!(self.build_call(index_fn, &[list.into(), value.into()], "idx_call"));
        let idx_val = self.extract_call_value(idx_call).into_int_value();

        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let list_get_fn = self.get_builtin(BuiltinFn::ListGet);
        let list_pop_fn = self.get_builtin(BuiltinFn::ListPop);
        let list_set_fn = self.get_runtime_fn(RuntimeFn::ListSet);

        let len_call = emit!(self.build_call(list_len_fn, &[list.into()], "len"));
        let len = self.extract_call_value(len_call).into_int_value();
        let last = emit!(self.build_int_sub(len, self.i64_type().const_int(1, false), "last"));

        let i_alloca = emit!(self.build_alloca(self.i64_type(), "i"));
        emit!(self.build_store(i_alloca, idx_val));

        let parent = function;
        let loop_head = self.context.append_basic_block(parent, "loop_head");
        let loop_body = self.context.append_basic_block(parent, "loop_body");
        let done = self.context.append_basic_block(parent, "done");

        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(loop_head);
        let i = emit!(self.build_load(self.i64_type(), i_alloca, "i")).into_int_value();
        let cond = emit!(self.build_int_compare(IntPredicate::SLT, i, last, "cond"));
        emit!(self.build_conditional_branch(cond, loop_body, done));

        self.builder.position_at_end(loop_body);
        let next_i = emit!(self.build_int_add(i, self.i64_type().const_int(1, false), "next_i"));
        let next_slot_call =
            emit!(self.build_call(list_get_fn, &[list.into(), next_i.into()], "next_slot"));
        let next_slot = self.extract_call_value(next_slot_call).into_int_value();
        emit!(self.build_call(
            list_set_fn,
            &[list.into(), i.into(), next_slot.into()],
            "set_slot"
        ));
        emit!(self.build_store(i_alloca, next_i));
        emit!(self.build_unconditional_branch(loop_head));

        self.builder.position_at_end(done);
        emit!(self.build_call(list_pop_fn, &[list.into()], "pop_tail"));
        emit!(self.build_return(None));
    }

    fn codegen_mono_list_sort_helper(
        &mut self,
        function: FunctionValue<'ctx>,
        elem_ty: &ValueType,
    ) {
        let list = function.get_nth_param(0).unwrap().into_pointer_value();
        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let list_get_fn = self.get_builtin(BuiltinFn::ListGet);
        let list_set_fn = self.get_runtime_fn(RuntimeFn::ListSet);

        let len_call = emit!(self.build_call(list_len_fn, &[list.into()], "len"));
        let len = self.extract_call_value(len_call).into_int_value();

        let i_alloca = emit!(self.build_alloca(self.i64_type(), "i"));
        emit!(self.build_store(i_alloca, self.i64_type().const_int(1, false)));

        let parent = function;
        let outer_head = self.context.append_basic_block(parent, "outer_head");
        let outer_body = self.context.append_basic_block(parent, "outer_body");
        let inner_head = self.context.append_basic_block(parent, "inner_head");
        let inner_cmp = self.context.append_basic_block(parent, "inner_cmp");
        let inner_shift = self.context.append_basic_block(parent, "inner_shift");
        let inner_insert = self.context.append_basic_block(parent, "inner_insert");
        let done = self.context.append_basic_block(parent, "done");

        emit!(self.build_unconditional_branch(outer_head));

        self.builder.position_at_end(outer_head);
        let i = emit!(self.build_load(self.i64_type(), i_alloca, "i")).into_int_value();
        let outer_cond = emit!(self.build_int_compare(IntPredicate::SLT, i, len, "outer_cond"));
        emit!(self.build_conditional_branch(outer_cond, outer_body, done));

        self.builder.position_at_end(outer_body);
        let key_call = emit!(self.build_call(list_get_fn, &[list.into(), i.into()], "key"));
        let key = self.extract_call_value(key_call).into_int_value();
        let j_alloca = emit!(self.build_alloca(self.i64_type(), "j"));
        let j_init = emit!(self.build_int_sub(i, self.i64_type().const_int(1, false), "j_init"));
        emit!(self.build_store(j_alloca, j_init));
        emit!(self.build_unconditional_branch(inner_head));

        self.builder.position_at_end(inner_head);
        let j = emit!(self.build_load(self.i64_type(), j_alloca, "j")).into_int_value();
        let j_nonneg = emit!(self.build_int_compare(
            IntPredicate::SGE,
            j,
            self.i64_type().const_zero(),
            "j_nonneg"
        ));
        emit!(self.build_conditional_branch(j_nonneg, inner_cmp, inner_insert));

        self.builder.position_at_end(inner_cmp);
        let cur_call = emit!(self.build_call(list_get_fn, &[list.into(), j.into()], "cur"));
        let cur = self.extract_call_value(cur_call).into_int_value();
        let key_lt_cur = self.mono_elem_lt_from_slots(key, cur, elem_ty);
        emit!(self.build_conditional_branch(key_lt_cur, inner_shift, inner_insert));

        self.builder.position_at_end(inner_shift);
        let j_plus_1 =
            emit!(self.build_int_add(j, self.i64_type().const_int(1, false), "j_plus_1"));
        emit!(self.build_call(
            list_set_fn,
            &[list.into(), j_plus_1.into(), cur.into()],
            "shift"
        ));
        let j_dec = emit!(self.build_int_sub(j, self.i64_type().const_int(1, false), "j_dec"));
        emit!(self.build_store(j_alloca, j_dec));
        emit!(self.build_unconditional_branch(inner_head));

        self.builder.position_at_end(inner_insert);
        let j_curr = emit!(self.build_load(self.i64_type(), j_alloca, "j_curr")).into_int_value();
        let insert_pos =
            emit!(self.build_int_add(j_curr, self.i64_type().const_int(1, false), "insert_pos"));
        emit!(self.build_call(
            list_set_fn,
            &[list.into(), insert_pos.into(), key.into()],
            "insert"
        ));
        let i_next = emit!(self.build_int_add(i, self.i64_type().const_int(1, false), "i_next"));
        emit!(self.build_store(i_alloca, i_next));
        emit!(self.build_unconditional_branch(outer_head));

        self.builder.position_at_end(done);
        emit!(self.build_return(None));
    }

    fn codegen_mono_list_sorted_helper(
        &mut self,
        function: FunctionValue<'ctx>,
        elem_ty: &ValueType,
    ) {
        let list = function.get_nth_param(0).unwrap().into_pointer_value();
        let list_copy_fn = self.get_builtin(BuiltinFn::ListCopy);
        let copy_call = emit!(self.build_call(list_copy_fn, &[list.into()], "copy"));
        let copy = self.extract_call_value(copy_call).into_pointer_value();

        let sort_fn = self.ensure_mono_list_helper(MonoListOp::Sort, elem_ty);
        emit!(self.build_call(sort_fn, &[copy.into()], "sort_copy"));
        emit!(self.build_return(Some(&copy)));
    }

    fn codegen_mono_list_helper_body(
        &mut self,
        function: FunctionValue<'ctx>,
        op: MonoListOp,
        elem_ty: &ValueType,
    ) {
        match op {
            MonoListOp::Eq => self.codegen_mono_list_eq_helper(function, elem_ty),
            MonoListOp::Lt => self.codegen_mono_list_lt_helper(function, elem_ty),
            MonoListOp::Contains => self.codegen_mono_list_contains_helper(function, elem_ty),
            MonoListOp::Index => self.codegen_mono_list_index_helper(function, elem_ty),
            MonoListOp::Count => self.codegen_mono_list_count_helper(function, elem_ty),
            MonoListOp::Remove => self.codegen_mono_list_remove_helper(function, elem_ty),
            MonoListOp::Sort => self.codegen_mono_list_sort_helper(function, elem_ty),
            MonoListOp::Sorted => self.codegen_mono_list_sorted_helper(function, elem_ty),
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
        if matches!(func, BuiltinFn::ListEqGeneric) {
            let elem_ty = Self::list_elem_type_from_first_arg(args)
                .expect("ICE: ListEqGeneric expected first arg of type list[...]");
            let helper_fn = self.ensure_mono_list_helper(MonoListOp::Eq, elem_ty);
            let arg_values = self.codegen_call_args(args);
            let call =
                emit!(self.build_call(helper_fn, &Self::to_meta_args(&arg_values), "mono_list_eq"));
            return Some(self.extract_call_value(call));
        }

        if matches!(func, BuiltinFn::ListSortAny | BuiltinFn::SortedAny) {
            let elem_ty = Self::list_elem_type_from_first_arg(args)
                .expect("ICE: list sort/sorted expected first arg of type list[...]");
            if let Some(fast_builtin) = Self::sort_fastpath_builtin(func, elem_ty) {
                let function = self.get_builtin(fast_builtin);
                let arg_values = self.codegen_call_args(args);
                let call = emit!(self.build_call(
                    function,
                    &Self::to_meta_args(&arg_values),
                    "builtin_call"
                ));
                return result_ty
                    .map(|ty| self.bool_from_runtime_abi(self.extract_call_value(call), ty));
            }
            let op = match func {
                BuiltinFn::ListSortAny => MonoListOp::Sort,
                BuiltinFn::SortedAny => MonoListOp::Sorted,
                _ => unreachable!(),
            };
            let helper_fn = self.ensure_mono_list_helper(op, elem_ty);
            let arg_values = self.codegen_call_args(args);
            let call = emit!(self.build_call(
                helper_fn,
                &Self::to_meta_args(&arg_values),
                "mono_list_sort_like"
            ));
            return result_ty.map(|_| self.extract_call_value(call));
        }

        if matches!(
            func,
            BuiltinFn::ListContains
                | BuiltinFn::ListIndex
                | BuiltinFn::ListCount
                | BuiltinFn::ListRemove
        ) {
            let elem_ty = Self::list_elem_type_from_first_arg(args)
                .expect("ICE: list method expected first arg of type list[...]");
            if Self::requires_mono_list_eq_ops(elem_ty) {
                let op = match func {
                    BuiltinFn::ListContains => MonoListOp::Contains,
                    BuiltinFn::ListIndex => MonoListOp::Index,
                    BuiltinFn::ListCount => MonoListOp::Count,
                    BuiltinFn::ListRemove => MonoListOp::Remove,
                    _ => unreachable!(),
                };
                let helper_fn = self.ensure_mono_list_helper(op, elem_ty);
                let arg_values = self.codegen_call_args(args);
                let call = emit!(self.build_call(
                    helper_fn,
                    &Self::to_meta_args(&arg_values),
                    "mono_list_eq_like"
                ));
                return result_ty.map(|_| self.extract_call_value(call));
            }
        }

        let function = self.get_builtin(func);

        // DictGet/DictPop need both:
        // - key (arg1) bitcasted to i64
        // - returned slot bitcasted from i64 to the value type
        if matches!(func, BuiltinFn::DictGet | BuiltinFn::DictPop) {
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
            BuiltinFn::DictContains | BuiltinFn::DictGet | BuiltinFn::DictPop
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

        // DictSet bitcasts key (arg1) and value (arg2).
        if matches!(func, BuiltinFn::DictSet) {
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

        // Set ops with element arg in position 1.
        if matches!(
            func,
            BuiltinFn::SetContains
                | BuiltinFn::SetAdd
                | BuiltinFn::SetRemove
                | BuiltinFn::SetDiscard
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
