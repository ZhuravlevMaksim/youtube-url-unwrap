#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "x86_64")]
pub mod platform;