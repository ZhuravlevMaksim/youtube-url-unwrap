use boa::Context;
use boa::class::ClassBuilder;
use boa::value::RcString;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use core::option::Option;
use serde_json::{Value};
use regex::Regex;


lazy_static! {
    static ref CLIENT: Client = {
        Client::builder().default_headers({
            let mut map = HeaderMap::new();
            map.insert("X-YouTube-Client-Name", "1".parse().unwrap());
            map.insert("X-YouTube-Client-Version", "2.20200214.04.00".parse().unwrap());
            map.insert("Accept-Language", "en-EN, en;q=0.9".parse().unwrap());
            map
        }).build().unwrap()
    };

    static ref REGEX: Regex = Regex::new("\"([/|\\w.]+base\\.js)\"").unwrap();
}

#[cfg(target_arch = "x86_64")]
pub fn get_opus_stream(uid: &str) -> AudioStream {
    let json = CLIENT.get(&format!("https://www.youtube.com/watch?v={}&pbj=1", uid))
        .send().unwrap().text().unwrap();

    let serde_json_value: Value = serde_json::from_str(&json).unwrap();
    let ref content: Value = serde_json_value[2];

    match content.get("player") {
        Some(_) => panic!("no logic for player response"),
        None => match content.get("playerResponse") {
            Some(player) => {
                let mut opus = None;
                for item in player["streamingData"]["adaptiveFormats"].as_array().unwrap() {
                    if item["mimeType"] == "audio/webm; codecs=\"opus\"" {
                        opus = Some(item);
                        break;
                    }
                };

                let uid = uid.to_owned();
                let title = player["videoDetails"]["title"].as_str().unwrap().to_owned();
                let length_seconds = player["videoDetails"]["lengthSeconds"].as_str().unwrap().to_owned();

                match opus {
                    Some(url) => AudioStream {
                        uid,
                        title,
                        length_seconds,
                        url: match url.get("url") {
                            Some(url) => Some(url.as_str().unwrap().to_owned()),
                            None => None
                        },
                    },
                    None => AudioStream { uid, title, length_seconds, url: None },
                }
            }
            None => panic!("no player info")
        }
    }
}


fn parse_url(uid: &str) {
    let string = CLIENT.get(&format!("https://www.youtube.com/embed/{}", uid))
        .send().unwrap().text().unwrap();
    let player_url = &REGEX.captures(&string).unwrap()[1];
}

fn eval_js(s: &str) -> RcString {
    let mut context = Context::new();
    context
        .eval(s).unwrap()
        .to_string(&mut context).unwrap()
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    extern crate web_sys;

    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::Request;

    macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }

    #[wasm_bindgen]
    pub async fn unwrap_url() {
        log!("wasm fetch");
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init("https://httpbin.org/ip", &opts).unwrap();

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();

        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        let json = JsFuture::from(resp.json().unwrap()).await.unwrap();

        // Use serde to parse the JSON into a struct.
        let responseJson: AudioStreamInfo = json.into_serde().unwrap();

        log!("{:#?}", responseJson);
    }
}

#[derive(Debug)]
pub struct AudioStream {
    uid: String,
    title: String,
    length_seconds: String,
    url: Option<String>,
}
