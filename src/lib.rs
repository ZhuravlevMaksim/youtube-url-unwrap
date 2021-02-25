use boa::Context;
use serde::{Deserialize, Serialize};
use boa::value::RcString;

#[derive(Debug, Serialize, Deserialize)]
pub struct Unwrap {
    pub origin: String
}

fn eval_js(s: &str) -> RcString {
    let mut context = Context::new();
    context
        .eval(s).unwrap()
        .to_string(&mut context).unwrap()
}

#[cfg(target_arch = "x86_64")]
pub fn unwrap_url(uid: &str) -> Unwrap {
    reqwest::blocking::get(uid).unwrap().json::<Unwrap>().unwrap()
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    extern crate web_sys;

    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request};

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
        let responseJson: Unwrap = json.into_serde().unwrap();

        log!("{:#?}", responseJson);
    }
}

