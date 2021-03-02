use boa::Context;
use boa::value::RcString;
use reqwest::Client;
use reqwest::header::HeaderMap;
use core::option::Option;
use serde_json::{Value};
use regex::Regex;
use std::error::Error;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
struct NoPlayerResponse;

impl core::fmt::Display for NoPlayerResponse {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "No player in response")
    }
}

impl std::error::Error for NoPlayerResponse {}

#[derive(Debug)]
pub struct AudioStream {
    pub uid: String,
    pub title: String,
    pub length_seconds: String,
    pub url: Option<String>,
}

impl AudioStream {
    pub fn file_name(&self) -> String {
        println!("{}", &self.title);
        Regex::new(r"[<>:/|?*]").unwrap()
            .replace_all(&self.title, "")
            .to_string() + ".opus"
    }
}

pub struct Extractor {
    client: Client,
    regex: Regex,
}

impl Extractor {
    pub fn new() -> Self {
        Extractor {
            client: {
                Client::builder().default_headers({
                    let mut map = HeaderMap::new();
                    map.insert("X-YouTube-Client-Name", "1".parse().unwrap());
                    map.insert("X-YouTube-Client-Version", "2.20200214.04.00".parse().unwrap());
                    map.insert("Accept-Language", "en-EN, en;q=0.9".parse().unwrap());
                    map
                }).build().unwrap()
            },
            regex: Regex::new("\"([/|\\w.]+base\\.js)\"").unwrap(),
        }
    }

    pub async fn get_opus_stream(&self, uid: &str) -> Result<AudioStream> {
        let player = self.get_player(uid).await?;

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

        let url = self.decode_stream_url(opus);

        Ok(AudioStream {
            uid,
            title,
            length_seconds,
            url,
        })
    }

    async fn get_player(&self, uid: &str) -> Result<Value> {
        let json = self.client.get(&format!("https://www.youtube.com/watch?v={}&pbj=1", uid))
            .send().await?.text().await?;

        let mut serde_json_value: Value = serde_json::from_str(&json)?;
        let content = serde_json_value.get_mut(2).unwrap();

        content.get_mut("playerResponse")
            .ok_or(NoPlayerResponse.into())
            .map(|player| player.take())
    }

    fn decode_stream_url(&self, opus: Option<&Value>) -> Option<String> {
        match opus {
            Some(item) => {
                match item.get("url") {
                    Some(url) => Some(url.as_str().unwrap().to_owned()),
                    None => None
                }
            }
            None => None
        }
    }

    async fn parse_parse_url(&self, uid: &str) -> String {
        fn eval_js(s: &str) -> RcString {
            let mut context = Context::new();
            context
                .eval(s).unwrap()
                .to_string(&mut context).unwrap()
        }

        let string = self.client.get(&format!("https://www.youtube.com/embed/{}", uid))
            .send().await.unwrap().text().await.unwrap();

        self.regex.captures(&string).unwrap()[1].to_owned()
    }
}