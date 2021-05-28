use crate::yatp_pool::future_pool::FuturePool;
use crate::yatp_pool::{PoolTicker, YatpPoolBuilder};
use parking_lot::{Condvar, Mutex};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{self, AtomicBool};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A handle to a schedule job.
#[derive(Debug, Default)]
pub struct JobHandle(Arc<AtomicBool>);

impl JobHandle {
    pub fn cancel(&self) {
        self.0.store(true, atomic::Ordering::SeqCst);
    }
}

enum JobType {
    FixedRate {
        f: Box<dyn FnMut() + Send + 'static>,
        rate: Duration,
    },

    FixedDelay {
        f: Box<dyn FnMut() + Send + 'static>,
        delay: Duration,
    },
}

struct Job {
    typ: JobType,
    time: Instant,
    canceled: Arc<AtomicBool>,
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time).reverse()
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Job {}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

struct InnerPool {
    queue: BinaryHeap<Job>,
    shutdown: bool,
}

struct SharedPool {
    inner: Mutex<InnerPool>,
    cvar: Condvar,
}

impl SharedPool {
    fn run(&self, job: Job) {
        let mut inner = self.inner.lock();

        if inner.shutdown {
            return;
        }

        match inner.queue.peek() {
            None => self.cvar.notify_all(),
            Some(e) if e.time >= job.time => self.cvar.notify_all(),
            _ => 0usize,
        };

        inner.queue.push(job);
    }
}

pub struct SchedPool {
    shared: Arc<SharedPool>,
    runner: FuturePool,
}

#[derive(Clone)]
pub struct SchedTicker;

impl PoolTicker for SchedTicker {
    fn on_tick(&mut self) {}
}

impl Drop for SchedPool {
    fn drop(&mut self) {
        self.shared.inner.lock().shutdown = true;
        self.shared.cvar.notify_all();
    }
}

impl SchedPool {
    pub fn new(pool_size: usize, name_prefix: &str) -> Self {
        let shared_pool = Arc::new(SharedPool {
            inner: Mutex::new(InnerPool {
                queue: BinaryHeap::new(),
                shutdown: false,
            }),
            cvar: Condvar::new(),
        });

        let pool = YatpPoolBuilder::new(SchedTicker {})
            .thread_count(pool_size, pool_size)
            .max_tasks(1024)
            .name_prefix(name_prefix)
            .build_future_pool();

        for _ in 0..pool_size {
            let w = Worker {
                shared: shared_pool.clone(),
            };
            pool.spawn(async move {
                w.run();
            });
        }

        Self {
            shared: shared_pool.clone(),
            runner: pool,
        }
    }

    /// Creates and executes a periodic action that becomes enabled first after the given initial delay,
    /// and subsequently with the given period; that is executions will commence after initialDelay then
    /// initialDelay+period,then initialDelay + 2 * period, and so on. If any execution of the task
    /// encounters an exception, subsequent executions are suppressed.Otherwise, the task will only
    /// terminate via cancellation or termination of the executor. If any execution of this task takes
    /// longer than its period,then subsequent executions may start late, but will not concurrently execute
    pub fn schedule_at_fixed_rate<F>(
        &self,
        f: F,
        initial_delay: Duration,
        period: Duration,
    ) -> JobHandle
    where
        F: FnMut() + Send + 'static,
    {
        let canceled = Arc::new(AtomicBool::new(false));
        let job = Job {
            typ: JobType::FixedRate {
                f: Box::new(f),
                rate: period,
            },
            time: Instant::now() + initial_delay,
            canceled: canceled.clone(),
        };
        self.shared.run(job);
        JobHandle(canceled)
    }

    /// Creates and executes a periodic action that becomes enabled first after the given initial delay,
    /// and subsequently with the given delay between the termination of one execution and the commencement
    /// of the next. If any execution of the task encounters an exception, subsequent executions are suppressed.
    /// Otherwise, the task will only terminate
    /// e via cancellation or termination of the executor.
    pub fn schedule_at_fixed_delay<F>(
        &self,
        f: F,
        initial_delay: Duration,
        period: Duration,
    ) -> JobHandle
    where
        F: FnMut() + Send + 'static,
    {
        let canceled = Arc::new(AtomicBool::new(false));
        let job = Job {
            typ: JobType::FixedDelay {
                f: Box::new(f),
                delay: period,
            },
            time: Instant::now() + initial_delay,
            canceled: canceled.clone(),
        };

        self.shared.run(job);
        JobHandle(canceled)
    }
}

struct Worker {
    shared: Arc<SharedPool>,
}

