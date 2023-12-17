// https://github.com/rust-lang/docs.rs/blob/2f67be0ed1f3c8d84d2a6c48b7d102598090d864/src/web/page/templates.rs#L15

use anyhow::Context;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::{collections::HashMap, fmt};
use tera::{Result as TeraResult, Tera};
use tracing::trace;

use super::highlight;

pub const TEMPLATES_DIRECTORY: &str = "templates";

/// Holds all data relevant to templating
#[derive(Debug)]
pub(crate) struct TemplateData {
    pub templates: Tera,
}

impl TemplateData {
    pub(crate) fn new() -> Result<Self> {
        trace!("Loading templates");

        let data = Self {
            templates: load_templates()?,
        };

        trace!("Finished loading templates");

        Ok(data)
    }
}

fn load_templates() -> Result<Tera> {
    // This uses a custom function to find the templates in the filesystem instead of Tera's
    // builtin way (passing a glob expression to Tera::new), speeding up the startup of the
    // application and running the tests.
    //
    // The problem with Tera's template loading code is, it walks all the files in the current
    // directory and matches them against the provided glob expression. Unfortunately this means
    // Tera will walk all the rustwide workspaces, the git repository and a bunch of other
    // unrelated data, slowing down the search a lot.
    //
    // TODO: remove this when https://github.com/Gilnaa/globwalk/issues/29 is fixed
    let mut tera = Tera::default();
    let template_raws = crate::page::generated_code::raw_templates();
    tera.add_raw_templates(template_raws).with_context(|| {
        format!("failed while loading tera templates in {TEMPLATES_DIRECTORY:?}")
    })?;

    // // This function will return any global alert, if present.
    // ReturnValue::add_function_to(
    //     &mut tera,
    //     "global_alert",
    //     serde_json::to_value(crate::GLOBAL_ALERT)?,
    // );
    // // This function will return the current version of docs.rs.
    // ReturnValue::add_function_to(
    //     &mut tera,
    //     "docsrs_version",
    //     Value::String(crate::BUILD_VERSION.into()),
    // );

    // Custom filters
    tera.register_filter("timeformat", timeformat);
    tera.register_filter("dbg", dbg);
    tera.register_filter("dedent", dedent);
    tera.register_filter("fas", IconType::Strong);
    tera.register_filter("far", IconType::Regular);
    tera.register_filter("fab", IconType::Brand);
    tera.register_filter("highlight", Highlight);

    Ok(tera)
}

/// Simple function that returns the pre-defined value.
struct ReturnValue {
    name: &'static str,
    value: Value,
}

impl ReturnValue {
    fn add_function_to(tera: &mut Tera, name: &'static str, value: Value) {
        tera.register_function(name, Self { name, value })
    }
}

impl tera::Function for ReturnValue {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        debug_assert!(args.is_empty(), "{} takes no args", self.name);
        Ok(self.value.clone())
    }
}

/// Prettily format a timestamp
// TODO: This can be replaced by chrono
#[allow(clippy::unnecessary_wraps)]
fn timeformat(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let fmt = if let Some(Value::Bool(true)) = args.get("relative") {
        let value = DateTime::parse_from_rfc3339(value.as_str().unwrap())
            .unwrap()
            .with_timezone(&Utc);

        super::duration_to_str(value)
    } else {
        const TIMES: &[&str] = &["seconds", "minutes", "hours"];

        let mut value = value.as_f64().unwrap();
        let mut chosen_time = &TIMES[0];

        for time in &TIMES[1..] {
            if value / 60.0 >= 1.0 {
                chosen_time = time;
                value /= 60.0;
            } else {
                break;
            }
        }

        // TODO: This formatting section can be optimized, two string allocations aren't needed
        let mut value = format!("{value:.1}");
        if value.ends_with(".0") {
            value.truncate(value.len() - 2);
        }

        format!("{value} {chosen_time}")
    };

    Ok(Value::String(fmt))
}

/// Print a tera value to stdout
#[allow(clippy::unnecessary_wraps)]
fn dbg(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    println!("{value:?}");

    Ok(value.clone())
}

/// Dedent a string by removing all leading whitespace
#[allow(clippy::unnecessary_wraps)]
fn dedent(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let string = value.as_str().expect("dedent takes a string");

    let unindented = if let Some(levels) = args
        .get("levels")
        .map(|l| l.as_i64().expect("`levels` must be an integer"))
    {
        string
            .lines()
            .map(|mut line| {
                for _ in 0..levels {
                    // `.strip_prefix` returns `Some(suffix without prefix)` if it's successful. If it fails
                    // to strip the prefix (meaning there's less than `levels` levels of indentation),
                    // we can just quit early
                    if let Some(suffix) = line.strip_prefix("    ") {
                        line = suffix;
                    } else {
                        break;
                    }
                }

                line
            })
            .collect::<Vec<&str>>()
            .join("\n")
    } else {
        string
            .lines()
            .map(|l| l.trim_start())
            .collect::<Vec<&str>>()
            .join("\n")
    };

    Ok(Value::String(unindented))
}

enum IconType {
    Strong,
    Regular,
    Brand,
}

impl fmt::Display for IconType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let icon = match self {
            Self::Strong => "solid",
            Self::Regular => "regular",
            Self::Brand => "brands",
        };

        f.write_str(icon)
    }
}

impl tera::Filter for IconType {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let icon_name = tera::escape_html(value.as_str().expect("Icons only take strings"));

        let type_ = match self {
            IconType::Strong => font_awesome_as_a_crate::Type::Solid,
            IconType::Regular => font_awesome_as_a_crate::Type::Regular,
            IconType::Brand => font_awesome_as_a_crate::Type::Brands,
        };

        let icon_file_string = font_awesome_as_a_crate::svg(type_, &icon_name[..]).unwrap_or("");
        let (space, class_extra) = match args.get("extra").and_then(|s| s.as_str()) {
            Some(extra) => (" ", extra),
            None => ("", ""),
        };

        let icon = format!(
            "\
<span class=\"fa-svg fa-svg-fw{space}{class_extra}\" aria-hidden=\"true\">{icon_file_string}</span>"
        );

        Ok(Value::String(icon))
    }

    fn is_safe(&self) -> bool {
        true
    }
}

struct Highlight;

impl tera::Filter for Highlight {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let code = value.as_str().ok_or_else(|| {
            let msg = format!( "Filter `highlight` was called on an incorrect value: got `{value}` but expected a string");
            tera::Error::msg(msg)
        })?;
        let lang = args
            .get("lang")
            .and_then(|lang| {
                if lang.is_null() {
                    None
                } else {
                    Some(lang.as_str().ok_or_else(|| {
                        let msg = format!("Filter `highlight` received an incorrect type for arg `{lang}`: got `{lang}` but expected a string");
                        tera::Error::msg(msg)
                    }))
                }
            })
            .transpose()?;
        let highlighted = highlight::with_lang(lang, code);
        Ok(format!("<pre><code>{highlighted}</code></pre>").into())
    }

    fn is_safe(&self) -> bool {
        true
    }
}
