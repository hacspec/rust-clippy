use crate::utils::span_lint;
use rustc_lint::{EarlyLintPass, EarlyContext};
use rustc_session::{declare_lint_pass, declare_tool_lint};
// use rustc_span::Span;
use rustc_ast::{
    ast::{MacCall, PathSegment},
};

declare_clippy_lint! {
    /// **What it does:**
    ///
    /// **Why is this bad?**
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// // example code
    /// ```
    pub HACSPEC_MACROS,
    nursery, //pedantic, corectness or restriction, but shouldn't interfere with other lints ? or shipped completely separately
    "default lint description"
}

declare_lint_pass!(HacspecMacros => [HACSPEC_MACROS]);

impl EarlyLintPass for HacspecMacros {
    fn check_mac(&mut self, _cx: &EarlyContext<'_>, mac: &MacCall) {
    }
}
