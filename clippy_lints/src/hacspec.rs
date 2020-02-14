use crate::utils::*;
use rustc::lint::in_external_macro;
use rustc_hir::{
    intravisit, BindingAnnotation, Body, Expr, ExprKind, FnDecl, HirId, Item, ItemKind, Mod, Mutability, Param, Pat,
    PatKind, Path, PathSegment, Ty, TyKind,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};
use rustc_span::{
    hygiene::{DesugaringKind, ExpnKind},
    Span,
};

declare_clippy_lint! {
    pub HACSPEC,
    pedantic,
    "Checks whether the code belongs to the hacspec subset of Rust"
}

declare_lint_pass!(Hacspec => [HACSPEC]);

const ALLOWED_PATHS: &[&[&str]] = &[
    &["hacspec"],
    &["contracts"],
    &["std"],
    &["std", "num", "ParseIntError"],
    &["std", "ops"],
    &["std", "cmp", "min"],
    &["std", "cmp", "PartialEq"],
    &["std", "fmt"],
    &["{{root}}", "std", "prelude", "v1"],
    &["{{root}}", "std", "iter", "Iterator", "next"],
    &["{{root}}", "std", "iter", "IntoIterator", "into_iter"],
    &["{{root}}", "std", "option", "Option"],
    &["{{root}}", "std", "ops", "Range"],
    &["{{root}}", "std", "ops", "RangeFull"],
];

