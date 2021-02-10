extern crate pretty_env_logger;

use chrono::Utc;
use uuid::Uuid;
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

#[derive(Debug, Serialize, Deserialize)]
struct ApiLog {
    request_id: String,
    service_name: String,
    source: String,
    url_path: String,
    url_query: String,
    request_method: String,
    request_body: String,
    request_time: String,
    response_time: String,
    response_content: String,
}

async fn save_data_to_es(api_log: ApiLog) {
    let client = reqwest::Client::new();

    let es_host = match env::var("KYLIN_ES_HOST") {
        Ok(val) => val,
        Err(_e) => String::from("localhost:9200"),
    };

    let es_index_name = match env::var("KYLIN_ES_INDEX_NAME") {
        Ok(val) => val,
        Err(_e) => String::from("kylin_access_tracking"),
    };

    let full_es_endpoint = format!("http://{}/{}/_doc/", es_host, es_index_name);
    let resp = match client.post(full_es_endpoint.as_str())
        .json(&api_log)
        .send()
        .await{
        Ok(resp) => resp,
        Err(e) => panic!("Es save err: {}", e),
    };

    println!("api log: {:?}", api_log);
    println!("ES host: {}, resp: {:?}", full_es_endpoint, resp);
    println!("Resp content: {:?}", resp.text().await);
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

    let mut api_log = ApiLog{
        request_id: Uuid::new_v4().to_string(),
        service_name: String::from(rpc_fn_name),
        source: "".to_string(),
        url_path: String::from("/"),
        url_query: "".to_string(),
        request_method: "POST".to_string(),
        request_body: "".to_string(),
        request_time: Utc::now().timestamp_millis().to_string(),
        response_time: "".to_string(),
        response_content: "".to_string(),
    };

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

            api_log.request_body = serde_json::to_value(params.clone()).unwrap().to_string();
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
    api_log.response_time = Utc::now().timestamp_millis().to_string();
    api_log.response_content = resp.clone();
    save_data_to_es(api_log).await;

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
            .wrap(middleware::Logger::default())
            .service(web::resource("/api_list").route(web::get().to(api_list)))
            .service(web::resource("/").route(web::post().to(rpc_handler)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
