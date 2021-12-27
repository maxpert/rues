use actix_web::{HttpResponse, Responder, web};
use jmespatch::{JmespathError, Rcvar};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{get_rules, post, runtime};

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchEvalReq {
    rules: Vec<String>,
    context: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RuleEvalRes {
    Success {
        name: String,
        expression: String,
        is_truthy: bool,
        value: Rcvar,
    },
    Error {
        name: String,
        expression: String,
        reason: String,
    },
    NotFound {
        name: String
    },
}

fn evaluate_rule(expression: &String, value: &Value) -> Result<Rcvar, JmespathError> {
    runtime::compile_expr(expression.to_owned())?.search(value)
}

fn eval_rules(ctx: Value, rule_names: Vec<String>) -> Vec<RuleEvalRes> {
    let mut expressions: Vec<RuleEvalRes> = vec![];
    let cfg = get_rules();
    for name in rule_names {
        let exp_res = cfg.get_str(name.as_str());

        if exp_res.is_err() {
            expressions.push(RuleEvalRes::NotFound {
                name: name.clone(),
            });

            continue;
        }

        let expression = exp_res.unwrap();
        let result = evaluate_rule(&expression, &ctx);

        expressions.push(match &result {
            Ok(r) => RuleEvalRes::Success {
                is_truthy: r.is_truthy(),
                value: r.clone(),
                expression,
                name,
            },
            Err(e) => RuleEvalRes::Error {
                reason: e.to_string(),
                name,
                expression,
            }
        })
    }

    return expressions;
}

#[post("/eval")]
pub async fn eval_rules_handler(
    web::Json(batch): web::Json<BatchEvalReq>
) -> impl Responder {
    HttpResponse::Ok().json(
        eval_rules(batch.context, batch.rules)
    )
}

#[post("/eval/{rule_name}")]
pub async fn eval_rule_handler(
    web::Path(rule): web::Path<String>,
    web::Json(ctx): web::Json<Value>,
) -> impl Responder {
    HttpResponse::Ok().json(
        eval_rules(ctx, vec![rule]).first()
    )
}
