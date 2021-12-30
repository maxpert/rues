use cached::proc_macro::cached;
use jmespatch::{Context, ErrorReason, JmespathError, Rcvar, Variable};
use jmespatch::ast::Ast;
use jmespatch::functions::{ArgumentType, Function, Signature};
use regex::{Match, Regex};

const ERROR_MSG: &str = "Regex must be a literal string please only use &'...'";

pub struct RegexFn {
    signature: Signature,
}

impl Default for RegexFn {
    fn default() -> Self {
        RegexFn::new()
    }
}

impl RegexFn {
    pub fn new() -> RegexFn {
        RegexFn {
            signature: Signature::new(
                vec![
                    ArgumentType::Any,
                    ArgumentType::Any,
                ],
                None,
            )
        }
    }

    fn match_regex(regex: Regex, payload: &String) -> Variable {
        regex.captures(payload).map(|capture| {
            let matches: Vec<Rcvar> = capture.iter().map(|match_group| {
                let mg: Vec<Rcvar> = match_group.ok_or_else(|| {
                    vec![] as Vec<Match>
                }).iter().map(|matched| {
                    let var = Variable::String(matched.as_str().to_string());
                    Rcvar::new(var)
                }).collect();

                if mg.len() == 1 {
                    mg.first().unwrap().to_owned()
                } else {
                    Rcvar::new(Variable::Array(mg))
                }
            }).collect();

            Variable::Array(matches)
        }).unwrap_or(Variable::Null)
    }

    fn string_literal<'s>(var: &'s Rcvar, ctx: &Context<'_>) -> Result<&'s str, JmespathError> {
        return match var.as_expref() {
            Some(Ast::Literal { ref value, .. }) => {
                match value.as_string() {
                    Some(str) => Ok(str.as_str()),
                    _ => Err(JmespathError::from_ctx(
                        ctx,
                        ErrorReason::Parse(ERROR_MSG.to_owned()),
                    ))
                }
            }
            _ => {
                Err(JmespathError::from_ctx(
                    ctx,
                    ErrorReason::Parse(ERROR_MSG.to_owned()),
                ))
            }
        };
    }
}

impl Function for RegexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let re_str = Self::string_literal(&args[0], ctx)?;
        let regex = compile_regex(re_str.to_owned())?;

        return match args[1].as_string() {
            Some(payload) => Ok(Rcvar::new(Self::match_regex(regex, payload))),
            None => Ok(Rcvar::new(Variable::Null))
        };
    }
}


#[cached(size = 1024, result = true)]
fn compile_regex(regex_str: String) -> Result<Regex, JmespathError> {
    Regex::new(regex_str.as_str()).map_err(|e| {
        JmespathError::new(
            regex_str.as_str(),
            0,
            ErrorReason::Parse(e.to_string().to_owned()),
        )
    })
}

#[cfg(test)]
mod regex_tests {
    use jmespatch::{ErrorReason, JmespathError};
    use serde_json::json;

    use crate::exp_engine::regex_fn::ERROR_MSG;
    use crate::exp_engine::runtime::compile_expr;

    #[test]
    fn test_match_returns_matched_segments_in_regex() {
        let exp = compile_expr("match(&'foo', a.b)".to_string()).unwrap();
        let r = exp.search(json!({
            "a": {
                "b": "food"
            }
        })).unwrap();
        assert!(r.is_truthy());

        let arr = r.as_array().unwrap();
        assert!(arr.len() > 0);
        assert_eq!(arr[0].as_string().unwrap(), "foo");
    }

    #[test]
    fn test_match_returns_null_unmatched_segments_in_regex() {
        let exp = compile_expr("match(&'foo', a.b)".to_string()).unwrap();
        let r = exp.search(json!({
            "a": {
                "b": "bar"
            }
        })).unwrap();
        assert!(!r.is_truthy());
        assert!(r.is_null());
    }

    #[test]
    fn test_match_returns_null_when_element_is_null() {
        let exp = compile_expr("match(&'foo', a.c)".to_string()).unwrap();
        let r = exp.search(json!({
            "a": {
                "b": "bar"
            }
        })).unwrap();
        assert!(!r.is_truthy());
        assert!(r.is_null());
    }

    #[test]
    fn test_match_returns_multiple_segments() {
        let exp = compile_expr(r"match(&'(\d{4})-(\d{2})-(\d{2})', a)".to_string()).unwrap();
        let r = exp.search(json!({
            "a": "2010-03-14"
        })).unwrap();
        assert!(r.is_truthy());

        let matches = r.as_array().unwrap();
        assert_eq!(matches[0].as_string().unwrap(), "2010-03-14");
        assert_eq!(matches[1].as_string().unwrap(), "2010");
        assert_eq!(matches[2].as_string().unwrap(), "03");
        assert_eq!(matches[3].as_string().unwrap(), "14");
    }

    #[test]
    fn test_match_wont_compile_non_literal_regex() {
        let exp = compile_expr(r"match(a, a)".to_string()).unwrap();
        let r = exp.search(json!({
            "a": "2010-03-14"
        }));

        match r.err() {
            Some(
                JmespathError {
                    reason: ErrorReason::Parse(ref reason),
                    ..
                }
            ) => assert_eq!(reason, ERROR_MSG),
            _ => assert!(false),
        };
    }

    #[test]
    fn test_match_wont_compile_non_ref_literal_regex() {
        let exp = compile_expr(r"match('a', a)".to_string()).unwrap();
        let r = exp.search(json!({
            "a": "2010-03-14"
        }));

        match r.err() {
            Some(
                JmespathError {
                    reason: ErrorReason::Parse(ref reason),
                    ..
                }
            ) => assert_eq!(reason, ERROR_MSG),
            _ => assert!(false),
        };
    }
}