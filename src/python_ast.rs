#[macro_export]
macro_rules! ast_getattr {
    ($node:expr, $attr:expr) => {
        $node.getattr($attr).unwrap()
    };
}

#[macro_export]
macro_rules! ast_extract {
    ($node:expr, $ty:ty) => {
        $node.extract::<$ty>().unwrap()
    };
}

#[macro_export]
macro_rules! ast_get_list {
    ($node:expr, $attr:expr) => {{
        use pyo3::types::PyList;
        $crate::ast_getattr!($node, $attr)
            .cast_into::<PyList>()
            .unwrap()
    }};
}

#[macro_export]
macro_rules! ast_get_string {
    ($node:expr, $attr:expr) => {{
        let val = $crate::ast_getattr!($node, $attr);
        $crate::ast_extract!(val, String)
    }};
}

#[macro_export]
macro_rules! ast_get_int {
    ($node:expr, $attr:expr, $ty:ty) => {{
        let val = $crate::ast_getattr!($node, $attr);
        $crate::ast_extract!(val, $ty)
    }};
}

#[macro_export]
macro_rules! ast_type_name {
    ($node:expr) => {{
        let node_ref: &pyo3::Bound<pyo3::PyAny> = &$node;
        node_ref.get_type().name().unwrap().to_string()
    }};
}

#[macro_export]
macro_rules! ast_get_string_or {
    ($node:expr, $attr:expr, $default:expr) => {{
        let val = $crate::ast_getattr!($node, $attr);
        if val.is_none() {
            $default
        } else {
            $crate::ast_extract!(val, String)
        }
    }};
}
