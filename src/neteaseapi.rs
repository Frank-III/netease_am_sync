use crate::errors;
use chrono::Local;
use reqwest::{Request, Response};
use serde_json::Value;
use std::error;
use std::sync::Arc;
use std::{thread, time::Duration};
use tokio::sync::RwLock;
// TODO: watch the great build video about how login qr check could update the cookies
#[derive(Debug)]
struct NeteaseApi {
    client: reqwest::Client,
    base_uri: String,
    //headers: reqwest::header::HeaderMap,
    cookies: Option<String>, //seems to work
    uid: Option<i64>,
}

impl NeteaseApi {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_uri: std::env::var("NETEASE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4000".to_string()),
            cookies: std::env::var("NETEASE_COOKIES").ok(),
            uid: None,
            //headers: reqwest::headers::HeaderMap::new(),
        }
    }
    async fn gen_qr_code(&self) -> Result<Response, reqwest::Error> {
        self.client
            .get(format!("{}login/qr/key", self.base_uri))
            .query(&vec![(
                "timestamp",
                Local::now().timestamp_millis().to_string(),
            )])
            .send()
            .await
    }

    async fn login_qr_check(&self, key: &str) -> Result<String, errors::NeteaseCallError> {
        let timestamp = Local::now().timestamp_millis();
        let Ok(response) = self.client
        .get(format!(
            "{}/login/qr/check?timestamp={}&key={}",
            self.base_uri, timestamp, key
        ))
        .send()
        .await else {
            return Err(errors::NeteaseCallError::ClientFailError(502));
        };

        let Ok(serialize) = response.json::<Value>().await else {
        return Err(errors::NeteaseCallError::ParseError("failed to parse the response".to_string()));
    };

        match serialize.get("code").unwrap().as_i64() {
            Some(801) => Err(errors::NeteaseCallError::QrCodeError(
                "QRCode is outdated".to_string(),
            )),
            Some(802) => Err(errors::NeteaseCallError::QrCodeError(
                "authorize on your phone!".to_string(),
            )),
            Some(803) => {
                println!("Qrcode succeed!");
                let cookies = serialize.get("cookie").unwrap().as_str();
                // let cookie_map = get_cookies(cookies.unwrap().to_string());
                // println!("the cookie is {:#?}", cookies);
                // println!("the cookie_map is {:#?}", cookie_map);
                Ok(cookies.unwrap().to_string())
            }
            // Some(200) => {
            //     println!("Qrcode succeed!");
            //     println!("the cookie is {:#?}", serialize.get("cookie").unwrap());
            //     return true;
            // }
            Some(_) => Err(errors::NeteaseCallError::ParseError(
                "unrecognized code".to_string(),
            )),
            None => Err(errors::NeteaseCallError::ParseError(
                "fail parsing code".to_string(),
            )),
        }
    }

    async fn user_account(&self) -> Result<Response, reqwest::Error> {
        todo!()
    }
}
