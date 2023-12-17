// https://github.com/rust-lang/docs.rs/blob/7fdd5d839cb68d703c2732d784aa12692d58ab54/src/web/page/mod.rs
pub mod templates;
pub mod web_page;

pub(crate) use templates::TemplateData;

use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GlobalAlert {
    pub(crate) url: &'static str,
    pub(crate) text: &'static str,
    pub(crate) css_class: &'static str,
    pub(crate) fa_icon: &'static str,
}

#[cfg(test)]
mod tera_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialize_global_alert() {
        let alert = GlobalAlert {
            url: "http://www.hasthelargehadroncolliderdestroyedtheworldyet.com/",
            text: "THE WORLD WILL SOON END",
            css_class: "THE END IS NEAR",
            fa_icon: "https://gph.is/1uOvmqR",
        };

        let correct_json = json!({
            "url": "http://www.hasthelargehadroncolliderdestroyedtheworldyet.com/",
            "text": "THE WORLD WILL SOON END",
            "css_class": "THE END IS NEAR",
            "fa_icon": "https://gph.is/1uOvmqR"
        });

        assert_eq!(correct_json, serde_json::to_value(alert).unwrap());
    }
}
