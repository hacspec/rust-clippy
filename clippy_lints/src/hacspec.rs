use crate::utils::*;
use rustc::hir;
use rustc::lint::{EarlyContext, EarlyLintPass, LateContext, LateLintPass, LintArray, LintPass};
use rustc::{declare_lint_pass, declare_tool_lint};
use syntax::*;
use syntax_pos::Span;

declare_clippy_lint! {
    pub HACSPEC,
    pedantic,
    "Checks whether the code belongs to the hacspec subset of Rust"
}

declare_lint_pass!(Hacspec => [HACSPEC]);

const ALLOWED_USES: &'static [&'static [&'static str]] = &[
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
    &["{{root}}", "std", "prelude", "v1"]
];

fn is_allowed(queried_use: &hir::HirVec<hir::PathSegment>) -> bool {
    ALLOWED_USES
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
        // only check top level `use` statements
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
                        "Crate used: {:?}",
                        path.segments.iter().map(|x| x.ident).collect::<Vec<ast::Ident>>()
                    ),
                )
            }
        }
    }
}

impl EarlyLintPass for Hacspec {
    fn check_expr<'tcx>(&mut self, cx: &EarlyContext<'tcx>, expr: &ast::Expr) {
        if in_macro(expr.span) {
            return ();
        };
        if let ast::ExprKind::Path(_, path) = &expr.kind {
            if path.segments.len() != 1 {
                span_lint(
                    cx,
                    HACSPEC,
                    expr.span,
                    &format!("Number of segments: {}", path.segments.len()),
                )
            }
        }
    }
}
