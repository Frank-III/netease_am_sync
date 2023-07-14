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
    let mut headers = reqwest::header::HeaderMap::new();
    dotenv().ok();
    let endpoint =
        std::env::var("NETEASE_ENDPOINT").unwrap_or_else(|_| "http://localhost:4000".to_string());

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
    let http_client = reqwest::Client::new();
    let t = Local::now().timestamp_millis().to_string();
    println!("{endpoint}, {t}");
    let Ok(response) = http_client
        .get(format!("{}/login/qr/key", endpoint))
        .headers(headers.clone())
        .query(&vec![("timestamp", Local::now().timestamp_millis().to_string())])
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
    let mut cookie: Option<String> = None;
    loop {
        let cookie_map = login_qr_check(&http_client, unikey.unwrap()).await;
        match cookie_map {
            Ok(cookies) => {
                println!("success");

                //this cookies works
                // headers.insert(header::COOKIE, "_ga=GA1.1.513827141.1688404224; NMTID=00O523APh9QRBswzEzwtrFhEkA1ShAAAAGJHTEHvg; _ga_KMJJCFZDKF=GS1.1.1689192763.4.1.1689194255.0.0.0; __csrf=9c6e41b48e910c50b0372e10fe87d060; MUSIC_U=009CE7AE44ABA22ED3B2899C974A1A4AFC43211CC6EC4D0774AF0414A4D0A687EE81B666C83E23D41EDC57B6A197159B0764D4DB43C539115520DA6437975AF4AE2F70084F7AF66B2DB7D61EFAA60DF61E058E0CB40B864FC3093897F7D276A8D8D0876419AD937AA835487D5E0303ADDB053FE2DC47E8BB5C81A17566D9A43854E449A95A1B56E806636C8E6CA4BE631659A23CE80DE0EB32C031A38417A2A847EAC96BCF315E946A007146BFC87B22616E13C155F1BA9868E7514E5948745C58D6B3FE930B0E3DC6FEEDF87DA3FA9F85470763A38A20E95BCB6C684D3BB076F0023FE68EE8573075933C3965AC67477546B5000D08C6FB80078DAE42E13DE1576F80E4539AFD6D29AC8CC96A52C4E27D504C08203226ECA68E3A3C0C61B7EBF9E973364029F872F82087765F453CFB1C88BA4231589C392C44A9C8C2B4F2D1C8".parse().unwrap());
                cookie = Some(cookies);
                // headers.insert(header::COOKIE, cookies.parse().unwrap());
                break;
            }
            Err(err) => {
                eprint!("{err:#?}");
                thread::sleep(Duration::from_secs(3));
            }
        }
    }
    match get_endpoint(
        &http_client,
        format!("{}/login/status", &endpoint),
        cookie.unwrap().as_str(),
    )
    .await
    {
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

async fn get_endpoint<U: IntoUrl>(
    client: &reqwest::Client,
    endpoint: U,
    cookies: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    client
        .get(endpoint)
        .query(&vec![
            ("timestamp", Local::now().timestamp_millis().to_string()),
            ("cookie", cookies.to_string()),
        ])
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
            "http://localhost:27232/api/login/qr/check?timestamp={}&key={}",
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

// TODO: do I need a hashmap or just string
fn get_cookies(cookies: String) -> String {
    let mut cookie_vals: HashMap<String, String> = HashMap::new();
    cookie_vals.insert(
        "_ga_KMJJCFZDKF".into(),
        "GS1.1.1689200897.5.1.1689200923.0.0.0".into(),
    );
    cookie_vals.insert("_ga".into(), "GA1.1.513827141.1688404224".into());
    cookies
        .replace(" HTTPOnly", "")
        .split(";;")
        .for_each(|cookie| {
            let mut cookie_ = cookie.split(';').next().unwrap().split('=');
            if cookies.len() >= 2 {
                cookie_vals.insert(
                    cookie_.next().unwrap().to_string(),
                    // format!("cookie-{}", cookie_.next().unwrap()),
                    cookie_.next().unwrap().to_string(),
                );
            }
        });
    cookie_vals
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(";")
}
