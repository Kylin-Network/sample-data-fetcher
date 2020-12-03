extern crate pretty_env_logger;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;

mod kylin_network_api;
type KylinNetworkAPI = kylin_network_api::KylinNetworkAPI;

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    api_name: String,
}

async fn api_list() -> Result<HttpResponse, Error> {
    let raw_content = r#"[
              "liquidation_order_list",
              "bitmex_perpetual_contract_rate",
              "bitmex_large_order_list",
              "bitfinex_holdings_minutes"
        ]"#;
    let json_content: Value = serde_json::from_str(raw_content)?;
    println!("Response: {:?}", json_content);
    println!("============================================================");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json_content))
}

async fn rpc_handler(req: web::Json<RpcRequest>) -> Result<HttpResponse, Error> {
    println!("req: {:?}", req);

    let rpc_fn_name = req.api_name.as_str();
    let resp: String;

    let api_key = match env::var("KYLIN_API_KEY") {
        Ok(val) => val,
        Err(_e) => panic!("KYLIN_API_KEY is not set"),
    };

    let api_secret = match env::var("KYLIN_API_SECRET") {
        Ok(val) => val,
        Err(_e) => panic!("KYLIN_API_SECRET is not set"),
    };

    let api = KylinNetworkAPI::new(api_key, api_secret);

    match rpc_fn_name {
        "liquidation_order_list" => {
            let params: BTreeMap<String, String> = [
                (String::from("coinName"), String::from("BTC")),
                (String::from("exchCode"), String::from("okex")),
                (String::from("type"), String::from("0")),
                (String::from("futureType"), String::from("0")),
            ]
            .iter()
            .cloned()
            .collect();

            resp = api.contract_liquidation_order_list(params).await;
        }
        "bitmex_perpetual_contract_rate" => {
            resp = api.contract_bitmex_perpetual_contract_rate().await;
        }
        "bitmex_large_order_list" => {
            resp = api.contract_bitmex_large_order_list().await;
        }
        "bitfinex_holdings_minutes" => {
            resp = api.contract_bitfinex_holdings_minutes().await;
        }
        _ => {
            // HttpResponse::BadRequest().finish()
            panic!("Unknown rpc function: {}", rpc_fn_name);
        } // TODO: Return bad request error
    }

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .service(web::resource("/api_list").route(web::get().to(api_list)))
            .service(web::resource("/").route(web::post().to(rpc_handler)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
