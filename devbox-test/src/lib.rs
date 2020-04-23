extern crate proc_macro;

use std::iter::FromIterator;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, Block, Expr, FnArg, ItemFn, LitStr, Local, Pat, Result, Stmt, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Eq, Let, Semi},
};

/// This is a macro complementing Rust's standard `#[test]` macro adding test parametrization.
///
/// A test function can have any number of parameters which can have anonymouse types that will
/// be filled in by the macro based on supplied arguments.
///
/// # Test case
///
/// Macro emits a new standard Rust test for each named argument set (also called a case) by
/// suffixing function name with case name.
///
/// Cases are seperated by `;` and need to have unique names for particular test function.
/// Each case needs argument list seperated by `,` that consumes equal number of function parameters
/// when generating the actual test function.
///
/// To mark a case as one that should panic, add a suffix with a slice of expected message after `!`
///
/// Syntax for a case is ```<case-name>: <arg1>, <arg2> ... <argN> [! "<message slice>"];```
///
/// # Cartesian product
///
/// You can apply mutiple test macros to a single function with individual macro cases consuming
/// only a subset of function parameters. This forms a cartesian product of cases from each macro
/// instance. It is import that all cartesian products consume all parameters or you will end up
/// with a test function with nonzero parameters which is not supported by Rust built in test macro.
///
/// # Example
///
/// The following example have two cases named `char_a` and `char_b`
///
///     #[devbox_test::test(
///         char_a: 97, 'a';
///         char_b: 98, 'b';)]
///     #[devbox_test::test(
///         offset_0: 0;
///         offset_1: 1 ! "code incorrect";)]
///     fn parametrized_test_for(code:_, letter:_, offset:_) {
///         assert_eq!(code + offset, letter as u8, "Letter code incorrect");
///     }
///
/// Should produce:
/// ```txt
/// test parametrized_test_for__char_a__offset_0 ... ok
/// test parametrized_test_for__char_b__offset_0 ... ok
/// test parametrized_test_for__char_a__offset_1 ... ok
/// test parametrized_test_for__char_b__offset_1 ... ok
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    let cases = parse_macro_input!(attr as Cases);
    let input = parse_macro_input!(input as ItemFn);

    let mut output = quote!{};
    for case in cases.0 {
        let should_panic = case.panics.clone().map(|e| quote!{ #[should_panic(expected = #e)] });
        let func = make_case(&input, case);
        let test = if func.sig.inputs.len() == 0 || !has_test_attribute(&func) {
            Some(quote!{ #[test] }  )
        } else {
            None
        };

        output.extend(quote!{
            #test
            #should_panic
            #func
        });
    }

    output.into()
}

fn has_test_attribute(func: &ItemFn) -> bool {
    func.attrs.iter().any(|a| a.path.segments.last().map_or(false, |seg|seg.ident=="test"))
}

fn make_case(input: &ItemFn, case: Case) -> ItemFn {
    if case.values.len() > input.sig.inputs.len() {
        emit_error!(
            input,
            "Devbox: Test case '{}' arguments outnumber function '{}' parameters {} to {}",
            case.ident, input.sig.ident, case.values.len(), input.sig.inputs.len()
        );
    }

    let mut func = input.clone();
    let name = format!("{}__{}", func.sig.ident, case.ident.to_string());
    func.sig.ident = Ident::new(name.as_ref(), Span::call_site());

    let inputs = func.sig.inputs.clone();
    let mut args = inputs.iter().map(|t|t.clone());
    for expr in case.values {
        if let Some(arg) = args.next() {
            insert_param(&mut func.block, arg, expr);
        }
    }
    func.sig.inputs = syn::punctuated::Punctuated::from_iter(args);

    func
}

fn insert_param(block: &mut Box<Block>, arg: FnArg, init:Box<Expr>){
    match arg {
        FnArg::Typed(arg) => {
            block.stmts.insert(0, Stmt::Local(Local {
                attrs: vec![],
                let_token: Let { span: Span::call_site() },
                pat: Pat::Type(arg),
                init: Some((Eq{ spans: [Span::call_site()] }, init)),
                semi_token: Semi { spans: [Span::call_site()] },
            }))
        },
        FnArg::Receiver(_) => emit_error!(
            arg,
            "Devbox: Parametrized test applied to non-associated function"
        )
    }
}

struct Case {
    pub ident: Ident,
    pub colon: Token![:],
    pub values: Vec<Box<Expr>>,
    pub panics: Option<LitStr>,
}

impl Parse for Case {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Case {
            ident: input.parse()?,
            colon: input.parse()?,
            values: {
                let mut result = vec![Box::new(input.parse()?)];
                let mut more: Option<Token![,]> = input.parse()?;
                while more.is_some() {
                    result.push(Box::new(input.parse()?));
                    more = input.parse()?;
                }
                result
            },
            panics: {
                let excl: Option<Token![!]> = input.parse()?;
                if excl.is_some() {
                    input.parse()?
                } else {
                    None
                }
            }
        })
    }
}

struct Cases(pub Punctuated<Case, Token![;]>);

impl Parse for Cases {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Cases(input.parse_terminated(Case::parse)?))
    }
}