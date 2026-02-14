use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::tir::TirExpr;
use crate::{ast_getattr, ast_type_name};

use super::super::Lowering;
use super::NormalizedCallArgs;

impl Lowering {
    pub(super) fn normalize_call_args(
        &mut self,
        args_list: &Bound<PyList>,
        keywords_list: &Bound<PyList>,
        line: usize,
    ) -> Result<NormalizedCallArgs> {
        let mut positional = Vec::with_capacity(args_list.len());
        for arg in args_list.iter() {
            positional.push(self.lower_expr(&arg)?);
        }

        let mut keyword: Vec<(String, TirExpr)> = Vec::with_capacity(keywords_list.len());
        for kw in keywords_list.iter() {
            let kw_name_node = ast_getattr!(kw, "arg");
            if kw_name_node.is_none() {
                return Err(self.syntax_error(
                    line,
                    "dictionary unpacking in calls (`**kwargs`) is not supported",
                ));
            }
            let kw_name = kw_name_node.extract::<String>()?;
            let kw_value = self.lower_expr(&ast_getattr!(kw, "value"))?;
            keyword.push((kw_name, kw_value));
        }

        Ok(NormalizedCallArgs {
            positional,
            keyword,
        })
    }

    pub(super) fn detect_sum_generator_fast_path(
        &mut self,
        func_node: &Bound<PyAny>,
        args_list: &Bound<PyList>,
        keywords_list: &Bound<PyList>,
        line: usize,
    ) -> Result<Option<crate::tir::CallResult>> {
        if ast_type_name!(func_node) != "Name" {
            return Ok(None);
        }
        let func_name = crate::ast_get_string!(func_node, "id");
        if func_name != "sum" || args_list.len() != 2 || !keywords_list.is_empty() {
            return Ok(None);
        }
        let first_arg = args_list.get_item(0)?;
        if ast_type_name!(first_arg) != "GeneratorExp" {
            return Ok(None);
        }

        let start_expr = self.lower_expr(&args_list.get_item(1)?)?;
        let lowered = self.lower_sum_generator(&first_arg, start_expr, line)?;
        Ok(Some(crate::tir::CallResult::Expr(lowered)))
    }
}
