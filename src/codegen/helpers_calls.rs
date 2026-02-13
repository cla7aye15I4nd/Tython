use inkwell::module::Linkage;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::BasicType;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;

use crate::tir::builtin::BuiltinFn;
use crate::tir::{TirExpr, ValueType};

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
