use jmespatch::functions::{ArgumentType, Function, Signature};
use jmespatch::{Context, JmespathError, Rcvar, Variable};

pub struct EmailFn {
    signature: Signature,
}

impl Default for EmailFn {
    fn default() -> Self {
        EmailFn::new()
    }
}

impl EmailFn {
    pub fn new() -> EmailFn {
        EmailFn {
            signature: Signature::new(
                vec![
                    ArgumentType::Any
                ],
                None,
            )
        }
    }
}

impl Function for EmailFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) ->  Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        if args[0].is_string() {
            let res = mailchecker::is_valid(args[0].as_string().unwrap());
            return Ok(Rcvar::new(Variable::Bool(res)));
        }

        return Ok(Rcvar::new(Variable::Bool(false)));
    }
}


#[cfg(test)]
mod email_tests {
    use serde_json::json;
    use crate::runtime::compile_expr;

    #[test]
    fn test_is_valid_email_returns_true_for_good_emails() {
        let exp = compile_expr("valid_email('therustyguy158@gmail.com')".to_string()).unwrap();
        let r = exp.search(json!(null)).unwrap();
        assert!(r.is_truthy());
    }

    #[test]
    fn test_is_valid_email_returns_false_for_junk_emails() {
        let exp = compile_expr("valid_email('foo@guerrillamailblock.com')".to_string()).unwrap();
        let r = exp.search(json!(null)).unwrap();
        assert!(!r.is_truthy());
    }

    #[test]
    fn test_is_valid_email_returns_false_for_non_strings() {
        let exp = compile_expr("valid_email(a)".to_string()).unwrap();
        let r = exp.search(json!({
            "a": 5
        })).unwrap();
        assert!(!r.is_truthy());
    }
}