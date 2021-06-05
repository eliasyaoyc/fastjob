use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};
pub use std::time::Duration;
use time::{Duration as TimeDuration, Timespec};
use tracing::field::debug;

/// Converts Duration to milliseconds.
#[inline]
pub fn duration_to_ms(d: Duration) -> u64 {
    let nanos = u64::from(d.subsec_nanos());
    d.as_secs() * 1_000 + (nanos / 1_000_000)
}

/// Convert Duration to second
#[inline]
pub fn duration_to_sec(d: Duration) -> f64 {
    let nanos = f64::from(d.subsec_nanos());
    d.as_secs() as f64 + (nanos / 1_000_000_000.0)
}

/// Converts Duration to nanoseconds.
#[inline]
pub fn duration_to_nanos(d: Duration) -> u64 {
    let nanos = u64::from(d.subsec_nanos());
    d.as_secs() * 1_000_000_000 + nanos
}

use self::inner::monotonic_coarse_now;
pub use self::inner::monotonic_now;
/// Returns the monotonic raw time since some unspecified starting point.
pub use self::inner::monotonic_raw_now;

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
const MILLISECOND_PER_SECOND: i64 = 1_000;
const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;

#[cfg(not(target_os = "linux"))]
mod inner {
    use super::NANOSECONDS_PER_SECOND;
    use time::{self, Timespec};

    pub fn monotonic_raw_now() -> Timespec {
        // TODO Add monotonic raw clock time impl for macos and windows
        // Currently use `time::get_precise_ns()` instead.
        let ns = time::precise_time_ns();
        let s = ns / NANOSECONDS_PER_SECOND;
        let ns = ns % NANOSECONDS_PER_SECOND;
        Timespec::new(s as i64, ns as i32)
    }

    pub fn monotonic_now() -> Timespec {
        // TODO Add monotonic clock time impl for macos and windows
        monotonic_raw_now()
    }

    pub fn monotonic_coarse_now() -> Timespec {
        // TODO Add monotonic coarse clock time impl for macos and windows
        monotonic_raw_now()
    }
}

#[cfg(target_os = "linux")]
mod inner {
    use std::io;
    use time::Timespec;

    #[inline]
    pub fn monotonic_raw_now() -> Timespec {
        get_time(libc::CLOCK_MONOTONIC_RAW)
    }

    #[inline]
    pub fn monotonic_now() -> Timespec {
        get_time(libc::CLOCK_MONOTONIC)
    }

    #[inline]
    pub fn monotonic_coarse_now() -> Timespec {
        get_time(libc::CLOCK_MONOTONIC_COARSE)
    }

    #[inline]
    fn get_time(clock: libc::clockid_t) -> Timespec {
        let mut t = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let errno = unsafe { libc::clock_gettime(clock, &mut t) };
        if errno != 0 {
            panic!(
                "failed to get clocktime, err {}",
                io::Error::last_os_error()
            );
        }
        Timespec::new(t.tv_sec, t.tv_nsec as _)
    }
}

/// A measurement of a monotonically increasing clock.
/// It's similar and meant to replace `std::time::Instant`,
/// for providing extra features.
#[derive(Copy, Clone, Debug)]
pub enum Instant {
    Monotonic(Timespec),
    MonotonicCoarse(Timespec),
}

impl Instant {
    pub fn now() -> Instant {
        Instant::Monotonic(monotonic_now())
    }

    pub fn now_coarse() -> Instant {
        Instant::MonotonicCoarse(monotonic_coarse_now())
    }

