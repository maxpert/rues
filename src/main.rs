use std::env;

use actix_http::HttpService;
use actix_server::Server;
use actix_service::map_config;
use actix_web::{App, get, HttpResponse, post, Responder, web};
use actix_web::dev::AppConfig;
use jmespatch::{JmespathError, Rcvar};
use log::info;
use serde_json::{json, Value};

use exp_engine::runtime;

use crate::configuration::{get_rules, init_rules};

mod configuration;
mod exp_engine;

fn evaluate_rule(expression: &String, value: &Value) -> Result<Rcvar, JmespathError> {
    runtime::compile_expr(expression.clone())?.search(value)
}

#[post("/eval/{rule_name}")]
async fn eval_rule(
    web::Path(rule): web::Path<String>,
    web::Json(ctx): web::Json<Value>,
) -> impl Responder {
    let expression = {
        get_rules().get_str(rule.as_str())
    };

    if expression.is_err() {
        return HttpResponse::NotFound().body(format!("Rule {} not found", rule));
    }

    let expression_string = expression.unwrap();
    let result = evaluate_rule(&expression_string, &ctx)
        .map(|expression_value| {
            json!({
                "rule": rule,
                "expression": expression_string,
                "is_truthy": expression_value.is_truthy(),
                "exp_value": expression_value
            })
        });

    return match result {
        Ok(output) => HttpResponse::Ok().body(output.to_string()),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string())
    };
}

#[get("/")]
async fn index() -> impl Responder {
    let body = serde_json::to_string(
        &json!({"status": true})
    ).unwrap();

    return HttpResponse::Ok()
        .content_type("application/json")
        .body(body);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| {
        "0.0.0.0:8080".to_string()
    });
    env_logger::init();

    info!("Loading rules...");
    init_rules();

    Server::build()
        .bind("rudo", bind_address.clone(), || {
            let app = App::new()
                .service(eval_rule)
                .service(index);

            HttpService::build()
                .finish(map_config(app, |_| AppConfig::default()))
                .tcp()
        })?
        .run()
        .await
}