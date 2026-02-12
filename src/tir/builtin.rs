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
    ByteArrayInsert    => "__tython_bytearray_insert",     params: [ValueType::ByteArray, ValueType::Int, ValueType::Int], ret: None;
    ByteArrayRemove    => "__tython_bytearray_remove",     params: [ValueType::ByteArray, ValueType::Int],       ret: None;
    ByteArrayReverse   => "__tython_bytearray_reverse",    params: [ValueType::ByteArray],                       ret: None;

    // list builtins (all List(...) map to ptr in LLVM; inner type is a sentinel)
    ListEmpty          => "__tython_list_empty",          params: [],                                                                                ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListLen            => "__tython_list_len",            params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: Some(ValueType::Int);
    ListGet            => "__tython_list_get",            params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: Some(ValueType::Int);
    ListAppend         => "__tython_list_append",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: None;
    ListPop            => "__tython_list_pop",            params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: Some(ValueType::Int);
    ListClear          => "__tython_list_clear",          params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: None;
    PrintListInt       => "__tython_print_list_int",      params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: None;
    PrintListFloat     => "__tython_print_list_float",    params: [ValueType::List(Box::new(ValueType::Float))],                                     ret: None;
    PrintListBool      => "__tython_print_list_bool",     params: [ValueType::List(Box::new(ValueType::Bool))],                                      ret: None;
    PrintListStr       => "__tython_print_list_str",      params: [ValueType::List(Box::new(ValueType::Str))],                                       ret: None;
    PrintListBytes     => "__tython_print_list_bytes",    params: [ValueType::List(Box::new(ValueType::Bytes))],                                     ret: None;
    PrintListByteArray => "__tython_print_list_bytearray", params: [ValueType::List(Box::new(ValueType::ByteArray))],                                ret: None;

    // list equality
    ListEqShallow      => "__tython_list_eq_shallow",     params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    ListEqDeep         => "__tython_list_eq_deep",        params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);

    // list containment
    ListContains       => "__tython_list_contains",       params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);

    // str containment
    StrContains        => "__tython_str_contains",        params: [ValueType::Str, ValueType::Str], ret: Some(ValueType::Bool);

    // list methods
    ListInsert         => "__tython_list_insert",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    ListRemove         => "__tython_list_remove",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    ListIndex          => "__tython_list_index",          params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    ListCount          => "__tython_list_count",          params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    ListReverse        => "__tython_list_reverse",        params: [ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListSortInt        => "__tython_list_sort_int",       params: [ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListSortFloat      => "__tython_list_sort_float",     params: [ValueType::List(Box::new(ValueType::Float))], ret: None;
    SortedInt          => "__tython_sorted_int",          params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    SortedFloat        => "__tython_sorted_float",        params: [ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::List(Box::new(ValueType::Float)));
    ListExtend         => "__tython_list_extend",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListCopy           => "__tython_list_copy",           params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));

    // aggregate builtins
    SumInt             => "__tython_sum_int",             params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    SumFloat           => "__tython_sum_float",           params: [ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::Float);
    SumIntStart        => "__tython_sum_int_start",       params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    SumFloatStart      => "__tython_sum_float_start",     params: [ValueType::List(Box::new(ValueType::Float)), ValueType::Float], ret: Some(ValueType::Float);
    AllList            => "__tython_all_list",            params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    AnyList            => "__tython_any_list",            params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
}
