use cached::proc_macro::cached;
use jmespatch::{Expression, JmespathError, Runtime};
use lazy_static::lazy_static;
use crate::exp_engine::email_fn::EmailFn;

use super::regex_fn::RegexFn;

lazy_static! {
    pub static ref EXTENDED_RUNTIME: Runtime = {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        runtime.register_function("match", Box::new(RegexFn::new()));
        runtime.register_function("valid_email", Box::new(EmailFn::new()));
        runtime
    };
}

#[cached(size = 1024, result = true)]
pub fn compile_expr(expr: String) -> Result<Expression<'static>, JmespathError> {
    EXTENDED_RUNTIME.compile(expr.as_str())
}