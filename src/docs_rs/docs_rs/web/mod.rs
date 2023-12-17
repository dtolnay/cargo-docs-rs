// https://github.com/rust-lang/docs.rs/blob/7fdd5d839cb68d703c2732d784aa12692d58ab54/src/web/mod.rs

use chrono::{DateTime, Utc};
use serde::Serialize;

pub mod crate_details;
pub mod csp;
pub mod highlight;
pub mod markdown;
pub mod metrics;
pub mod page;
pub mod rustdoc;

/// Converts Timespec to nice readable relative time string
fn duration_to_str(init: DateTime<Utc>) -> String {
    let now = Utc::now();
    let delta = now.signed_duration_since(init);

    let delta = (
        delta.num_days(),
        delta.num_hours(),
        delta.num_minutes(),
        delta.num_seconds(),
    );

    match delta {
        (days, ..) if days > 5 => format!("{}", init.format("%b %d, %Y")),
        (days @ 2..=5, ..) => format!("{days} days ago"),
        (1, ..) => "one day ago".to_string(),

        (_, hours, ..) if hours > 1 => format!("{hours} hours ago"),
        (_, 1, ..) => "an hour ago".to_string(),

        (_, _, minutes, _) if minutes > 1 => format!("{minutes} minutes ago"),
        (_, _, 1, _) => "one minute ago".to_string(),

        (_, _, _, seconds) if seconds > 0 => format!("{seconds} seconds ago"),
        _ => "just now".to_string(),
    }
}

/// MetaData used in header
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MetaData {
    pub(crate) name: String,
    // If we're on a page with /latest/ in the URL, the string "latest".
    // Otherwise, the version as a string.
    pub(crate) version_or_latest: String,
    // The exact version of the crate being shown. Never contains "latest".
    pub(crate) version: String,
    pub(crate) description: Option<String>,
    pub(crate) target_name: Option<String>,
    pub(crate) rustdoc_status: bool,
    pub(crate) default_target: String,
    pub(crate) doc_targets: Vec<String>,
    pub(crate) yanked: bool,
    /// CSS file to use depending on the rustdoc version used to generate this version of this
    /// crate.
    pub(crate) rustdoc_css_file: String,
}
