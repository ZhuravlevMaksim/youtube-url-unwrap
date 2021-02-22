use wasm_bindgen::prelude::*;
use boa::Context;
use std::collections::HashMap;


#[wasm_bindgen]
pub fn evalJs(s: &str) {
    let mut context = Context::new();

    let value = context.eval(s).unwrap();

    println!("{}", value.to_string(&mut context).unwrap());
}

extern crate web_sys;
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct Unwrap {
    pub origin: String
}

#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub async fn unwrap() -> Result<(), JsValue> {
    log!("wasm fetch");
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("https://httpbin.org/ip", &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Use serde to parse the JSON into a struct.
    let responseJson: Unwrap = json.into_serde().unwrap();

    log!("{:#?}", responseJson);

    Ok(())

}

#[cfg(target_arch = "x86_64")]
pub fn unwrap() -> Result<(), Box<dyn std::error::Error>> {
    println!("x86_64 fetch");
    let resp = reqwest::blocking::get("https://httpbin.org/ip")?
        .json::<HashMap<String, String>>()?;
    println!("{:#?}", resp);
    Ok(())
}
