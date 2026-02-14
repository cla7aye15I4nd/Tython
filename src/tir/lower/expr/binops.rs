use anyhow::Result;

use crate::tir::{
    builtin, type_rules, ArithBinOp, CastKind, RawBinOp, TirExpr, TirExprKind, ValueType,
};

use crate::tir::lower::Lowering;

impl Lowering {
    /// Resolve a binary operation into a TIR expression.
    /// Sequence operations (concat, repeat) become `ExternalCall`;
    /// arithmetic/bitwise operations become `BinOp`.
    pub(in crate::tir::lower) fn resolve_binop(
        &self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        if let Some(class_expr) =
            self.try_lower_class_binop_magic(line, raw_op, left.clone(), right.clone())?
        {
            return Ok(class_expr);
        }

        let left_ast = left.ty.to_type();
        let right_ast = right.ty.to_type();
        let rule = type_rules::lookup_binop(raw_op, &left_ast, &right_ast).ok_or_else(|| {
            self.type_error(
                line,
                type_rules::binop_type_error_message(raw_op, &left_ast, &right_ast),
            )
        })?;

        let result_vty = Self::to_value_type(&rule.result_type);

        // Sequence operations → ExternalCall
        if let Some(func) = Self::resolve_seq_binop(raw_op, &result_vty) {
            let args = if matches!(raw_op, RawBinOp::Arith(ArithBinOp::Mul))
                && left.ty == ValueType::Int
            {
                // Repeat with int on left: normalize to (seq, int)
                vec![right, left]
            } else {
                vec![left, right]
            };
            return Ok(TirExpr {
                kind: TirExprKind::ExternalCall { func, args },
                ty: result_vty,
            });
        }

        // Arithmetic/bitwise → BinOp
        let typed_op = type_rules::resolve_typed_binop(raw_op, &rule.result_type);
        let final_left = Self::apply_coercion(left, rule.left_coercion);
        let final_right = Self::apply_coercion(right, rule.right_coercion);

        // Construct typed operation variant based on TypedBinOp
        let kind = match typed_op {
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Add) => {
                TirExprKind::IntAdd(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Sub) => {
                TirExprKind::IntSub(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Mul) => {
                TirExprKind::IntMul(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::FloorDiv) => {
                TirExprKind::IntFloorDiv(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Mod) => {
                TirExprKind::IntMod(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Pow) => {
                TirExprKind::IntPow(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Add) => {
                TirExprKind::FloatAdd(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Sub) => {
                TirExprKind::FloatSub(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Mul) => {
                TirExprKind::FloatMul(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Div) => {
                TirExprKind::FloatDiv(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::FloorDiv) => {
                TirExprKind::FloatFloorDiv(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Mod) => {
                TirExprKind::FloatMod(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Pow) => {
                TirExprKind::FloatPow(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::BitAnd) => {
                TirExprKind::BitAnd(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::BitOr) => {
                TirExprKind::BitOr(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::BitXor) => {
                TirExprKind::BitXor(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::LShift) => {
                TirExprKind::LShift(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::RShift) => {
                TirExprKind::RShift(Box::new(final_left), Box::new(final_right))
            }
        };
        Ok(TirExpr {
            kind,
            ty: result_vty,
        })
    }

    fn resolve_seq_binop(raw_op: RawBinOp, result_ty: &ValueType) -> Option<builtin::BuiltinFn> {
        match (raw_op, result_ty) {
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::Str) => {
                Some(builtin::BuiltinFn::StrConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::Bytes) => {
                Some(builtin::BuiltinFn::BytesConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::ByteArray) => {
                Some(builtin::BuiltinFn::ByteArrayConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::List(_)) => {
                Some(builtin::BuiltinFn::ListConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::Str) => {
                Some(builtin::BuiltinFn::StrRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::Bytes) => {
                Some(builtin::BuiltinFn::BytesRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::ByteArray) => {
                Some(builtin::BuiltinFn::ByteArrayRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::List(_)) => {
                Some(builtin::BuiltinFn::ListRepeat)
            }
            _ => None,
        }
    }

    fn try_lower_class_binop_magic(
        &self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<Option<TirExpr>> {
        let magic = type_rules::lookup_class_binop_magic(raw_op)
            .expect("ICE: missing class binop magic mapping");

        let mut found_class_side = false;

        if let ValueType::Class(class_name) = &left.ty {
            found_class_side = true;
            let class_info = self.lookup_class(line, class_name)?;
            if class_info.methods.contains_key(magic.left_method) {
                return self
                    .lower_class_magic_method_with_args(
                        line,
                        left,
                        &[magic.left_method],
                        None,
                        "binary operator",
                        vec![right],
                    )
                    .map(Some);
            }
        }

        if let ValueType::Class(class_name) = &right.ty {
            found_class_side = true;
            let class_info = self.lookup_class(line, class_name)?;
            if class_info.methods.contains_key(magic.right_method) {
                return self
                    .lower_class_magic_method_with_args(
                        line,
                        right,
                        &[magic.right_method],
                        None,
                        "binary operator",
                        vec![left],
                    )
                    .map(Some);
            }
        }

        if found_class_side {
            return Err(self.type_error(
                line,
                format!(
                    "operator `{}` requires class magic methods `{}` or `{}` for operand types `{}` and `{}`",
                    raw_op, magic.left_method, magic.right_method, left.ty, right.ty
                ),
            ));
        }

        Ok(None)
    }

    pub(in crate::tir::lower) fn apply_coercion(
        expr: TirExpr,
        coercion: type_rules::Coercion,
    ) -> TirExpr {
        match coercion {
            type_rules::Coercion::None => expr,
            type_rules::Coercion::ToFloat => {
                if expr.ty == ValueType::Float {
                    expr
                } else {
                    let cast_kind = match &expr.ty {
                        ValueType::Int => CastKind::IntToFloat,
                        ValueType::Bool => CastKind::BoolToFloat,
                        _ => unreachable!(),
                    };
                    TirExpr {
                        kind: TirExprKind::Cast {
                            kind: cast_kind,
                            arg: Box::new(expr),
                        },
                        ty: ValueType::Float,
                    }
                }
            }
        }
    }
}
