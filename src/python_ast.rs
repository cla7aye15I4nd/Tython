#[macro_export]
macro_rules! ast_getattr {
    ($node:expr, $attr:expr) => {
        $node.getattr($attr).unwrap()
    };
}

/// Extract a typed value from a PyAny object
#[macro_export]
macro_rules! ast_extract {
    ($node:expr, $ty:ty) => {
        $node.extract::<$ty>().unwrap()
    };
}

/// Get a list attribute from an AST node and cast it to PyList
#[macro_export]
macro_rules! ast_get_list {
    ($node:expr, $attr:expr) => {{
        use pyo3::types::PyList;
        ast_getattr!($node, $attr).cast_into::<PyList>().unwrap()
    }};
}

/// Get a string attribute from an AST node
#[macro_export]
macro_rules! ast_get_string {
    ($node:expr, $attr:expr) => {{
        let val = ast_getattr!($node, $attr);
        ast_extract!(val, String)
    }};
}

/// Get an integer attribute from an AST node
#[macro_export]
macro_rules! ast_get_int {
    ($node:expr, $attr:expr, $ty:ty) => {{
        let val = ast_getattr!($node, $attr);
        ast_extract!(val, $ty)
    }};
}

/// Get the type name of an AST node
#[macro_export]
macro_rules! ast_type_name {
    ($node:expr) => {{
        let node_ref: &pyo3::Bound<pyo3::PyAny> = &$node;
        node_ref.get_type().name().unwrap().to_string()
    }};
}
