use crate::errors;
use chrono::Local;
use reqwest::{Request, Response};
use serde_json::Value;
use std::sync::Arc;
use std::{error, vec};
use std::{thread, time::Duration};
use tokio::sync::RwLock;
// TODO: watch the great build video about how login qr check could update the cookies
#[derive(Debug)]
pub struct NeteaseApi {
    client: reqwest::Client,
    base_uri: String,
    //headers: reqwest::header::HeaderMap,
    cookies: Option<String>, //seems to work
    uid: Option<String>,
}

impl Default for NeteaseApi {
    fn default() -> Self {
        Self::new()
    }
}
impl NeteaseApi {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_uri: std::env::var("NETEASE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4000".to_string()),
            cookies: std::env::var("NETEASE_COOKIES").ok(),
            uid: std::env::var("NETEASE_UID").ok(),
            //headers: reqwest::headers::HeaderMap::new(),
        }
    }
    pub fn set_cookies(&mut self, cookies: &str) {
        self.cookies = Some(cookies.to_string());
    }
    pub fn has_cookie(&self) -> bool {
        self.cookies.is_some()
    }

    pub async fn gen_qr_code(&self) -> Result<Response, reqwest::Error> {
        self.client
            .get(format!("{}/login/qr/key", self.base_uri))
            .query(&vec![(
                "timestamp",
                Local::now().timestamp_millis().to_string(),
            )])
            .send()
            .await
    }

    pub async fn login_qr_check(&mut self, key: &str) -> Result<(), errors::NeteaseCallError> {
        let Ok(response) = self.client
        .get(format!("{}/login/qr/check",self.base_uri))
        .query(&[("timestamp", Local::now().timestamp_millis().to_string().as_str()), ("key", key)])
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
                self.cookies = serialize.get("cookie").map(|s| s.to_string());
                Ok(())
            }
            Some(_) => Err(errors::NeteaseCallError::ParseError(
                "unrecognized code".to_string(),
            )),
            None => Err(errors::NeteaseCallError::ParseError(
                "fail parsing code".to_string(),
            )),
        }
    }

    pub async fn user_account(&mut self) -> errors::NetResult<()> {
        if self.cookies.is_none() {
            return Err(errors::NeteaseCallError::NoCookieError);
        };
        match self
            .client
            .get(format!("{}/login/status", &self.base_uri))
            .query(&vec![
                (
                    "timestamp",
                    Local::now().timestamp_millis().to_string().as_str(),
                ),
                ("cookie", self.cookies.as_ref().unwrap()),
            ])
            .send()
            .await
        {
            Ok(data) => match data.json::<Value>().await {
                Ok(serialized) => {
                    println!("{serialized:#?}");
                    match serialized.get("data") {
                        Some(data) => {
                            self.uid = data
                                .get("profile")
                                .unwrap()
                                .get("userId")
                                .map(|s| s.to_string());
                            println!("{}", self.uid.as_ref().unwrap());
                            Ok(())
                        }
                        None => Err(errors::NeteaseCallError::ParseError(
                            "Fail to get Uid".to_string(),
                        )),
                    }
                }
                Err(_) => Err(errors::NeteaseCallError::ParseError(
                    "failed to parse user information".to_string(),
                )),
            },
            Err(_) => Err(errors::NeteaseCallError::ClientFailError(1)),
        }
    }

    // get vector of ids but I want the details
    pub async fn user_likelist(&self) -> errors::NetResult<Value> {
        if self.cookies.is_none() || self.uid.is_none() {
            return Err(errors::NeteaseCallError::NoCookieError);
        };
        match self
            .client
            .get(format!("{}/likelist", &self.base_uri))
            .query(&vec![
                (
                    "timestamp",
                    Local::now().timestamp_millis().to_string().as_str(),
                ),
                ("cookie", self.cookies.as_ref().unwrap()),
                ("uid", self.uid.as_ref().unwrap()),
            ])
            .send()
            .await
        {
            Ok(data) => match data.json::<Value>().await {
                Ok(v) => Ok(v),
                Err(_) => Err(errors::NeteaseCallError::ParseError(
                    "Failed to parse likelist".to_string(),
                )),
            },
            Err(e) => {
                eprintln!("{e:#?}");
                Err(errors::NeteaseCallError::ClientFailError(1))
            }
        }
    }

    // get likelist id + likelist
    pub async fn likelist_details(&self) -> errors::NetResult<Value> {
        if self.cookies.is_none() || self.uid.is_none() {
            return Err(errors::NeteaseCallError::NoCookieError);
        };
        match self
            .client
            .get(format!("{}/likelist", &self.base_uri))
            .query(&vec![
                (
                    "timestamp",
                    Local::now().timestamp_millis().to_string().as_str(),
                ),
                ("cookie", self.cookies.as_ref().unwrap()),
                ("uid", self.uid.as_ref().unwrap()),
            ])
            .send()
            .await
        {
            Ok(data) => match data.json::<Value>().await {
                Ok(v) => Ok(v),
                Err(_) => Err(errors::NeteaseCallError::ParseError(
                    "Failed to parse likelist".to_string(),
                )),
            },
            Err(e) => {
                eprintln!("{e:#?}");
                Err(errors::NeteaseCallError::ClientFailError(1))
            }
        }
    }
}