    pub fn elapsed(&self) -> Duration {
        match *self {
            Instant::Monotonic(t) => {
                let now = monotonic_now();
                Instant::elapsed_duration(now, t)
            }
            Instant::MonotonicCoarse(t) => {
                let now = monotonic_coarse_now();
                Instant::elapsed_duration_coarse(now, t)
            }
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        duration_to_sec(self.elapsed())
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        match (*self, earlier) {
            (Instant::Monotonic(later), Instant::Monotonic(earlier)) => {
                Instant::elapsed_duration(later, earlier)
            }
            (Instant::MonotonicCoarse(later), Instant::MonotonicCoarse(earlier)) => {
                Instant::elapsed_duration_coarse(later, earlier)
            }
            _ => {
                panic!("duration between different types of Instants.");
            }
        }
    }

    /// It is similar to `duration_since`, but it won't panic when `self` is less than `other`,
    /// and `None` will be returned in this case.
    ///
    /// Callers need to ensure that `self` and `other` are same type pf Instants.
    pub fn checked_sub(&self, other: Instant) -> Option<Duration> {
        if *self >= other {
            Some(self.duration_since(other))
        } else {
            None
        }
    }

    pub fn elapsed_duration(later: Timespec, earlier: Timespec) -> Duration {
        if later >= earlier {
            (later - earlier).to_std().unwrap()
        } else {
            panic!(
                "monotonic time jumped back, {:.9} -> {:.9}",
                earlier.sec as f64 + f64::from(earlier.nsec) / NANOSECONDS_PER_SECOND as f64,
                later.sec as f64 + f64::from(later.nsec) / NANOSECONDS_PER_SECOND as f64,
            );
        }
    }

    // It is different from `elapsed_duration`, the resolution here is millisecond.
    // The processors in an SMP system do not start all at exactly the same time
    // and therefore the timer registers are typically running at an offset.
    // Use millisecond resolution for ignoring the error.
    // See more: https://linux.die.net/man/2/clock_gettime
    fn elapsed_duration_coarse(later: Timespec, earlier: Timespec) -> Duration {
        let later_ms = later.sec * MILLISECOND_PER_SECOND
            + i64::from(later.nsec) / NANOSECONDS_PER_MILLISECOND;
        let earlier_ms = earlier.sec * MILLISECOND_PER_SECOND
            + i64::from(earlier.nsec) / NANOSECONDS_PER_MILLISECOND;
        let dur = later_ms - earlier_ms;
        if dur >= 0 {
            Duration::from_millis(dur as u64)
        } else {
            // debug!(
            //     "coarse time jumped back, {:.3} -> {:.3}",
            //     earlier.sec as f64 + f64::from(earlier.nsec) / NANOSECONDS_PER_SECOND as f64,
            //     later.sec as f64 + f64::from(later.nsec) / NANOSECONDS_PER_SECOND as f64
            // );
            Duration::from_millis(0)
        }
    }
}

impl PartialEq for Instant {
    fn eq(&self, other: &Instant) -> bool {
        match (*self, *other) {
            (Instant::Monotonic(this), Instant::Monotonic(other))
            | (Instant::MonotonicCoarse(this), Instant::MonotonicCoarse(other)) => this.eq(&other),
            _ => false,
        }
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Instant) -> Option<Ordering> {
        match (*self, *other) {
            (Instant::Monotonic(this), Instant::Monotonic(other))
            | (Instant::MonotonicCoarse(this), Instant::MonotonicCoarse(other)) => {
                this.partial_cmp(&other)
            }
            // The Order of different types of Instants is meaningless.
            _ => None,
        }
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        match self {
            Instant::Monotonic(t) => Instant::Monotonic(t + TimeDuration::from_std(other).unwrap()),
            Instant::MonotonicCoarse(t) => {
                Instant::MonotonicCoarse(t + TimeDuration::from_std(other).unwrap())
            }
        }
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = self.add(rhs)
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        match self {
            Instant::Monotonic(t) => Instant::Monotonic(t - TimeDuration::from_std(other).unwrap()),
            Instant::MonotonicCoarse(t) => {
                Instant::MonotonicCoarse(t - TimeDuration::from_std(other).unwrap())
            }
        }
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = self.sub(rhs)
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Sub;
    use std::time::{Duration, Instant};

    #[test]
    fn t_elapsed() {
        let instant = Instant::now();
        std::thread::sleep(Duration::from_millis(500));
        let cost1 = instant.elapsed().as_millis();
        std::thread::sleep(Duration::from_millis(500));
        let cost2 = instant.elapsed().as_millis().sub(cost1);
        std::thread::sleep(Duration::from_millis(500));
        let cost3 = instant.elapsed().as_millis().sub(cost1 + cost2);

        println!("{},{},{}", cost1, cost2, cost3)
    }
}
