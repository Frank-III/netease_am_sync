use chrono::Local;
use rqcode::render::unicode;
use rqcode::QrCode;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::{thread, time::Duration};
use tokio::sync::RwLock;

// #[derive(Debug)]
// struct NeteaseApi {
//     client: Arc<RwLock<reqwest::Client>>,
//     base_uri: String,
// }
//
// impl NeteaseApi {
//     async fn gen_qr_code() {
//         todo!()
//     }
//
//     async fn login_qr_check() {
//         todo!()
//     }
// }

#[tokio::main]
async fn main() {
    let mut params = HashMap::new();
    let timestamp = Local::now().timestamp_millis();
    println!("{:?}", timestamp);
    params.insert("timestamp", timestamp);
    let http_client = reqwest::Client::new();
    let Ok(response) = http_client
        .get("http://localhost:4000/login/qr/key")
        .query(&params)
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
        if login_qr_check(&http_client, unikey.unwrap().as_ref()).await {
            return;
        }
        thread::sleep(Duration::from_secs(2));
    }
}

async fn login_qr_check(client: &reqwest::Client, key: &str) -> bool {
    let timestamp = Local::now().timestamp_millis();
    let Ok(response) = client.get(format!("http://localhost:4000/login/qr/check?timestamp={}&key={}", timestamp, key)).send().await else {
        eprint!("fail to connect to the server");
        return false;
    };

    let Ok(serialize) = response
        .json::<Value>()
        .await else {
            eprint!("serialize failed");
            return false;
        };

    match serialize.get("code").unwrap().as_i64() {
        Some(801) => {
            eprint!("QRCode is outdated");
            false
        }
        Some(802) => {
            eprint!("QRCode is outdated");
            false
        }
        Some(803) => {
            println!("Qrcode succeed!");
            println!("the cookie is {:#?}", serialize.get("cookie").unwrap());
            true
        }
        // Some(200) => {
        //     println!("Qrcode succeed!");
        //     println!("the cookie is {:#?}", serialize.get("cookie").unwrap());
        //     return true;
        // }
        Some(_) => {
            eprint!("unrecognized code");
            false
        }
        None => {
            eprint!("fail parsing code");
            false
        }
    }
}

fn get_cookies(cookies: String) -> HashMap<String, String> {
    let mut cookie_vals: HashMap<String, String> = HashMap::new();

    cookies.split(";;").for_each(|cookie| {
        let mut cookie_ = cookie.split(';').next().unwrap().split('=');
        cookie_vals.insert(
            cookie_.next().unwrap().to_string(),
            cookie_.next().unwrap().to_string(),
        );
    });
    cookie_vals
}
