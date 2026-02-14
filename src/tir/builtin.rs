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
    StrHash       => "__tython_str_hash",       params: [ValueType::Str],                          ret: Some(ValueType::Int);

    // bytes builtins
    BytesConcat   => "__tython_bytes_concat",   params: [ValueType::Bytes, ValueType::Bytes],       ret: Some(ValueType::Bytes);
    BytesRepeat   => "__tython_bytes_repeat",   params: [ValueType::Bytes, ValueType::Int],         ret: Some(ValueType::Bytes);
    BytesLen      => "__tython_bytes_len",      params: [ValueType::Bytes],                        ret: Some(ValueType::Int);
    BytesCmp      => "__tython_bytes_cmp",      params: [ValueType::Bytes, ValueType::Bytes],       ret: Some(ValueType::Int);
    BytesEq       => "__tython_bytes_eq",       params: [ValueType::Bytes, ValueType::Bytes],       ret: Some(ValueType::Int);
    BytesFromInt  => "__tython_bytes_from_int", params: [ValueType::Int],                          ret: Some(ValueType::Bytes);
    BytesFromStr  => "__tython_bytes_from_str", params: [ValueType::Str],                          ret: Some(ValueType::Bytes);
    BytesCapitalize => "__tython_bytes_capitalize", params: [ValueType::Bytes],                    ret: Some(ValueType::Bytes);
    BytesCenter   => "__tython_bytes_center",   params: [ValueType::Bytes, ValueType::Int, ValueType::Bytes], ret: Some(ValueType::Bytes);
    BytesCount    => "__tython_bytes_count",    params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Int);
    BytesDecode   => "__tython_bytes_decode",   params: [ValueType::Bytes],                        ret: Some(ValueType::Str);
    BytesEndsWith => "__tython_bytes_endswith", params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Bool);
    BytesExpandTabs => "__tython_bytes_expandtabs", params: [ValueType::Bytes, ValueType::Int],    ret: Some(ValueType::Bytes);
    BytesFind     => "__tython_bytes_find",     params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Int);
    BytesFromHex  => "__tython_bytes_fromhex",  params: [ValueType::Bytes, ValueType::Str],        ret: Some(ValueType::Bytes);
    BytesHex      => "__tython_bytes_hex",      params: [ValueType::Bytes],                        ret: Some(ValueType::Str);
    BytesIndex    => "__tython_bytes_index",    params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Int);
    BytesIsAlnum  => "__tython_bytes_isalnum",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesIsAlpha  => "__tython_bytes_isalpha",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesIsAscii  => "__tython_bytes_isascii",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesIsDigit  => "__tython_bytes_isdigit",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesIsLower  => "__tython_bytes_islower",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesIsSpace  => "__tython_bytes_isspace",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesIsTitle  => "__tython_bytes_istitle",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesIsUpper  => "__tython_bytes_isupper",  params: [ValueType::Bytes],                        ret: Some(ValueType::Bool);
    BytesJoin     => "__tython_bytes_join",     params: [ValueType::Bytes, ValueType::List(Box::new(ValueType::Bytes))], ret: Some(ValueType::Bytes);
    BytesLJust    => "__tython_bytes_ljust",    params: [ValueType::Bytes, ValueType::Int, ValueType::Bytes], ret: Some(ValueType::Bytes);
    BytesLower    => "__tython_bytes_lower",    params: [ValueType::Bytes],                        ret: Some(ValueType::Bytes);
    BytesLStrip   => "__tython_bytes_lstrip",   params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Bytes);
    BytesMakeTrans => "__tython_bytes_maketrans", params: [ValueType::Bytes, ValueType::Bytes, ValueType::Bytes], ret: Some(ValueType::Bytes);
    BytesPartition => "__tython_bytes_partition", params: [ValueType::Bytes, ValueType::Bytes],    ret: Some(ValueType::Tuple(vec![ValueType::Bytes, ValueType::Bytes, ValueType::Bytes]));
    BytesRemovePrefix => "__tython_bytes_removeprefix", params: [ValueType::Bytes, ValueType::Bytes], ret: Some(ValueType::Bytes);
    BytesRemoveSuffix => "__tython_bytes_removesuffix", params: [ValueType::Bytes, ValueType::Bytes], ret: Some(ValueType::Bytes);
    BytesReplace  => "__tython_bytes_replace",  params: [ValueType::Bytes, ValueType::Bytes, ValueType::Bytes], ret: Some(ValueType::Bytes);
    BytesRFind    => "__tython_bytes_rfind",    params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Int);
    BytesRIndex   => "__tython_bytes_rindex",   params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Int);
    BytesRJust    => "__tython_bytes_rjust",    params: [ValueType::Bytes, ValueType::Int, ValueType::Bytes], ret: Some(ValueType::Bytes);
    BytesRPartition => "__tython_bytes_rpartition", params: [ValueType::Bytes, ValueType::Bytes],  ret: Some(ValueType::Tuple(vec![ValueType::Bytes, ValueType::Bytes, ValueType::Bytes]));
    BytesRSplit   => "__tython_bytes_rsplit",   params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::List(Box::new(ValueType::Bytes)));
    BytesRStrip   => "__tython_bytes_rstrip",   params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Bytes);
    BytesSplit    => "__tython_bytes_split",    params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::List(Box::new(ValueType::Bytes)));
    BytesSplitLines => "__tython_bytes_splitlines", params: [ValueType::Bytes],                    ret: Some(ValueType::List(Box::new(ValueType::Bytes)));
    BytesStartsWith => "__tython_bytes_startswith", params: [ValueType::Bytes, ValueType::Bytes],  ret: Some(ValueType::Bool);
    BytesStrip    => "__tython_bytes_strip",    params: [ValueType::Bytes, ValueType::Bytes],      ret: Some(ValueType::Bytes);
    BytesSwapCase => "__tython_bytes_swapcase", params: [ValueType::Bytes],                        ret: Some(ValueType::Bytes);
    BytesTitle    => "__tython_bytes_title",    params: [ValueType::Bytes],                        ret: Some(ValueType::Bytes);
    BytesTranslate => "__tython_bytes_translate", params: [ValueType::Bytes, ValueType::Bytes],    ret: Some(ValueType::Bytes);
    BytesUpper    => "__tython_bytes_upper",    params: [ValueType::Bytes],                        ret: Some(ValueType::Bytes);
    BytesZFill    => "__tython_bytes_zfill",    params: [ValueType::Bytes, ValueType::Int],        ret: Some(ValueType::Bytes);
    BytesGet      => "__tython_bytes_get",      params: [ValueType::Bytes, ValueType::Int],        ret: Some(ValueType::Int);

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
    ByteArrayCopy      => "__tython_bytearray_copy",       params: [ValueType::ByteArray],                       ret: Some(ValueType::ByteArray);
    ByteArrayPop       => "__tython_bytearray_pop",        params: [ValueType::ByteArray],                       ret: Some(ValueType::Int);
    ByteArrayCapitalize => "__tython_bytearray_capitalize", params: [ValueType::ByteArray],                      ret: Some(ValueType::ByteArray);
    ByteArrayCenter    => "__tython_bytearray_center",     params: [ValueType::ByteArray, ValueType::Int, ValueType::Bytes], ret: Some(ValueType::ByteArray);
    ByteArrayCount     => "__tython_bytearray_count",      params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::Int);
    ByteArrayDecode    => "__tython_bytearray_decode",     params: [ValueType::ByteArray],                       ret: Some(ValueType::Str);
    ByteArrayEndsWith  => "__tython_bytearray_endswith",   params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::Bool);
    ByteArrayExpandTabs => "__tython_bytearray_expandtabs", params: [ValueType::ByteArray, ValueType::Int],      ret: Some(ValueType::ByteArray);
    ByteArrayFind      => "__tython_bytearray_find",       params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::Int);
    ByteArrayFromHex   => "__tython_bytearray_fromhex",    params: [ValueType::ByteArray, ValueType::Str],       ret: Some(ValueType::ByteArray);
    ByteArrayHex       => "__tython_bytearray_hex",        params: [ValueType::ByteArray],                       ret: Some(ValueType::Str);
    ByteArrayIndex     => "__tython_bytearray_index",      params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::Int);
    ByteArrayIsAlnum   => "__tython_bytearray_isalnum",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayIsAlpha   => "__tython_bytearray_isalpha",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayIsAscii   => "__tython_bytearray_isascii",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayIsDigit   => "__tython_bytearray_isdigit",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayIsLower   => "__tython_bytearray_islower",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayIsSpace   => "__tython_bytearray_isspace",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayIsTitle   => "__tython_bytearray_istitle",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayIsUpper   => "__tython_bytearray_isupper",    params: [ValueType::ByteArray],                       ret: Some(ValueType::Bool);
    ByteArrayJoin      => "__tython_bytearray_join",       params: [ValueType::ByteArray, ValueType::List(Box::new(ValueType::ByteArray))], ret: Some(ValueType::ByteArray);
    ByteArrayLJust     => "__tython_bytearray_ljust",      params: [ValueType::ByteArray, ValueType::Int, ValueType::Bytes], ret: Some(ValueType::ByteArray);
    ByteArrayLower     => "__tython_bytearray_lower",      params: [ValueType::ByteArray],                       ret: Some(ValueType::ByteArray);
    ByteArrayLStrip    => "__tython_bytearray_lstrip",     params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::ByteArray);
    ByteArrayMakeTrans => "__tython_bytearray_maketrans",  params: [ValueType::ByteArray, ValueType::Bytes, ValueType::Bytes], ret: Some(ValueType::Bytes);
    ByteArrayPartition => "__tython_bytearray_partition",  params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::Tuple(vec![ValueType::ByteArray, ValueType::ByteArray, ValueType::ByteArray]));
    ByteArrayRemovePrefix => "__tython_bytearray_removeprefix", params: [ValueType::ByteArray, ValueType::Bytes], ret: Some(ValueType::ByteArray);
    ByteArrayRemoveSuffix => "__tython_bytearray_removesuffix", params: [ValueType::ByteArray, ValueType::Bytes], ret: Some(ValueType::ByteArray);
    ByteArrayReplace   => "__tython_bytearray_replace",    params: [ValueType::ByteArray, ValueType::Bytes, ValueType::Bytes], ret: Some(ValueType::ByteArray);
    ByteArrayRFind     => "__tython_bytearray_rfind",      params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::Int);
    ByteArrayRIndex    => "__tython_bytearray_rindex",     params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::Int);
    ByteArrayRJust     => "__tython_bytearray_rjust",      params: [ValueType::ByteArray, ValueType::Int, ValueType::Bytes], ret: Some(ValueType::ByteArray);
    ByteArrayRPartition => "__tython_bytearray_rpartition", params: [ValueType::ByteArray, ValueType::Bytes],    ret: Some(ValueType::Tuple(vec![ValueType::ByteArray, ValueType::ByteArray, ValueType::ByteArray]));
    ByteArrayRSplit    => "__tython_bytearray_rsplit",     params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::List(Box::new(ValueType::ByteArray)));
    ByteArrayRStrip    => "__tython_bytearray_rstrip",     params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::ByteArray);
    ByteArraySplit     => "__tython_bytearray_split",      params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::List(Box::new(ValueType::ByteArray)));
    ByteArraySplitLines => "__tython_bytearray_splitlines", params: [ValueType::ByteArray],                       ret: Some(ValueType::List(Box::new(ValueType::ByteArray)));
    ByteArrayStartsWith => "__tython_bytearray_startswith", params: [ValueType::ByteArray, ValueType::Bytes],    ret: Some(ValueType::Bool);
    ByteArrayStrip     => "__tython_bytearray_strip",      params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::ByteArray);
    ByteArraySwapCase  => "__tython_bytearray_swapcase",   params: [ValueType::ByteArray],                       ret: Some(ValueType::ByteArray);
    ByteArrayTitle     => "__tython_bytearray_title",      params: [ValueType::ByteArray],                       ret: Some(ValueType::ByteArray);
    ByteArrayTranslate => "__tython_bytearray_translate",  params: [ValueType::ByteArray, ValueType::Bytes],     ret: Some(ValueType::ByteArray);
    ByteArrayUpper     => "__tython_bytearray_upper",      params: [ValueType::ByteArray],                       ret: Some(ValueType::ByteArray);
    ByteArrayZFill     => "__tython_bytearray_zfill",      params: [ValueType::ByteArray, ValueType::Int],       ret: Some(ValueType::ByteArray);
    ByteArrayGet       => "__tython_bytearray_get",       params: [ValueType::ByteArray, ValueType::Int],       ret: Some(ValueType::Int);

    // list builtins (all List(...) map to ptr in LLVM; inner type is a sentinel)
    ListEmpty          => "__tython_list_empty",          params: [],                                                                                ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListConcat         => "__tython_list_concat",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListLen            => "__tython_list_len",            params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: Some(ValueType::Int);
    ListGet            => "__tython_list_get",            params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: Some(ValueType::Int);
    ListSlice          => "__tython_list_slice",          params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int],      ret: Some(ValueType::List(Box::new(ValueType::Int)));
    TupleGetItem       => "__tython_tuple_getitem",       params: [ValueType::Tuple(vec![ValueType::Int]), ValueType::Int],                        ret: Some(ValueType::Int);
    ListRepeat         => "__tython_list_repeat",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListAppend         => "__tython_list_append",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int],                       ret: None;
    ListPop            => "__tython_list_pop",            params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: Some(ValueType::Int);
    ListClear          => "__tython_list_clear",          params: [ValueType::List(Box::new(ValueType::Int))],                                       ret: None;

    // list equality
    ListEqShallow      => "__tython_list_eq_shallow",     params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    ListEqDeep         => "__tython_list_eq_deep",        params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    ListEqGeneric      => "__tython_list_eq_generic",     params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    ListEqByTag        => "__tython_list_eq_by_tag",      params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    ListLtByTag        => "__tython_list_lt_by_tag",      params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);

    // list containment
    ListContains       => "__tython_list_contains",       params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    ListContainsByTag  => "__tython_list_contains_by_tag", params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Bool);

    // str containment
    StrContains        => "__tython_str_contains",        params: [ValueType::Str, ValueType::Str], ret: Some(ValueType::Bool);

    // list methods
    ListInsert         => "__tython_list_insert",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    ListRemove         => "__tython_list_remove",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    ListRemoveByTag    => "__tython_list_remove_by_tag",  params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    ListIndex          => "__tython_list_index",          params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    ListIndexByTag     => "__tython_list_index_by_tag",   params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Int);
    ListCount          => "__tython_list_count",          params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    ListCountByTag     => "__tython_list_count_by_tag",   params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Int);
    ListReverse        => "__tython_list_reverse",        params: [ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListSortInt        => "__tython_list_sort_int",       params: [ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListSortFloat      => "__tython_list_sort_float",     params: [ValueType::List(Box::new(ValueType::Float))], ret: None;
    ListSortStr        => "__tython_list_sort_str",        params: [ValueType::List(Box::new(ValueType::Str))], ret: None;
    ListSortBytes      => "__tython_list_sort_bytes",     params: [ValueType::List(Box::new(ValueType::Bytes))], ret: None;
    ListSortByteArray  => "__tython_list_sort_bytearray", params: [ValueType::List(Box::new(ValueType::ByteArray))], ret: None;
    ListSortAny        => "__tython_list_sort_any",       params: [ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListSortByTag      => "__tython_list_sort_by_tag",    params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SortedInt          => "__tython_sorted_int",          params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    SortedFloat        => "__tython_sorted_float",        params: [ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::List(Box::new(ValueType::Float)));
    SortedStr          => "__tython_sorted_str",          params: [ValueType::List(Box::new(ValueType::Str))], ret: Some(ValueType::List(Box::new(ValueType::Str)));
    SortedBytes        => "__tython_sorted_bytes",        params: [ValueType::List(Box::new(ValueType::Bytes))], ret: Some(ValueType::List(Box::new(ValueType::Bytes)));
    SortedByteArray    => "__tython_sorted_bytearray",    params: [ValueType::List(Box::new(ValueType::ByteArray))], ret: Some(ValueType::List(Box::new(ValueType::ByteArray)));
    SortedAny          => "__tython_sorted_any",          params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    SortedByTag        => "__tython_sorted_by_tag",       params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ReversedList       => "__tython_reversed_list",       params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListExtend         => "__tython_list_extend",         params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: None;
    ListCopy           => "__tython_list_copy",           params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListIAdd           => "__tython_list_iadd",           params: [ValueType::List(Box::new(ValueType::Int)), ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListIMul           => "__tython_list_imul",           params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    ListDel            => "__tython_list_del",            params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: None;

    // dict builtins (all Dict(...) map to ptr in LLVM; key/value types are sentinels)
    DictEmpty          => "__tython_dict_empty",          params: [], ret: Some(ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)));
    DictLen            => "__tython_dict_len",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    DictContains       => "__tython_dict_contains",       params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    DictContainsByTag  => "__tython_dict_contains_by_tag", params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Bool);
    DictGet            => "__tython_dict_get",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    DictGetByTag       => "__tython_dict_get_by_tag",     params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Int);
    DictGetDefaultByTag => "__tython_dict_get_default_by_tag", params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int, ValueType::Int], ret: Some(ValueType::Int);
    DictSet            => "__tython_dict_set",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    DictSetByTag       => "__tython_dict_set_by_tag",     params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int, ValueType::Int], ret: None;
    DictSetDefaultByTag => "__tython_dict_setdefault_by_tag", params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int, ValueType::Int], ret: Some(ValueType::Int);
    DictDelByTag       => "__tython_dict_del_by_tag",     params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    DictClear          => "__tython_dict_clear",          params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: None;
    DictPop            => "__tython_dict_pop",            params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    DictPopByTag       => "__tython_dict_pop_by_tag",     params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Int);
    DictPopDefaultByTag => "__tython_dict_pop_default_by_tag", params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int, ValueType::Int], ret: Some(ValueType::Int);
    DictEq             => "__tython_dict_eq",             params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    DictEqByTag        => "__tython_dict_eq_by_tag",      params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Bool);
    DictUpdateByTag    => "__tython_dict_update_by_tag",  params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: None;
    DictOrByTag        => "__tython_dict_or_by_tag",      params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)));
    DictIOrByTag       => "__tython_dict_ior_by_tag",     params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)));
    DictFromKeysByTag  => "__tython_dict_fromkeys_by_tag", params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)));
    DictCopy           => "__tython_dict_copy",           params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)));
    DictItems          => "__tython_dict_items",          params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    DictPopItem        => "__tython_dict_popitem",        params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::Tuple(vec![ValueType::Int, ValueType::Int]));
    DictKeys           => "__tython_dict_keys",           params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));
    DictValues         => "__tython_dict_values",         params: [ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int))], ret: Some(ValueType::List(Box::new(ValueType::Int)));

    // set builtins (all Set(...) map to ptr in LLVM; element type is a sentinel)
    SetEmpty           => "__tython_set_empty",           params: [], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetFromStr         => "__tython_set_from_str",        params: [ValueType::Str], ret: Some(ValueType::List(Box::new(ValueType::Str)));
    SetLen             => "__tython_set_len",             params: [ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    SetContains        => "__tython_set_contains",        params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetContainsByTag   => "__tython_set_contains_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: Some(ValueType::Bool);
    SetAdd             => "__tython_set_add",             params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetAddByTag        => "__tython_set_add_by_tag",      params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    SetRemove          => "__tython_set_remove",          params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetRemoveByTag     => "__tython_set_remove_by_tag",   params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    SetDiscard         => "__tython_set_discard",         params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetDiscardByTag    => "__tython_set_discard_by_tag",  params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Int, ValueType::Int], ret: None;
    SetUnionByTag      => "__tython_set_union_by_tag",    params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetUpdateByTag     => "__tython_set_update_by_tag",   params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetIntersectionByTag => "__tython_set_intersection_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetIntersectionUpdateByTag => "__tython_set_intersection_update_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetDifferenceByTag => "__tython_set_difference_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetDifferenceUpdateByTag => "__tython_set_difference_update_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetSymmetricDifferenceByTag => "__tython_set_symmetric_difference_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetSymmetricDifferenceUpdateByTag => "__tython_set_symmetric_difference_update_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: None;
    SetIsDisjointByTag => "__tython_set_isdisjoint_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetIsSubsetByTag   => "__tython_set_issubset_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetIsSupersetByTag => "__tython_set_issuperset_by_tag", params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetLtByTag         => "__tython_set_lt_by_tag",       params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetLeByTag         => "__tython_set_le_by_tag",       params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetGtByTag         => "__tython_set_gt_by_tag",       params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetGeByTag         => "__tython_set_ge_by_tag",       params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetIAndByTag       => "__tython_set_iand_by_tag",     params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetIOrByTag        => "__tython_set_ior_by_tag",      params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetISubByTag       => "__tython_set_isub_by_tag",     params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetIXorByTag       => "__tython_set_ixor_by_tag",     params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Set(Box::new(ValueType::Int)));
    SetPop             => "__tython_set_pop",             params: [ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    SetClear           => "__tython_set_clear",           params: [ValueType::Set(Box::new(ValueType::Int))], ret: None;
    SetEq              => "__tython_set_eq",              params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    SetEqByTag         => "__tython_set_eq_by_tag",       params: [ValueType::Set(Box::new(ValueType::Int)), ValueType::Set(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Bool);
    SetCopy            => "__tython_set_copy",            params: [ValueType::Set(Box::new(ValueType::Int))], ret: Some(ValueType::Set(Box::new(ValueType::Int)));

    // aggregate builtins
    SumInt             => "__tython_sum_int",             params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Int);
    SumFloat           => "__tython_sum_float",           params: [ValueType::List(Box::new(ValueType::Float))], ret: Some(ValueType::Float);
    SumIntStart        => "__tython_sum_int_start",       params: [ValueType::List(Box::new(ValueType::Int)), ValueType::Int], ret: Some(ValueType::Int);
    SumFloatStart      => "__tython_sum_float_start",     params: [ValueType::List(Box::new(ValueType::Float)), ValueType::Float], ret: Some(ValueType::Float);
    AllList            => "__tython_all_list",            params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
    AnyList            => "__tython_any_list",            params: [ValueType::List(Box::new(ValueType::Int))], ret: Some(ValueType::Bool);
}
