use rustix::time::{ClockId, clock_gettime};

const SECOND: i64 = 1_000_000_000;

pub fn timestamp() -> i64 {
    let timespec = clock_gettime(ClockId::Realtime);
    timespec.tv_sec * SECOND + timespec.tv_nsec
}
