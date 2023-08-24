use anyhow::Result;
use nix::time::{clock_gettime, ClockId};
use std::time::Duration;

#[inline]
pub fn duration_since_boot() -> Result<Duration> {
    Ok(Duration::from(clock_gettime(ClockId::CLOCK_MONOTONIC)?))
}
