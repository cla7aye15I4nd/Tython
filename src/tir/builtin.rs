use super::ValueType;

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

            pub fn param_types(&self) -> Vec<ValueType> {
                match self {
                    $(Self::$variant => vec![$($param),*],)*
                }
            }

            pub fn return_type(&self) -> Option<ValueType> {
                match self {
                    $(Self::$variant => $ret,)*
                }
            }
        }
    };
}

define_builtins! {
    PrintInt      => "__tython_print_int",      params: [ValueType::Int],                          ret: None;
    PrintFloat    => "__tython_print_float",    params: [ValueType::Float],                        ret: None;
    PrintBool     => "__tython_print_bool",     params: [ValueType::Bool],                         ret: None;
    PrintStr      => "__tython_print_str",      params: [ValueType::Str],                          ret: None;
    PrintBytes    => "__tython_print_bytes",    params: [ValueType::Bytes],                        ret: None;
    PrintByteArray => "__tython_print_bytearray", params: [ValueType::ByteArray],                  ret: None;
    PrintSpace    => "__tython_print_space",    params: [],                                        ret: None;
    PrintNewline  => "__tython_print_newline",  params: [],                                        ret: None;
    Assert        => "__tython_assert",         params: [ValueType::Bool],                         ret: None;
    PowInt        => "__tython_pow_int",        params: [ValueType::Int, ValueType::Int],           ret: Some(ValueType::Int);
    AbsInt        => "__tython_abs_int",        params: [ValueType::Int],                          ret: Some(ValueType::Int);
    AbsFloat      => "__tython_abs_float",      params: [ValueType::Float],                        ret: Some(ValueType::Float);
    MinInt        => "__tython_min_int",        params: [ValueType::Int, ValueType::Int],           ret: Some(ValueType::Int);
    MinFloat      => "__tython_min_float",      params: [ValueType::Float, ValueType::Float],       ret: Some(ValueType::Float);
    MaxInt        => "__tython_max_int",        params: [ValueType::Int, ValueType::Int],           ret: Some(ValueType::Int);
    MaxFloat      => "__tython_max_float",      params: [ValueType::Float, ValueType::Float],       ret: Some(ValueType::Float);
    RoundFloat    => "__tython_round_float",    params: [ValueType::Float],                        ret: Some(ValueType::Int);

    // str builtins
    StrConcat     => "__tython_str_concat",     params: [ValueType::Str, ValueType::Str],           ret: Some(ValueType::Str);
    StrRepeat     => "__tython_str_repeat",     params: [ValueType::Str, ValueType::Int],           ret: Some(ValueType::Str);
    StrLen        => "__tython_str_len",        params: [ValueType::Str],                          ret: Some(ValueType::Int);
    StrCmp        => "__tython_str_cmp",        params: [ValueType::Str, ValueType::Str],           ret: Some(ValueType::Int);
    StrEq         => "__tython_str_eq",         params: [ValueType::Str, ValueType::Str],           ret: Some(ValueType::Int);
    StrFromInt    => "__tython_str_from_int",   params: [ValueType::Int],                          ret: Some(ValueType::Str);
    StrFromFloat  => "__tython_str_from_float", params: [ValueType::Float],                        ret: Some(ValueType::Str);
    StrFromBool   => "__tython_str_from_bool",  params: [ValueType::Bool],                         ret: Some(ValueType::Str);

    // bytes builtins
    BytesConcat   => "__tython_bytes_concat",   params: [ValueType::Bytes, ValueType::Bytes],       ret: Some(ValueType::Bytes);
    BytesRepeat   => "__tython_bytes_repeat",   params: [ValueType::Bytes, ValueType::Int],         ret: Some(ValueType::Bytes);
    BytesLen      => "__tython_bytes_len",      params: [ValueType::Bytes],                        ret: Some(ValueType::Int);
    BytesCmp      => "__tython_bytes_cmp",      params: [ValueType::Bytes, ValueType::Bytes],       ret: Some(ValueType::Int);
    BytesEq       => "__tython_bytes_eq",       params: [ValueType::Bytes, ValueType::Bytes],       ret: Some(ValueType::Int);
    BytesFromInt  => "__tython_bytes_from_int", params: [ValueType::Int],                          ret: Some(ValueType::Bytes);
    BytesFromStr  => "__tython_bytes_from_str", params: [ValueType::Str],                          ret: Some(ValueType::Bytes);

    // bytearray builtins
    ByteArrayConcat    => "__tython_bytearray_concat",     params: [ValueType::ByteArray, ValueType::ByteArray], ret: Some(ValueType::ByteArray);
    ByteArrayRepeat    => "__tython_bytearray_repeat",     params: [ValueType::ByteArray, ValueType::Int],       ret: Some(ValueType::ByteArray);
    ByteArrayLen       => "__tython_bytearray_len",        params: [ValueType::ByteArray],                       ret: Some(ValueType::Int);
    ByteArrayCmp       => "__tython_bytearray_cmp",        params: [ValueType::ByteArray, ValueType::ByteArray], ret: Some(ValueType::Int);
    ByteArrayEq        => "__tython_bytearray_eq",         params: [ValueType::ByteArray, ValueType::ByteArray], ret: Some(ValueType::Int);
    ByteArrayAppend    => "__tython_bytearray_append",     params: [ValueType::ByteArray, ValueType::Int],       ret: None;
    ByteArrayExtend    => "__tython_bytearray_extend",     params: [ValueType::ByteArray, ValueType::Bytes],     ret: None;
    ByteArrayClear     => "__tython_bytearray_clear",      params: [ValueType::ByteArray],                       ret: None;
    ByteArrayFromInt   => "__tython_bytearray_from_int",   params: [ValueType::Int],                             ret: Some(ValueType::ByteArray);
    ByteArrayFromBytes => "__tython_bytearray_from_bytes", params: [ValueType::Bytes],                           ret: Some(ValueType::ByteArray);
    ByteArrayEmpty     => "__tython_bytearray_empty",      params: [],                                           ret: Some(ValueType::ByteArray);
}

pub fn resolve_print(arg_ty: &ValueType) -> Option<BuiltinFn> {
    match arg_ty {
        ValueType::Float => Some(BuiltinFn::PrintFloat),
        ValueType::Bool => Some(BuiltinFn::PrintBool),
        ValueType::Int => Some(BuiltinFn::PrintInt),
        ValueType::Str => Some(BuiltinFn::PrintStr),
        ValueType::Bytes => Some(BuiltinFn::PrintBytes),
        ValueType::ByteArray => Some(BuiltinFn::PrintByteArray),
        ValueType::Class(_) => None,
    }
}
