use log::debug;
use prometheus::HistogramVec;
use std::time::Instant;

use crate::docs_rs::docs_rs::metrics::duration_to_seconds;
struct RenderingTime {
    start: Instant,
    step: &'static str,
}

pub(crate) struct RenderingTimesRecorder<'a> {
    metric: &'a HistogramVec,
    current: Option<RenderingTime>,
}

impl<'a> RenderingTimesRecorder<'a> {
    pub(crate) fn new(metric: &'a HistogramVec) -> Self {
        Self {
            metric,
            current: None,
        }
    }

    pub(crate) fn step(&mut self, step: &'static str) {
        self.record_current();
        self.current = Some(RenderingTime {
            start: Instant::now(),
            step,
        });
    }

    fn record_current(&mut self) {
        if let Some(current) = self.current.take() {
            debug!(
                "rendering time - {}: {:?}",
                current.step,
                current.start.elapsed()
            );
            self.metric
                .with_label_values(&[current.step])
                .observe(duration_to_seconds(current.start.elapsed()));
        }
    }
}

impl Drop for RenderingTimesRecorder<'_> {
    fn drop(&mut self) {
        self.record_current();
    }
}
