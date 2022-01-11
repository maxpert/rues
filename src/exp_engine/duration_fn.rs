use jmespatch::{Context, ErrorReason, JmespathError, Rcvar, Variable};
use jmespatch::functions::{ArgumentType, Function, Signature};
use serde_json::Number;

pub struct DurationFn {
    signature: Signature,
}

impl Default for DurationFn {
    fn default() -> Self {
        DurationFn::new()
    }
}

impl DurationFn {
    pub fn new() -> DurationFn {
        DurationFn {
            signature: Signature::new(
                vec![
                    ArgumentType::String
                ],
                None,
            )
        }
    }

    pub fn parse_duration(str: &str) -> Result<f64, String> {
        match parse_duration::parse(str) {
            Ok(d) => {
                let nanos = d.subsec_nanos() as f64 / 1_000_000_000.0;
                let seconds = d.as_secs() as f64;
                Ok(seconds + nanos)
            },
            Err(e) => Err(e.to_string())
        }
    }
}

impl Function for DurationFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let fmt = args[0].as_string().unwrap();
        match DurationFn::parse_duration(fmt.as_str()) {
            Ok(v) => Ok(Rcvar::new(Variable::Number(
                Number::from_f64(v).unwrap_or(Number::from(0))
            ))),
            Err(e) => Err(
                JmespathError::from_ctx(
                    ctx,
                    ErrorReason::Parse(format!("Error parsing duration: {}", e)),
                ))
        }
    }
}

#[cfg(test)]
mod duration_tests {
    use serde_json::json;
    use crate::runtime::compile_expr;

    #[test]
    fn test_duration_returns_milliseconds() {
        let exp = compile_expr("duration('1h1m100ms')".to_string()).unwrap();
        let r = exp.search(json!(null)).unwrap();
        assert!(r.is_truthy());
        assert!(r.is_number());
        assert_eq!(3_660.1, r.as_number().unwrap());
    }

    #[test]
    fn test_neg_duration_returns_error() {
        let exp = compile_expr("duration('-1ns')".to_string()).unwrap();
        let r = exp.search(json!(null));
        assert!(r.is_err());
    }
}