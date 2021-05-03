use core::option::Option;
use std::collections::HashMap;

use boa::Context;
use regex::Regex;
use reqwest::Client;
use reqwest::header::HeaderMap;
use serde_json::Value;

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
        Regex::new(r"[^a-zA-Z0-9 ]").unwrap()
            .replace_all(&self.title, "")
            .to_string() + ".opus"
    }
}

pub struct Extractor {
    client: Client,
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
        }
    }

    pub async fn get_opus_stream(&self, uid: &str) -> Result<AudioStream> {
        let player = self.get_player(uid).await?;

        let mut opus = None;
        let mut signature_cipher = None;
        for item in player["streamingData"]["adaptiveFormats"].as_array().unwrap() {
            if item["mimeType"] == "audio/webm; codecs=\"opus\"" {
                opus = Some(item);
                signature_cipher = item["signatureCipher"].as_str();
                break;
            }
        };

        let title = player["videoDetails"]["title"].as_str().unwrap().to_owned();
        let length_seconds = player["videoDetails"]["lengthSeconds"].as_str().unwrap().to_owned();

        let url = Decoder {
            uid,
            opus,
            signature_cipher: signature_cipher.unwrap(),
            client: &self.client,
        }.decode_stream_url().await;

        Ok(AudioStream {
            uid: uid.to_owned(),
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
}

pub struct Decoder<'a> {
    uid: &'a str,
    opus: Option<&'a Value>,
    signature_cipher: &'a str,
    client: &'a Client,
}

impl Decoder<'_> {
    #[allow(unused_must_use)]
    async fn decode_stream_url(&self) -> Option<String> {
        match self.opus {
            Some(item) => {
                match item.get("url") {
                    Some(url) => Some(url.as_str().unwrap().to_owned()),
                    None => {
                        let player_with_base_js = self.client.get(&format!("https://www.youtube.com/embed/{}", self.uid))
                            .send().await.unwrap().text().await.unwrap();
                        let base_js_url = &Regex::new("\"([/|\\w.]+base\\.js)\"").unwrap()
                            .captures(&player_with_base_js).unwrap()[1];
                        let base_js = self.client.get(&format!("https://youtube.com{}", base_js_url))
                            .send().await.unwrap().text().await.unwrap();


                        let name_matchers = vec![
                            Regex::new("(?:\\b|[^a-zA-Z0-9$])([a-zA-Z0-9$]{2})\\s*=\\s*function\\(\\s*a\\s*\\)\\s*\\{\\s*a\\s*=\\s*a\\.split\\(\\s*\"\"\\s*\\)"),
                            Regex::new("([\\w$]+)\\s*=\\s*function\\((\\w+)\\)\\{\\s*\\2=\\s*\\2\\.split\\(\"\"\\)\\s*;"),
                            Regex::new("yt\\.akamaized\\.net/\\)\\s*\\|\\|\\s*.*?\\s*c\\s*&&\\s*d\\.set\\([^,]+\\s*,\\s*(:encodeURIComponent\\s*\\()([a-zA-Z0-9$]+)\\("),
                            Regex::new("\\bc\\s*&&\\s*d\\.set\\([^,]+\\s*,\\s*(:encodeURIComponent\\s*\\()([a-zA-Z0-9$]+)\\(")
                        ];

                        let function_name = name_matchers.into_iter().find_map(|x| x.unwrap().captures(&base_js).unwrap().get(1)).unwrap().as_str();
                        let body_matcher = Regex::new(&*("(".to_owned() + &*function_name.replace("$", "\\$") + "=function\\([a-zA-Z0-9_]+\\)\\{.+?\\})")).unwrap();
                        let fun_body = "var ".to_owned() + &body_matcher.captures(&base_js).unwrap()[1] + ";";
                        let helper = &Regex::new(";([A-Za-z0-9_\\$]{2})\\...\\(").unwrap().captures(&fun_body).unwrap()[1];
                        let helper = "(var ".to_owned() + helper.replace("$", "\\$").as_str() + "=\\{.+?\\}\\};)";
                        let base_js = &base_js.replace("\n", "");
                        let helper = &Regex::new(&helper).unwrap().captures(base_js).unwrap()[1];
                        let caller = "function decrypt(a){return ".to_owned() + function_name + "(a);}";
                        let decryption_function = helper.to_owned() + &*fun_body + &*caller;

                        let cipher = self.signature_cipher
                            .split('&')
                            .map(|kv| kv.split('='))
                            .map(|mut kv| (kv.next().unwrap().into(),
                                           kv.next().unwrap().into()))
                            .collect::<HashMap<String, String>>();

                        let mut context = Context::new();
                        context.eval(decryption_function);
                        let eval = context.eval("decrypt('".to_owned() + &cipher["s"] + "')").unwrap().to_string(&mut context);


                        let url = cipher["url"].to_owned() + "&" + &cipher["sp"] + "=" + eval.unwrap().as_str();

                        // todo: fix replace
                        Some(url.replace("%3F", "?").replace("%3D", "=").replace("%26", "&").replace("%25", "%"))
                    }
                }
            }
            None => None
        }
    }
}