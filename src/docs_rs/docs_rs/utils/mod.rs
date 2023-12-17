use std::{future::Future, time::Duration};

use log::warn;

use anyhow::Result;

pub mod rustc_version;

pub(crate) async fn retry_async<T, Fut, F: FnMut() -> Fut>(mut f: F, max_attempts: u32) -> Result<T>
where
    Fut: Future<Output = Result<T>>,
{
    for attempt in 1.. {
        match f().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if attempt > max_attempts {
                    return Err(err);
                } else {
                    let sleep_for = 2u32.pow(attempt);
                    warn!(
                        "got error on attempt {}, will try again after {}s:\n{:?}",
                        attempt, sleep_for, err
                    );
                    tokio::time::sleep(Duration::from_secs(sleep_for as u64)).await;
                }
            }
        }
    }
    unreachable!();
}
