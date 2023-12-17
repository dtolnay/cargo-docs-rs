use std::time::Duration;

/// Converts a `Duration` to seconds, used by prometheus internally
#[inline]
pub(crate) fn duration_to_seconds(d: Duration) -> f64 {
    let nanos = f64::from(d.subsec_nanos()) / 1e9;
    d.as_secs() as f64 + nanos
}
