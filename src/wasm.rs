extern crate web_sys;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Request;
use self::web_sys::{RequestInit, RequestMode, Response};


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
    // let responseJson: AudioStream = json.into_serde().unwrap();

    // log!("{:#?}", responseJson);
}
