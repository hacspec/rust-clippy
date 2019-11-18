use crate::utils::*;
use rustc::hir;
use rustc::lint::{LateContext, LateLintPass, LintArray, LintPass};
use rustc::{declare_lint_pass, declare_tool_lint};
use syntax::*;
use syntax_pos::Span;

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
    fn check_mod(&mut self, cx: &LateContext<'a, 'tcx>, m: &'tcx hir::Mod, span: Span, _: hir::HirId) {
        let map = cx.tcx.hir();
        // Ensures an import whitelist
        for item in &m.item_ids {
            let item = map.expect_item(item.id);
            if in_macro(span) {
                continue;
            };
            if let hir::ItemKind::Use(ref path, _) = item.kind {
                if is_allowed(&path.segments) {
                    continue;
                };
                span_lint(
                    cx,
                    HACSPEC,
                    item.span,
                    &format!(
                        "[HACSPEC] Cannot use crate {:#?}",
                        path.segments.iter().map(|x| x.ident).collect::<Vec<ast::Ident>>()
                    ),
                )
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
        if in_macro(expr.span) {
            return ();
        };
        // Restricts the items used to the whitelist
        match &expr.kind {
            hir::ExprKind::Path(hir::QPath::Resolved(_, ref path)) => {
                if path.segments.len() != 1 && !is_allowed(&path.segments) {
                    span_lint(
                        cx,
                        HACSPEC,
                        expr.span,
                        &format!("Number of segments: {:?}", path.segments),
                    )
                }
            },
            _ => (),
        }
    }
}
