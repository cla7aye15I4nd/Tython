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
    OpenReadAll   => "__tython_open_read_all",  params: [ValueType::Str],                          ret: Some(ValueType::Str);
    PowInt        => "__tython_pow_int",        params: [ValueType::Int, ValueType::Int],           ret: Some(ValueType::Int);
    AbsInt        => "__tython_abs_int",        params: [ValueType::Int],                          ret: Some(ValueType::Int);
    AbsFloat      => "__tython_abs_float",      params: [ValueType::Float],                        ret: Some(ValueType::Float);
    MinInt        => "__tython_min_int",        params: [ValueType::Int, ValueType::Int],           ret: Some(ValueType::Int);
    MinFloat      => "__tython_min_float",      params: [ValueType::Float, ValueType::Float],       ret: Some(ValueType::Float);
    MaxInt        => "__tython_max_int",        params: [ValueType::Int, ValueType::Int],           ret: Some(ValueType::Int);
    MaxFloat      => "__tython_max_float",      params: [ValueType::Float, ValueType::Float],       ret: Some(ValueType::Float);
    MaxListInt    => "__tython_max_list_int",   params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    MaxListFloat  => "__tython_max_list_float", params: [ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::Float);
    RoundFloat    => "__tython_round_float",    params: [ValueType::Float],                        ret: Some(ValueType::Int);
    MathLog       => "__tython_math_log",       params: [ValueType::Float],                        ret: Some(ValueType::Float);
    MathExp       => "__tython_math_exp",       params: [ValueType::Float],                        ret: Some(ValueType::Float);
    RandomSeed    => "__tython_random_seed",    params: [ValueType::Int],                          ret: None;
    RandomGauss   => "__tython_random_gauss",   params: [ValueType::Float, ValueType::Float],      ret: Some(ValueType::Float);
    RandomShuffle => "__tython_random_shuffle", params: [ValueType::List(Box::new(ValueType::Int))], ret: None;
    RandomChoicesInt => "__tython_random_choices_int", params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    Range1        => "__tython_range_1",        params: [ValueType::Int],                          ret: Some(ValueType::List(Box::new(ValueType::Int)));
    Range2        => "__tython_range_2",        params: [ValueType::Int, ValueType::Int],          ret: Some(ValueType::List(Box::new(ValueType::Int)));
    Range3        => "__tython_range_3",        params: [ValueType::Int, ValueType::Int, ValueType::Int], ret: Some(ValueType::List(Box::new(ValueType::Int)));

    // str builtins
    StrConcat     => "__tython_str_concat",     params: [ValueType::Str, ValueType::Str],           ret: Some(ValueType::Str);
    StrRepeat     => "__tython_str_repeat",     params: [ValueType::Str, ValueType::Int],           ret: Some(ValueType::Str);
    StrLen        => "__tython_str_len",        params: [ValueType::Str],                          ret: Some(ValueType::Int);
    StrCmp        => "__tython_str_cmp",        params: [ValueType::Str, ValueType::Str],           ret: Some(ValueType::Int);
    StrEq         => "__tython_str_eq",         params: [ValueType::Str, ValueType::Str],           ret: Some(ValueType::Int);
    StrGetChar    => "__tython_str_get_char",   params: [ValueType::Str, ValueType::Int],            ret: Some(ValueType::Str);
    StrFromInt    => "__tython_str_from_int",   params: [ValueType::Int],                          ret: Some(ValueType::Str);
    StrFromFloat  => "__tython_str_from_float", params: [ValueType::Float],                        ret: Some(ValueType::Str);
    StrFromBool   => "__tython_str_from_bool",  params: [ValueType::Bool],                         ret: Some(ValueType::Str);
    StrFromBytes  => "__tython_str_from_bytes", params: [ValueType::Bytes],                        ret: Some(ValueType::Str);
    StrFromByteArray => "__tython_str_from_bytearray", params: [ValueType::ByteArray],             ret: Some(ValueType::Str);
    ReprStr       => "__tython_repr_str",       params: [ValueType::Str],                          ret: Some(ValueType::Str);
    StrRead       => "__tython_str_read",       params: [ValueType::Str],                          ret: Some(ValueType::Str);
    StrStrip      => "__tython_str_strip",      params: [ValueType::Str],                          ret: Some(ValueType::Str);
    StrSplit      => "__tython_str_split",      params: [ValueType::Str, ValueType::Str],          ret: Some(ValueType::List(Box::new(ValueType::Str)));
    StrJoin       => "__tython_str_join",       params: [ValueType::Str, ValueType::List(Box::new(ValueType::Str))], ret: Some(ValueType::Str);

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
    ListConcat         => "__tython_list_concat",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListLen            => "__tython_list_len",            params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: Some(ValueType::Int);
    ListGet            => "__tython_list_get",            params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: Some(ValueType::Int);
    ListSlice          => "__tython_list_slice",          params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int],      ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListRepeat         => "__tython_list_repeat",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListAppend         => "__tython_list_append",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: None;
    ListPop            => "__tython_list_pop",            params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: Some(ValueType::Int);
    ListClear          => "__tython_list_clear",          params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: None;

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
    ListSortStr        => "__tython_list_sort_str",        params: [ValueType::List(Box::new(ValueType::Str))], ret: None;
    ListSortBytes      => "__tython_list_sort_bytes",     params: [ValueType::List(Box::new(ValueType::Bytes))], ret: None;
    ListSortByteArray  => "__tython_list_sort_bytearray", params: [ValueType::List(Box::new(ValueType::ByteArray))], ret: None;
    SortedInt          => "__tython_sorted_int",          params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    SortedFloat        => "__tython_sorted_float",        params: [ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::List(Box::new(ValueType::Float)));
    SortedStr          => "__tython_sorted_str",          params: [ValueType::List(Box::new(ValueType::Str))], ret: Some(ValueType::List(Box::new(ValueType::Str)));
    SortedBytes        => "__tython_sorted_bytes",        params: [ValueType::List(Box::new(ValueType::Bytes))], ret: Some(ValueType::List(Box::new(ValueType::Bytes)));
    SortedByteArray    => "__tython_sorted_bytearray",    params: [ValueType::List(Box::new(ValueType::ByteArray))], ret: Some(ValueType::List(Box::new(ValueType::ByteArray)));
    ReversedList       => "__tython_reversed_list",       params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListExtend         => "__tython_list_extend",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListCopy           => "__tython_list_copy",           params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));

    // dict builtins (all Dict(...) map to ptr in LLVM; key/value types are sentinels)
    DictEmpty          => "__tython_dict_empty",          params: [], ret: Some(ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)));
    DictLen            => "__tython_dict_len",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    DictContains       => "__tython_dict_contains",       params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    DictGet            => "__tython_dict_get",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    DictSet            => "__tython_dict_set",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    DictClear          => "__tython_dict_clear",          params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: None;
    DictPop            => "__tython_dict_pop",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    DictEq             => "__tython_dict_eq",             params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    DictCopy           => "__tython_dict_copy",           params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)));
    DictValues         => "__tython_dict_values",         params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));

    // set builtins (all Set(...) map to ptr in LLVM; element type is a sentinel)
    SetEmpty           => "__tython_set_empty",           params: [], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetFromStr         => "__tython_set_from_str",        params: [ValueType::Str], ret: Some(ValueType::List(Box::new(ValueType::Str)));
    SetLen             => "__tython_set_len",             params: [ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    SetContains        => "__tython_set_contains",        params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetAdd             => "__tython_set_add",             params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetRemove          => "__tython_set_remove",          params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetDiscard         => "__tython_set_discard",         params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetPop             => "__tython_set_pop",             params: [ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    SetClear           => "__tython_set_clear",           params: [ValueType::Set(Box::new(ValueType::Int))], ret: None;
    SetEq              => "__tython_set_eq",              params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    SetCopy            => "__tython_set_copy",            params: [ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Set(Box::new(ValueType::Int)));

    // aggregate builtins
    SumInt             => "__tython_sum_int",             params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    SumFloat           => "__tython_sum_float",           params: [ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::Float);
    SumIntStart        => "__tython_sum_int_start",       params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    SumFloatStart      => "__tython_sum_float_start",     params: [ValueType::List(Box::new(ValueType::Float)), ValueType::Float], ret: Some(ValueType::Float);
    AllList            => "__tython_all_list",            params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    AnyList            => "__tython_any_list",            params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
}
