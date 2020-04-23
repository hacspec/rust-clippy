use crate::utils::span_lint;
use rustc_lint::{EarlyLintPass, EarlyContext};
use rustc_session::{impl_lint_pass, declare_tool_lint};
use rustc_ast::{
    ast::{MacCall, PathSegment},
    tokenstream::TokenTree,
    token::{Token, TokenKind},
};

declare_clippy_lint! {
    /// **What it does:** Hacpsec sub-language : checks macros invocations and adds type declarations to the authorized types
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
    hacspec_lang,
    ""
}

#[derive(Default)]
pub struct HacspecMacros {
    added_macros: Vec<Vec<String>>,
}

impl_lint_pass!(HacspecMacros => [HACSPEC_MACROS]);



const ALLOWED_MACRO_TYPE_DECL: &[&[&str]] = &[
    &["array"],
    &["bytes"],
    &["public_bytes"],
    &["public_array"],
    &["poly"],
    &["nat_mod"],
    &["public_nat_mod"],
    &["abstract_unsigned_public_integer"],
];

const ALLOWED_MACROS_MISC : &[&[&str]] = &[
    &["concat"],
    &["secret_array"],
    &["secret_bytes"],
    &["assert"],
    &["assert_ne"],
    &["debug_assert"],
    &["assert_bytes_eq"],
    &["assert_eq"],
    &["assert_secret_array_eq"],

    &["println"],
];

fn allowed_path(authorized_macros: &[&[&str]], queried_use: &[PathSegment]) -> bool {
    authorized_macros.iter().any(|&allowed_use| {
        allowed_use
            .iter()
            .zip(queried_use.iter())
            .filter(|(&allowed_segment, queried_segment)| *allowed_segment == *queried_segment.ident.name.as_str())
            .count()
            == allowed_use.len()
    })
}

// same as above except for argument, not really nice for code reuse
impl HacspecMacros {
    fn check_added_macro(&self, queried_use: &[PathSegment]) -> bool {
        self.added_macros.iter().any(|allowed_use| {
            allowed_use
                .iter()
                .zip(queried_use.iter())
                .filter(|(allowed_segment, queried_segment)| *allowed_segment.as_str() == *queried_segment.ident.name.as_str())
                .count()
                == allowed_use.len()
        })
    }
}

impl EarlyLintPass for HacspecMacros {
    fn check_mac(&mut self, cx: &EarlyContext<'_>, mac: &MacCall) {
        if allowed_path(ALLOWED_MACRO_TYPE_DECL, &mac.path.segments) {
            let mut flag = false;
            let _ = &mac.args.inner_tokens().map_enumerated(
                |i, tk_tr| {
                    if i == 0 {
                        if let TokenTree::Token( Token { kind:TokenKind::Ident(name, _b), span: _ }) = tk_tr {
                            name.with(
                                |s| {   // if self.added_macros.contains(String::from(s)) {
                                        //     span_lint(cx, HACSPEC_MACROS, mac.span(), &format!("Already declared type {}", type_decl)) }
                                        // shouldn't be possible, macro expansion would cause an error, identifier already used or something like this
                                        self.added_macros.push(vec![String::from(s)]);
                                        flag = true;
                                    }
                            );
                        }
                    };  tk_tr }
            );
            if flag {
                return;
            } else { span_lint(cx, HACSPEC_MACROS, mac.span(), "FORBIDDEN MACRO") }
        }
        if allowed_path(ALLOWED_MACROS_MISC, &mac.path.segments)
        ||  self.check_added_macro(&mac.path.segments) { return; }

        span_lint(
            cx,
            HACSPEC_MACROS,
            mac.span(),
            "FORBIDDEN MACRO",
        )
    }
}