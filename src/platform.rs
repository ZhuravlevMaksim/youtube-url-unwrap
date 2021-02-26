use boa::Context;
use boa::value::RcString;
use lazy_static::lazy_static;
use reqwest::Client;
use reqwest::header::HeaderMap;
use core::option::Option;
use serde_json::{Value};
use regex::Regex;
use std::error::Error;


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

pub async fn get_opus_stream(uid: &str) -> Result<AudioStream, Box<dyn std::error::Error>> {
    let json = CLIENT.get(&format!("https://www.youtube.com/watch?v={}&pbj=1", uid))
        .send().await?.text().await?;

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
                    Some(url) => Ok(AudioStream {
                        uid,
                        title,
                        length_seconds,
                        url: match url.get("url") {
                            Some(url) => Some(url.as_str().unwrap().to_owned()),
                            None => None
                        },
                    }),
                    None => Ok(AudioStream { uid, title, length_seconds, url: None })
                }
            }
            None => panic!("no logic for this situation")
        }
    }
}

async fn parse_url(uid: &str) {
    fn eval_js(s: &str) -> RcString {
        let mut context = Context::new();
        context
            .eval(s).unwrap()
            .to_string(&mut context).unwrap()
    }

    let string = CLIENT.get(&format!("https://www.youtube.com/embed/{}", uid))
        .send().await.unwrap().text().await.unwrap();

    let base_player_url = &REGEX.captures(&string).unwrap()[1];
}

#[derive(Debug)]
pub struct AudioStream {
    uid: String,
    title: String,
    length_seconds: String,
    url: Option<String>,
}
