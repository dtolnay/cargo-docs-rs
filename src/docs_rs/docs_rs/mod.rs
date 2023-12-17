#![allow(unused)]

// https://github.com/rust-lang/docs.rs/blob/2f67be0ed1f3c8d84d2a6c48b7d102598090d864/src/web/mod.rs
pub mod config;
pub mod index;
pub mod metrics;
pub mod registry_api;
pub mod storage;
pub mod utils;
pub mod web;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

// from https://github.com/servo/rust-url/blob/master/url/src/parser.rs
// and https://github.com/tokio-rs/axum/blob/main/axum-extra/src/lib.rs
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
const PATH: &AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');

pub(crate) fn encode_url_path(path: &str) -> String {
    utf8_percent_encode(path, PATH).to_string()
}
