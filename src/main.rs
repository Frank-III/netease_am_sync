use chrono::Local;
use reqwest::{header, Response};
use rqcode::render::unicode;
use rqcode::QrCode;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::{thread, time::Duration};
mod errors;

#[tokio::main]
async fn main() {
    let mut params = HashMap::new();
    // let timestamp = Local::now().timestamp_millis().to_string();
    // println!("{:?}", timestamp);
    // params.insert("timestamp", timestamp);
    let http_client = reqwest::Client::new();
    let Ok(response) = http_client
        .get("http://localhost:4000/login/qr/key")
        .header("timestamp", Local::now().timestamp_millis().to_string())
        .send()
        .await else {
            eprint!("fail to connect to the server");
            return;
        };
    let Ok(serialize) = response.json::<Value>().await else {
            eprint!("fail to deserialize the result");
            return;
    };

    let unikey: Option<&str> = match serialize["code"].as_i64() {
        Some(200) => serialize
            .get("data")
            .unwrap()
            .get("unikey")
            .unwrap()
            .as_str(),
        _ => {
            eprint!("the reponse failed");
            return;
        }
    };
    println!("the unikey is {unikey:?}");
    let code = QrCode::new(format!(
        "https://music.163.com/login?codekey={}",
        unikey.unwrap()
    ))
    .unwrap();
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{}", image);
    // enter a loop
    loop {
        let cookie_map = login_qr_check(&http_client, unikey.unwrap()).await;
        match cookie_map {
            Ok(cookies) => {
                println!("success");
                params.insert(header::COOKIE.as_ref(), cookies);
                break;
            }
            Err(err) => {
                eprint!("{err:#?}");
                thread::sleep(Duration::from_secs(3));
            }
        }
    }
    match get_endpoint(&http_client, "http://localhost:4000/user/account", &params).await {
        Ok(data) => match data.json::<Value>().await {
            Ok(serialized) => {
                println!("{serialized:#?}");
            }
            Err(_) => {
                eprintln!("failed to parse user information");
            }
        },
        Err(_) => eprintln!("fail to fetch user information"),
    }
}

async fn get_endpoint(
    client: &reqwest::Client,
    endpoint: &str,
    query: &HashMap<&str, String>,
) -> Result<reqwest::Response, reqwest::Error> {
    client
        .get(endpoint)
        .query(query)
        .header("timestamp", Local::now().timestamp_millis().to_string())
        .send()
        .await
}

// TODO: refactor this to return a Result
async fn login_qr_check(
    client: &reqwest::Client,
    key: &str,
) -> Result<String, errors::NeteaseCallError> {
    let timestamp = Local::now().timestamp_millis();
    let Ok(response) = client
        .get(format!(
            "http://localhost:4000/login/qr/check?timestamp={}&key={}",
            timestamp, key
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
            println!("the cookie is {:#?}", cookies);
            let cookie_map = get_cookies(cookies.unwrap().to_string());
            println!("the cookie_map is {:#?}", cookie_map);
            Ok(cookie_map)
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

// TODO: do I need a hashmap or just string
fn get_cookies(cookies: String) -> String {
    let mut cookie_vals: HashMap<String, String> = HashMap::new();

    cookies.split(";;").for_each(|cookie| {
        let mut cookie_ = cookie.split(';').next().unwrap().split('=');
        cookie_vals.insert(
            cookie_.next().unwrap().to_string(),
            // format!("cookie-{}", cookie_.next().unwrap()),
            cookie_.next().unwrap().to_string(),
        );
    });
    cookie_vals
        .iter()
        .map(|(k, v)| format!("{}={}", k, v).replace(";", "%3B"))
        .collect::<Vec<_>>()
        .join(";")
}
