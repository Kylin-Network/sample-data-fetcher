extern crate pretty_env_logger;

use chrono::prelude::*;
use hmac::{Hmac, Mac, NewMac};
use log::debug;
use reqwest::StatusCode;
use serde_json::Value;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::error::Error;

type HmacSha256 = Hmac<Sha256>;

pub struct KylinNetworkAPI {
    api_key: String,
    api_secret: String,
    base: String,
    client: reqwest::Client,
}

impl KylinNetworkAPI {
    pub fn new(api_key: String, api_secret: String) -> KylinNetworkAPI {
        KylinNetworkAPI {
            api_key: api_key,
            api_secret: api_secret,
            base: String::from("https://api.kylin.network"),
            client: reqwest::Client::new(),
        }
    }

    fn signature(&self, params: &BTreeMap<String, String>) -> String {
        let sorted_params: Vec<String> =
            params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        let str_to_be_signed = sorted_params.join("&");
        debug!("String to be signed: {}", str_to_be_signed);
        debug!("Secret: {}", self.api_secret);

        let mut mac = HmacSha256::new_varkey(self.api_secret.as_bytes()).unwrap();
        mac.update(str_to_be_signed.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        hex::encode(code_bytes)
    }

    async fn call_api(
        &self,
        api_method: &str,
        api_url: &String,
        api_params: &BTreeMap<String, String>,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        // TODO: Find how to add log to actix_web
        println!("============================================================");
        println!("API path: {}, api_params: {:?}", api_url, api_params);

        let mut _api_params = api_params.clone();

        // Insert required fields
        _api_params.insert(
            String::from("timestamp"),
            Utc::now().timestamp_millis().to_string(),
        );
        let signature = self.signature(&_api_params);
        _api_params.insert(String::from("signature"), signature);

        let resp: reqwest::Response;
        match api_method {
            "POST" => {
                resp = self
                    .client
                    .post(api_url)
                    .header("APIKEY", self.api_key.as_str())
                    .json(&_api_params)
                    .send()
                    .await?;
            }
            _ => {
                panic!("Unknown api_method: {}", api_method);
            }
        }

        match resp.status() {
            StatusCode::OK => {
                let raw_content = resp.text().await?;
                let json_content: Value = serde_json::from_str(raw_content.as_str())?;
                println!("Response: {:?}", raw_content);
                println!("============================================================");
                Ok(json_content)
            }
            _ => {
                panic!("Got error response: {:?}", resp);
            }
        }
    }

    // POST /data/liquidation
    pub async fn contract_liquidation_order_list(
        &self,
        invoke_params: BTreeMap<String, String>,
    ) -> String {
        let api_path = String::from("/data/liquidation");

        // Process params
        let mut api_params = invoke_params.clone();

        // Validate required params are existing
        if !api_params.contains_key(&String::from("exchCode"))
            || !api_params.contains_key(&String::from("type"))
        {
            panic!("exchCode or type are required");
        }

        // Update params default value
        api_params
            .entry(String::from("coinName"))
            .or_insert(String::from("BTC"));
        api_params
            .entry(String::from("pageNum"))
            .or_insert(String::from("1"));
        api_params
            .entry(String::from("pageSize"))
            .or_insert(String::from("10"));

        let full_api_endpoint = format!("{}{}", self.base, api_path);

        let resp_content = match self.call_api("POST", &full_api_endpoint, &api_params).await {
            Ok(r) => r,
            Err(e) => panic!("Call API error: {:?}", e),
        };

        resp_content["data"].to_string()
    }

    pub async fn contract_bitmex_perpetual_contract_rate(&self) -> String {
        let api_path = String::from("/data/getContractRate/XBTUSD");
        let full_api_endpoint = format!("{}{}", self.base, api_path);
        let api_params: BTreeMap<String, String> = BTreeMap::new();
        let resp_content = match self.call_api("POST", &full_api_endpoint, &api_params).await {
            Ok(r) => r,
            Err(e) => panic!("Call API error: {:?}", e),
        };

        resp_content["data"].to_string()
    }

    pub async fn contract_bitmex_large_order_list(&self) -> String {
        let api_path = String::from("/data/largeDeal");
        let full_api_endpoint = format!("{}{}", self.base, api_path);
        let api_params: BTreeMap<String, String> = BTreeMap::new();
        let resp_content = match self.call_api("POST", &full_api_endpoint, &api_params).await {
            Ok(r) => r,
            Err(e) => panic!("Call API error: {:?}", e),
        };

        resp_content["data"].to_string()
    }

    pub async fn contract_bitfinex_holdings_minutes(&self) -> String {
        let api_path = String::from("/data/getBitfinexPositionRatio/minute");
        let full_api_endpoint = format!("{}{}", self.base, api_path);
        let api_params: BTreeMap<String, String> = BTreeMap::new();
        let resp_content = match self.call_api("POST", &full_api_endpoint, &api_params).await {
            Ok(r) => r,
            Err(e) => panic!("Call API error: {:?}", e),
        };

        resp_content["data"].to_string()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn signature() {
        // Case from document @2020-11-11
        // url: https://docs-api.kylin.network/#example
        pretty_env_logger::init();
        let _api = KylinNetworkAPI::new(
            String::from(""),
            String::from("CHK5kxIQtd4WWkK8th8mBwctKF55vIEBztJ7KMnI6oniR9Rhlb1JB2WyWOhLG2GQ"),
        );

        let api_params: BTreeMap<String, String> = [
            (String::from("coinName"), String::from("BTC")),
            (String::from("timestamp"), String::from("1603271977470")),
            (String::from("exchCode"), String::from("okex")),
            (String::from("pageSize"), String::from("10")),
            (String::from("pageNum"), String::from("1")),
            (String::from("futureType"), String::from("0")),
            (String::from("type"), String::from("0")),
        ]
        .iter()
        .cloned()
        .collect();

        let sign_rslt = _api.signature(&api_params);

        assert_eq!(
            sign_rslt,
            "7e22854ab87ee121ec5d0675f5f0b9a4c74135f021d60771c37006c60e0674c6"
        );
    }
}
