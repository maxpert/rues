use chrono::{Utc};
use jmespatch::functions::{Function, Signature};
use jmespatch::{Context, ErrorReason, JmespathError, Rcvar, Variable};
use serde_json::Number;

pub struct NowFn {
    signature: Signature,
}

impl Default for NowFn {
    fn default() -> Self {
        NowFn::new()
    }
}

impl NowFn {
    pub fn new() -> NowFn {
        NowFn {
            signature: Signature::new(
                vec![],
                None,
            )
        }
    }
}

impl Function for NowFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let now = Utc::now();
        let ts: f64 = now.timestamp() as f64 + (now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0);
        let n = Number::from_f64(ts).ok_or(
            JmespathError::from_ctx(
                ctx,
                ErrorReason::Parse("Unable to cast timestamp".to_string())
            )
        )?;
        Ok(Rcvar::new(Variable::Number(n)))
    }
}

#[cfg(test)]
mod now_tests {
    use std::ops::Deref;
    use chrono::{TimeZone, Utc};
    use jmespatch::{Variable};
    use serde_json::json;
    use crate::runtime::compile_expr;

    #[test]
    fn test_now_returns_timestamp() {
        let exp = compile_expr("now()".to_string()).unwrap();
        let r = exp.search(json!(null)).unwrap();
        let now = Utc::now();
        match r.deref() {
            Variable::Number(n) => {
                let ts = n.as_f64().unwrap();
                let parsed_ts = Utc.timestamp(ts as i64, 0);
                // Ignore milli-seconds
                assert_eq!(parsed_ts.timestamp(), now.timestamp());
            }
            _ => assert!(false)
        }
    }
}