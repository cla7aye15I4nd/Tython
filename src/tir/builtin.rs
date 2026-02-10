use crate::ast::Type;

macro_rules! define_builtins {
    ($($variant:ident => $symbol:literal, params: [$($param:expr),*], ret: $ret:expr);* $(;)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum BuiltinFn {
            $($variant,)*
        }

        impl BuiltinFn {
            pub fn symbol(&self) -> &'static str {
                match self {
                    $(Self::$variant => $symbol,)*
                }
            }

            pub fn param_types(&self) -> Vec<Type> {
                match self {
                    $(Self::$variant => vec![$($param),*],)*
                }
            }

            pub fn return_type(&self) -> Type {
                match self {
                    $(Self::$variant => $ret,)*
                }
            }
        }
    };
}

define_builtins! {
    PrintInt      => "__tython_print_int",      params: [Type::Int],   ret: Type::Unit;
    PrintFloat    => "__tython_print_float",    params: [Type::Float], ret: Type::Unit;
    PrintBool     => "__tython_print_bool",     params: [Type::Bool],  ret: Type::Unit;
    PrintSpace    => "__tython_print_space",    params: [],            ret: Type::Unit;
    PrintNewline  => "__tython_print_newline",  params: [],            ret: Type::Unit;
    Assert        => "__tython_assert",         params: [Type::Bool],  ret: Type::Unit;
}

pub fn resolve_print(arg_ty: &Type) -> BuiltinFn {
    match arg_ty {
        Type::Float => BuiltinFn::PrintFloat,
        Type::Bool => BuiltinFn::PrintBool,
        _ => BuiltinFn::PrintInt,
    }
}
