use chrono::Local;
use dotenvy::dotenv;
use reqwest::header::HeaderMap;
use reqwest::{header, IntoUrl, Response};
use rqcode::render::unicode;
use rqcode::QrCode;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::{thread, time::Duration};
pub mod errors;
pub mod neteaseapi;
#[tokio::main]
async fn main() {
    dotenv().ok();
    // headers.insert(
    //     reqwest::header::CONTENT_TYPE,
    //     "application/x-www-form-urlencoded".parse().unwrap(),
    // );
    // headers.insert(reqwest::header::ACCEPT, "*/*".parse().unwrap());
    // headers.insert(
    //     reqwest::header::REFERER,
    //     "https://music.163.com".parse().unwrap(),
    // );
    // headers.insert(
    //     reqwest::header::USER_AGENT,
    //     "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:65.0) Gecko/20100101 Firefox/65.0"
    //         .parse()
    //         .unwrap(),
    // );
    //
    let mut client = neteaseapi::NeteaseApi::new();
    let Ok(response) = client.gen_qr_code().await else {
        eprintln!("gen code failed");
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
    loop {
        match client.login_qr_check(unikey.as_ref().unwrap()).await {
            Ok(_) => {
                println!("success");
                break;
            }
            Err(err) => {
                eprintln!("{err:#?}");
                thread::sleep(Duration::from_secs(3));
            }
        }
    }
    client.user_account().await;
    println!("{client:#?}");
}
