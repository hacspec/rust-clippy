use crate::utils::*;
use rustc::hir;
use rustc::lint::{in_external_macro, LateContext, LateLintPass, LintArray, LintContext, LintPass};
use rustc::{declare_lint_pass, declare_tool_lint};
use syntax::*;
use syntax_pos::{
    hygiene::{DesugaringKind, ExpnKind},
    Span,
};

declare_clippy_lint! {
    pub HACSPEC,
    pedantic,
    "Checks whether the code belongs to the hacspec subset of Rust"
}

declare_lint_pass!(Hacspec => [HACSPEC]);

const ALLOWED_PATHS: &'static [&'static [&'static str]] = &[
    &["hacspec"],
    &["num"],
    &["std"],
    &["std", "num", "ParseIntError"],
    &["std", "ops"],
    &["std", "cmp", "min"],
    &["std", "cmp", "PartialEq"],
    &["std", "fmt"],
    &["uint"],
    &["uint", "natmod_p"],
    &["uint", "traits"],
    &["uint", "uint_n"],
    &["wrapping_arithmetic", "wrappit"],
    &["{{root}}", "std", "prelude", "v1"],
    &["{{root}}", "std", "iter", "Iterator", "next"],
    &["{{root}}", "std", "iter", "IntoIterator", "into_iter"],
    &["{{root}}", "std", "option", "Option"],
    &["{{root}}", "std", "ops", "Range"],
];

fn is_allowed(queried_use: &hir::HirVec<hir::PathSegment>) -> bool {
    ALLOWED_PATHS
        .iter()
        .find(|&allowed_use| {
            allowed_use
                .iter()
                .zip(queried_use.iter())
                .filter(|(&allowed_segment, queried_segment)| *allowed_segment == *queried_segment.ident.name.as_str())
                .count()
                == allowed_use.len()
        })
        .is_some()
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Hacspec {
    fn check_path(&mut self, cx: &LateContext<'a, 'tcx>, path: &'tcx hir::Path, _: hir::HirId) {
        // Items used in the code are whitelisted
        if in_macro(path.span) || in_external_macro(cx.sess(), path.span) {
            return ();
        };
        if path.segments.len() == 1 || is_allowed(&path.segments) {
            // Paths of len 1 correspond to items inside the crate, except when used in imports
            return ();
        };
        span_lint(cx, HACSPEC, path.span, &format!("[HACSPEC] Unauthorized item {}", path))
    }

    fn check_mod(&mut self, cx: &LateContext<'a, 'tcx>, m: &'tcx hir::Mod, span: Span, _: hir::HirId) {
        let map = cx.tcx.hir();
        // only check top level `use` statements
        for item in &m.item_ids {
            let item = map.expect_item(item.id);
            if in_macro(span) {
                continue;
            };
            if let hir::ItemKind::Use(ref path, _) = item.kind {
                // Even though the check_path also checks the import path, we have to restricts
                // imports path of lenght 1 that are not allowed
                if is_allowed(&path.segments) {
                    continue;
                };
                span_lint(cx, HACSPEC, item.span, &format!("Unauthorized item {}", path))
            }
        }
    }

    fn check_param(&mut self, cx: &LateContext<'a, 'tcx>, param: &'tcx hir::Param) {
        if in_macro(param.span) {
            return ();
        };
        // Function parameters cannot be borrowed
        param.pat.walk(|pat: &hir::Pat| match &pat.kind {
            hir::PatKind::Binding(binding_annot, _, _, _) => {
                if *binding_annot != hir::BindingAnnotation::Unannotated {
                    span_lint(
                        cx,
                        HACSPEC,
                        pat.span,
                        &format!("[HACSPEC] Cannot annotate function parameter with mutability"),
                    );
                    return false;
                }
                true
            },
            _ => true,
        })
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        _: hir::intravisit::FnKind<'tcx>,
        sig: &'tcx hir::FnDecl,
        _: &'tcx hir::Body,
        span: Span,
        _: hir::HirId,
    ) {
        if in_macro(span) {
            return ();
        };
        // The types of function parameters cannot be references
        for param in sig.inputs.iter() {
            match &param.kind {
                hir::TyKind::Ptr(_) | hir::TyKind::Rptr(_, _) => span_lint(
                    cx,
                    HACSPEC,
                    param.span,
                    &format!("[HACSPEC] Function parameters cannot be references"),
                ),
                _ => (),
            }
        }
    }

    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &hir::Expr) {
        if in_macro(expr.span) || in_external_macro(cx.sess(), expr.span) {
            return ();
        }
        // Restricts the items used to the whitelist
        match &expr.kind {
            hir::ExprKind::AddrOf(_, _) => {
                if expr.span.from_expansion() {
                    match expr.span.ctxt().outer_expn_data().kind {
                        // Loop ranges are desugared with an mutable addrOf expression so we
                        // authorize them
                        ExpnKind::Desugaring(DesugaringKind::ForLoop) => (),
                        _ => span_lint(cx, HACSPEC, expr.span, &format!("[HACSPEC] Unauthorized expression")),
                    }
                }
            },
            hir::ExprKind::Binary(_, _, _) => (),
            hir::ExprKind::Unary(_, _) => (),
            hir::ExprKind::Lit(_) => (),
            hir::ExprKind::Type(_, _) => (),
            hir::ExprKind::Loop(_, _, _) => (),
            hir::ExprKind::Match(_, _, _) => (),
            hir::ExprKind::Assign(_, _) => (),
            hir::ExprKind::AssignOp(_, _, _) => (),
            hir::ExprKind::Break(_, _) => (),
            hir::ExprKind::Array(_)
            | hir::ExprKind::Call(_, _)
            | hir::ExprKind::MethodCall(_, _, _)
            | hir::ExprKind::Tup(_)
            | hir::ExprKind::Block(_, _)
            | hir::ExprKind::Field(_, _)
            | hir::ExprKind::Index(_, _)
            | hir::ExprKind::Struct(_, _, _)
            | hir::ExprKind::DropTemps(_)
            | hir::ExprKind::Path(_)
            | hir::ExprKind::Ret(_) => (),
            hir::ExprKind::Cast(_, _)
            | hir::ExprKind::Box(_)
            | hir::ExprKind::Closure(_, _, _, _, _)
            | hir::ExprKind::Continue(_)
            | hir::ExprKind::InlineAsm(_, _, _)
            | hir::ExprKind::Repeat(_, _)
            | hir::ExprKind::Yield(_, _)
            | hir::ExprKind::Err => span_lint(cx, HACSPEC, expr.span, &format!("[HACSPEC] Unauthorized expression")),
        }
    }
}