impl Worker {
    fn run(&self) {
        while let Some(job) = self.get_job() {
            let _ = panic::catch_unwind(AssertUnwindSafe(|| self.execute(job)));
        }
    }

    fn get_job(&self) -> Option<Job> {
        enum Need {
            Wait,
            WaitTimeout(Duration),
        }

        let mut inner = self.shared.inner.lock();
        loop {
            let now = Instant::now();

            let need = match inner.queue.peek() {
                None if inner.shutdown => return None,
                None => Need::Wait,
                Some(e) if e.time <= now => break,
                Some(e) => Need::WaitTimeout(e.time - now),
            };

            match need {
                Need::Wait => self.shared.cvar.wait(&mut inner),
                Need::WaitTimeout(t) => {
                    self.shared.cvar.wait_until(&mut inner, now + t);
                }
            };
        }
        Some(inner.queue.pop().unwrap())
    }

    fn execute(&self, job: Job) {
        if job.canceled.load(atomic::Ordering::SeqCst) {
            return;
        }

        match job.typ {
            JobType::FixedRate { mut f, rate } => {
                f();
                let new_job = Job {
                    typ: JobType::FixedRate { f, rate },
                    time: Instant::now() + rate,
                    canceled: job.canceled,
                };
                self.shared.run(new_job);
            }
            JobType::FixedDelay { mut f, delay } => {
                f();
                let new_job = Job {
                    typ: JobType::FixedDelay { f, delay },
                    time: job.time + delay,
                    canceled: job.canceled,
                };
                self.shared.run(new_job);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::sync::{Arc, Barrier};
    use std::time::{Duration, Instant};

    use super::SchedPool;

    const TASKS: usize = 4;

    #[test]
    fn test_fixed_rate() {
        let pool = SchedPool::new(TASKS, "sched-pool");
        let (tx, rx) = channel();

        let now = Instant::now();
        pool.schedule_at_fixed_rate(
            move || {
                tx.send(Instant::now() - now).unwrap();
                std::thread::sleep(Duration::from_millis(200));
            },
            Duration::from_millis(500),
            Duration::from_millis(500),
        );

        println!("{}", rx.recv().unwrap().as_millis());
        println!("{}", rx.recv().unwrap().as_millis());
    }

    #[test]
    fn test_recovery_from_subtask_panic() {
        let pool = SchedPool::new(TASKS, "sched-pool");

        let waiter = Arc::new(Barrier::new(TASKS as usize));
        for _ in 0..TASKS {
            let waiter = waiter.clone();
            pool.schedule_at_fixed_rate(
                move || {
                    waiter.wait();
                    panic!();
                },
                Duration::from_nanos(0),
                Duration::from_nanos(0),
            );
        }

        let (tx, rx) = channel();
        let waiter = Arc::new(Barrier::new(TASKS as usize));
        for _ in 0..TASKS {
            let tx = tx.clone();
            let waiter = waiter.clone();
            pool.schedule_at_fixed_rate(
                move || {
                    waiter.wait();
                    tx.send(1usize).unwrap();
                },
                Duration::from_nanos(0),
                Duration::from_nanos(0),
            );
        }

        assert_eq!(rx.iter().take(TASKS).sum::<usize>(), TASKS);
    }

    #[test]
    fn test_fixed_delay_jobs_stop_after_drop() {
        let pool = Arc::new(SchedPool::new(TASKS, "sched-pool"));
        let (tx, rx) = channel();
        let (tx2, rx2) = channel();

        let mut pool2 = Some(pool.clone());
        let mut i = 0i32;
        pool.schedule_at_fixed_rate(
            move || {
                i += 1;
                tx.send(i).unwrap();
                rx2.recv().unwrap();
                if i == 2 {
                    drop(pool2.take().unwrap())
                }
            },
            Duration::from_millis(500),
            Duration::from_millis(500),
        );

        drop(pool);

        assert_eq!(Ok(1), rx.recv());
        tx2.send(()).unwrap();
        assert_eq!(Ok(2), rx.recv());
        tx2.send(()).unwrap();
        assert!(rx.recv().is_err());
    }

    #[test]
    fn cancellation() {
        let pool = SchedPool::new(TASKS, "sched-pool");
        let (tx, rx) = channel();

        let handle = pool.schedule_at_fixed_rate(
            move || tx.send(()).unwrap(),
            Duration::from_millis(500),
            Duration::from_millis(500),
        );

        rx.recv().unwrap();
        handle.cancel();
        assert!(rx.recv().is_err())
    }
}
