#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LlvmTy {
    I64,
    I32,
    Ptr,
}

macro_rules! define_runtime_fns {
    (
        $($variant:ident => $symbol:literal, llvm: [$($param:expr),*] -> $ret:expr);* $(;)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum RuntimeFn {
            $($variant,)*
        }

        impl RuntimeFn {
            pub fn symbol(&self) -> &'static str {
                match self {
                    $(Self::$variant => $symbol,)*
                }
            }

            pub fn params(&self) -> &'static [LlvmTy] {
                match self {
                    $(Self::$variant => &[$($param),*],)*
                }
            }

            pub fn ret(&self) -> Option<LlvmTy> {
                match self {
                    $(Self::$variant => $ret,)*
                }
            }
        }
    };
}

define_runtime_fns! {
    Malloc         => "__tython_malloc",          llvm: [LlvmTy::I64]                               -> Some(LlvmTy::Ptr);
    StrNew         => "__tython_str_new",         llvm: [LlvmTy::Ptr, LlvmTy::I64]                 -> Some(LlvmTy::Ptr);
    BytesNew       => "__tython_bytes_new",       llvm: [LlvmTy::Ptr, LlvmTy::I64]                 -> Some(LlvmTy::Ptr);
    ListNew        => "__tython_list_new",        llvm: [LlvmTy::Ptr, LlvmTy::I64]                 -> Some(LlvmTy::Ptr);
    ListSet        => "__tython_list_set",        llvm: [LlvmTy::Ptr, LlvmTy::I64, LlvmTy::I64]   -> None;
    Personality    => "__gxx_personality_v0",      llvm: []                                          -> Some(LlvmTy::I32);
    Raise          => "__tython_raise",           llvm: [LlvmTy::I64, LlvmTy::Ptr]                 -> None;
    CxaBeginCatch  => "__cxa_begin_catch",        llvm: [LlvmTy::Ptr]                               -> Some(LlvmTy::Ptr);
    CxaEndCatch    => "__cxa_end_catch",          llvm: []                                          -> None;
    CxaRethrow     => "__cxa_rethrow",            llvm: []                                          -> None;
    CaughtTypeTag  => "__tython_caught_type_tag", llvm: [LlvmTy::Ptr]                               -> Some(LlvmTy::I64);
    CaughtMessage  => "__tython_caught_message",  llvm: [LlvmTy::Ptr]                               -> Some(LlvmTy::Ptr);
    CaughtMatches  => "__tython_caught_matches",  llvm: [LlvmTy::Ptr, LlvmTy::I64]                 -> Some(LlvmTy::I64);
}
