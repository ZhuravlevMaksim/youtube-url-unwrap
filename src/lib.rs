use wasm_bindgen::prelude::*;
use boa::Context;


#[wasm_bindgen]
pub fn eval(s: &str) {
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