fn allowed_path(queried_use: &[PathSegment<'_>]) -> bool {
    ALLOWED_PATHS.iter().any(|&allowed_use| {
        allowed_use
            .iter()
            .zip(queried_use.iter())
            .filter(|(&allowed_segment, queried_segment)| *allowed_segment == *queried_segment.ident.name.as_str())
            .count()
            == allowed_use.len()
    })
}

fn allowed_type(typ: &Ty<'_>) -> bool {
    match &typ.kind {
        TyKind::Array(ref typ, _) => allowed_type(typ),
        TyKind::Tup(typs) => typs.iter().all(|typ| allowed_type(typ)),
        TyKind::Path(_) => true,
        _ => false,
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Hacspec {
    fn check_path(&mut self, cx: &LateContext<'a, 'tcx>, path: &'tcx Path<'tcx>, _: HirId) {
        // Items used in the code are whitelisted
        if in_external_macro(cx.sess(), path.span) {
            return;
        };
        if path.segments.len() == 1 || allowed_path(&path.segments) {
            // Paths of len 1 correspond to items inside the crate, except when used in imports
            return;
        };
        span_lint(cx, HACSPEC, path.span, &format!("[HACSPEC] Unauthorized item {}", path))
    }

    fn check_mod(&mut self, cx: &LateContext<'a, 'tcx>, m: &'tcx Mod<'tcx>, span: Span, _: HirId) {
        let map = cx.tcx.hir();
        // only check top level `use` statements
        for item in m.item_ids {
            let item = map.expect_item(item.id);
            if in_external_macro(cx.sess(), span) {
                continue;
            };
            if let ItemKind::Use(ref path, _) = item.kind {
                // Even though the check_path also checks the import path, we have to restricts
                // imports path of lenght 1 that are not allowed
                if allowed_path(&path.segments) {
                    continue;
                };
                span_lint(cx, HACSPEC, item.span, &format!("Unauthorized item {}", path))
            }
        }
    }

    fn check_param(&mut self, cx: &LateContext<'a, 'tcx>, param: &'tcx Param<'tcx>) {
        if in_external_macro(cx.sess(), param.span) {
            return;
        };
        // Function parameters cannot be borrowed
        param.pat.walk(|pat: &Pat<'_>| {
            if let PatKind::Binding(binding_annot, _, _, _) = &pat.kind {
                if *binding_annot != BindingAnnotation::Unannotated {
                    span_lint(
                        cx,
                        HACSPEC,
                        pat.span,
                        &"[HACSPEC] Cannot annotate function parameter with mutability",
                    );
                    return false;
                }
                true
            } else {
                span_lint(cx, HACSPEC, pat.span, &"[HACSPEC] Wrong parameter format");
                false
            }
        })
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        _: intravisit::FnKind<'tcx>,
        sig: &'tcx FnDecl<'tcx>,
        _: &'tcx Body<'tcx>,
        span: Span,
        _: HirId,
    ) {
        if in_external_macro(cx.sess(), span) {
            return;
        };
        // The types of function parameters cannot be references
        for param in sig.inputs.iter() {
            if !allowed_type(param) {
                span_lint(cx, HACSPEC, param.span, &"[HACSPEC] Unsupported type")
            }
        }
    }

    fn check_item(&mut self, cx: &LateContext<'a, 'tcx>, item: &'tcx Item<'tcx>) {
        if in_external_macro(cx.sess(), item.span) {
            return;
        }
        match &item.kind {
            ItemKind::TyAlias(ref typ, _) | ItemKind::Const(ref typ, _) => {
                if !allowed_type(typ) {
                    span_lint(cx, HACSPEC, item.span, &"[HACSPEC] Unauthorized type for alias")
                }
            },
            ItemKind::Static(typ, m, _) => {
                if !allowed_type(typ) || *m == Mutability::Mut {
                    span_lint(cx, HACSPEC, item.span, &"[HACSPEC] Unauthorized static item")
                }
            },
            ItemKind::ExternCrate(_) | ItemKind::Use(_, _) | ItemKind::Fn(_, _, _) => (),
            _ => span_lint(cx, HACSPEC, item.span, &"[HACSPEC] Unauthorized item"),
        }
    }

    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &Expr<'tcx>) {
        if in_external_macro(cx.sess(), expr.span) {
            return;
        }
        // Restricts the items used to the whitelist
        match &expr.kind {
            ExprKind::AddrOf(_, _, _) => {
                if expr.span.from_expansion() {
                    match expr.span.ctxt().outer_expn_data().kind {
                        // Loop ranges are desugared with an mutable addrOf expression so we
                        // authorize them
                        ExpnKind::Desugaring(DesugaringKind::ForLoop) => (),
                        _ => span_lint(cx, HACSPEC, expr.span, &"[HACSPEC] Unauthorized reference expression"),
                    }
                }
            },
            ExprKind::Type(_, typ) => {
                if !allowed_type(typ) {
                    span_lint(cx, HACSPEC, expr.span, &"[HACSPEC] Unauthorized type for expression")
                }
            },
            ExprKind::Binary(_, _, _)
            | ExprKind::Unary(_, _)
            | ExprKind::Lit(_)
            | ExprKind::Loop(_, _, _)
            | ExprKind::Match(_, _, _)
            | ExprKind::Assign(_, _, _)
            | ExprKind::AssignOp(_, _, _)
            | ExprKind::Break(_, _)
            | ExprKind::Repeat(_, _)
            | ExprKind::Cast(_, _)
            | ExprKind::Array(_)
            | ExprKind::Call(_, _)
            | ExprKind::MethodCall(_, _, _)
            | ExprKind::Tup(_)
            | ExprKind::Block(_, _)
            | ExprKind::Field(_, _)
            | ExprKind::Index(_, _)
            | ExprKind::Struct(_, _, _)
            | ExprKind::DropTemps(_)
            | ExprKind::Path(_)
            | ExprKind::Ret(_) => (),
            ExprKind::Yield(_, _)
            | ExprKind::Closure(_, _, _, _, _)
            | ExprKind::Box(_)
            | ExprKind::Continue(_)
            | ExprKind::InlineAsm(_)
            | ExprKind::Err => span_lint(cx, HACSPEC, expr.span, &"[HACSPEC] Unauthorized expression"),
        }
    }
}
