use std::env;

use actix_http::HttpService;
use actix_server::Server;
use actix_service::map_config;
use actix_web::{App, get, HttpResponse, post, Responder};
use actix_web::dev::AppConfig;
use log::info;
use serde_json::json;

use exp_engine::runtime;

use crate::configuration::{get_rules, init_rules};

mod configuration;
mod exp_engine;
mod rule_handlers;

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
                .service(rule_handlers::eval_rule_handler)
                .service(rule_handlers::eval_rules_handler)
                .service(index);

            HttpService::build()
                .finish(map_config(app, |_| AppConfig::default()))
                .tcp()
        })?
        .run()
        .await
}