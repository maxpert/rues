use cached::proc_macro::cached;
use jmespatch::{Context, ErrorReason, JmespathError, Rcvar, Variable};
use jmespatch::functions::{ArgumentType, Function, Signature};
use regex::{Match, Regex};

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
                vec![ArgumentType::String, ArgumentType::String],
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
}

impl Function for RegexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let regex_str = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                "",
                0,
                ErrorReason::Parse("Expression argument should be a string".to_owned()),
            )
        })?;

        let regex = compile_regex(regex_str.to_owned())?;
        let payload = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                "",
                0,
                ErrorReason::Parse("Payload must be a string".to_owned()),
            )
        })?;

        Ok(Rcvar::new(Self::match_regex(regex, payload)))
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
    use serde_json::json;

    use crate::exp_engine::runtime::compile_expr;

    #[test]
    fn test_match_returns_matched_segments_in_regex() {
        let exp = compile_expr("match('foo', a.b)".to_string()).unwrap();
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
        let exp = compile_expr("match('foo', a.b)".to_string()).unwrap();
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
        let exp = compile_expr(r"match('(\d{4})-(\d{2})-(\d{2})', a)".to_string()).unwrap();
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
}