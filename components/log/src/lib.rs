mod file_log;
mod formatter;
pub mod log_macro;

use std::env;
use std::fmt;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;

use log::{self, SetLoggerError};
use slog::{self, slog_o, Drain, FnValue, Key, OwnedKVList, PushFnValue, Record, KV};
use slog_async::{Async, OverflowStrategy};
use slog_term::{Decorator, PlainDecorator, RecordDecorator};

use self::file_log::{RotateBySize, RotateByTime, RotatingFileLogger, RotatingFileLoggerBuilder};

pub use slog::{FilterFn, Level};
use std::fmt::Arguments;
use std::time::Duration;

// The suffix appended to the end of rotated log files by datetime log rotator
// Warning: Diagnostics service parses log files by file name format.
//          Remember to update the corresponding code when suffix layout is changed.
pub const DATETIME_ROTATE_SUFFIX: &str = "%Y-%m-%d-%H:%M:%S%.f";

// Default is 128.
// Extended since blocking is set, and we don't want to block very often.
const SLOG_CHANNEL_SIZE: usize = 10240;

// Default is DropAndReport.
// It is not desirable to have dropped logs in our use case.
const SLOG_CHANNEL_OVERFLOW_STRATEGY: OverflowStrategy = OverflowStrategy::Block;
const TIMESTAMP_FORMAT: &str = "%Y/%m/%d %H:%M:%S%.3f %:z";

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(usize::max_value());

#[derive(Clone, Debug)]
pub enum LogFormat {
    Text,
    Json,
}

/// Makes a thread name with an additional tag inherited from the current thread.
#[macro_export]
macro_rules! thd_name {
    ($name:expr) => {{
        $crate::get_tag_from_thread_name()
            .map(|tag| format!("{}::{}", $name, tag))
            .unwrap_or_else(|| $name.to_owned())
    }};
}

pub fn get_tag_from_thread_name() -> Option<String> {
    thread::current()
        .name()
        .and_then(|name| name.split("::").skip(1).last())
        .map(From::from)
}

pub fn init_log<D>(
    drain: D,
    level: Level,
    use_async: bool,
    init_stdlog: bool,
    mut disabled_targets: Vec<String>,
    slow_threshold: u64,
) -> Result<(), SetLoggerError>
    where
        D: Drain + Send + 'static,
        <D as Drain>::Err: std::fmt::Display,
{
    // Set the initial log level used by the Drains
    LOG_LEVEL.store(level.as_usize(), Ordering::Relaxed);

    // Only for debug purpose, so use environment instead of configuration file.
    if let Ok(extra_modules) = env::var("FASTJOB_DISABLE_LOG_TARGETS") {
        disabled_targets.extend(extra_modules.split(',').map(ToOwned::to_owned));
    }

    let filter = move |record: &Record| {
        if !disabled_targets.is_empty() {
            // Here get the highest level module name to check.
            let module = record.module().splitn(2, "::").next().unwrap();
            disabled_targets.iter().all(|target| target != module)
        } else {
            true
        }
    };

    let logger = if use_async {
        let drain = Async::new(LogAndFuse(drain))
            .chan_size(SLOG_CHANNEL_SIZE)
            .overflow_strategy(SLOG_CHANNEL_OVERFLOW_STRATEGY)
            .thread_name(thd_name!("slogger"))
            .build()
            .filter_level(level)
            .fuse();
        let drain = SlowLogFilter {
            threshold: slow_threshold,
            inner: drain,
        };
        let filtered = drain.filter(filter).fuse();
        slog::Logger::root(filtered, slog_o!())
    } else {
        let drain = LogAndFuse(Mutex::new(drain).filter_level(level));
        let drain = SlowLogFilter {
            threshold: slow_threshold,
            inner: drain,
        };
        let filtered = drain.filter(filter).fuse();
        slog::Logger::root(filtered, slog_o!())
    };

    set_global_logger(level, init_stdlog, logger)
}

pub fn set_global_logger(
    level: Level,
    init_stdlog: bool,
    logger: slog::Logger,
) -> Result<(), SetLoggerError> {
    slog_global::set_global(logger);
    if init_stdlog {
        let a = log::logger();

        slog_global::redirect_std_log(Some(level))?;
        grpcio::redirect_log();
    }

    Ok(())
}

/// Constructs a new file writer which outputs log to a file at the specified path.
/// The file writer rotates for the specified timespan.
pub fn file_writer<N>(
    path: impl AsRef<Path>,
    rotation_timespan: Duration,
    rotation_size: u64,
    rename: N,
) -> io::Result<BufWriter<RotatingFileLogger>>
    where
        N: 'static + Send + Fn(&Path) -> io::Result<PathBuf>,
{
    let logger = BufWriter::new(
        RotatingFileLoggerBuilder::builder(rename)
            .add_path(path)
            .add_rotator(RotateByTime::new(rotation_timespan))
            .add_rotator(RotateBySize::new(rotation_size))
            .build()?,
    );
    Ok(logger)
}

/// Constructs a new terminal writer which outputs logs to stderr.
pub fn term_writer() -> io::Stderr {
    io::stderr()
}

/// Formats output logs to "FastJob Log Format".
pub fn text_format<W>(io: W) -> FastJobFormat<PlainDecorator<W>>
    where
        W: io::Write,
{
    let decorator = PlainDecorator::new(io);
    FastJobFormat::new(decorator)
}

/// Formats output logs to JSON format.
pub fn json_format<W>(io: W) -> slog_json::Json<W>
    where
        W: io::Write,
{
    slog_json::Json::new(io)
        .set_newlines(true)
        .set_flush(true)
        .add_key_value(slog_o!(
            "message" => PushFnValue(|record, ser| ser.emit(record.msg())),
            "caller" => PushFnValue(|record, ser| ser.emit(format_args!(
                "{}:{}",
                Path::new(record.file())
                    .file_name()
                    .and_then(|path| path.to_str())
                    .unwrap_or("<unknown>"),
                record.line(),
            ))),
            "level" => FnValue(|record| get_unified_log_level(record.level())),
            "time" => FnValue(|_| chrono::Local::now().format(TIMESTAMP_FORMAT).to_string()),
        ))
        .build()
}

pub fn get_level_by_string(lv: &str) -> Option<Level> {
    match &*lv.to_owned().to_lowercase() {
        "critical" => Some(Level::Critical),
        "error" => Some(Level::Error),
        // We support `warn` due to legacy.
        "warning" | "warn" => Some(Level::Warning),
        "debug" => Some(Level::Debug),
        "trace" => Some(Level::Trace),
        "info" => Some(Level::Info),
        _ => None,
    }
}

// The `to_string()` function of `slog::Level` produces values like `erro` and `trce` instead of
// the full words. This produces the full word.
pub fn get_string_by_level(lv: Level) -> &'static str {
    match lv {
        Level::Critical => "critical",
        Level::Error => "error",
        Level::Warning => "warning",
        Level::Debug => "debug",
        Level::Trace => "trace",
        Level::Info => "info",
    }
}

// Converts `slog::Level` to unified log level format.
fn get_unified_log_level(lv: Level) -> &'static str {
    match lv {
        Level::Critical => "FATAL",
        Level::Error => "ERROR",
        Level::Warning => "WARN",
        Level::Info => "INFO",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
    }
}

pub fn convert_slog_level_to_log_level(lv: Level) -> log::Level {
    match lv {
        Level::Critical | Level::Error => log::Level::Error,
        Level::Warning => log::Level::Warn,
        Level::Debug => log::Level::Debug,
        Level::Trace => log::Level::Trace,
        Level::Info => log::Level::Info,
    }
}

pub fn convert_log_level_to_slog_level(lv: log::Level) -> Level {
    match lv {
        log::Level::Error => Level::Error,
        log::Level::Warn => Level::Warning,
        log::Level::Debug => Level::Debug,
        log::Level::Trace => Level::Trace,
        log::Level::Info => Level::Info,
    }
}

pub fn get_log_level() -> Option<Level> {
    Level::from_usize(LOG_LEVEL.load(Ordering::Relaxed))
}

pub fn set_log_level(new_level: Level) {
    LOG_LEVEL.store(new_level.as_usize(), Ordering::SeqCst)
}

pub struct FastJobFormat<D>
    where
        D: Decorator,
{
    decorator: D,
}

impl<D> FastJobFormat<D>
    where
        D: Decorator,
{
    pub fn new(decorator: D) -> Self {
        Self { decorator }
    }
}

impl<D> Drain for FastJobFormat<D>
    where
        D: Decorator,
{
    type Ok = ();
    type Err = io::Error;

    fn log(&self, record: &Record<'_>, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        if record.level().as_usize() <= LOG_LEVEL.load(Ordering::Relaxed) {
            self.decorator.with_record(record, values, |decorator| {
                write_log_header(decorator, record)?;
                write_log_msg(decorator, record)?;
                write_log_fields(decorator, record, values)?;

                decorator.start_whitespace()?;
                writeln!(decorator)?;

                decorator.flush()?;
                Ok(())
            })?;
        }
        Ok(())
    }
}

struct LogAndFuse<D>(D);

impl<D> Drain for LogAndFuse<D>
    where
        D: Drain,
        <D as Drain>::Err: std::fmt::Display,
{
    type Ok = ();
    type Err = slog::Never;

    fn log(&self, record: &Record<'_>, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        if record.level().as_usize() <= LOG_LEVEL.load(Ordering::Relaxed) {
            if let Err(e) = self.0.log(record, values) {
                let fatal_drainer = Mutex::new(text_format(term_writer())).ignore_res();
                fatal_drainer.log(record, values).unwrap();
                let fatal_logger = slog::Logger::root(fatal_drainer, slog_o!());
                slog::slog_crit!(
                    fatal_logger,
                    "logger encountered error";
                    "err" => %e,
                )
            }
        }
        Ok(())
    }
}

/// Filters logs with operation cost lower than threshold. Otherwise output logs to inner drainer.
struct SlowLogFilter<D> {
    threshold: u64,
    inner: D,
}

impl<D> Drain for SlowLogFilter<D>
    where
        D: Drain<Ok=(), Err=slog::Never>,
{
    type Ok = ();
    type Err = slog::Never;

    fn log(&self, record: &Record<'_>, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        Ok(())
    }
}

struct SlowCostSerializer {
    // None means input record without key `takes`
    cost: Option<u64>,
}

impl slog::ser::Serializer for SlowCostSerializer {
    fn emit_arguments(&mut self, _key: Key, _val: &fmt::Arguments<'_>) -> slog::Result {
        Ok(())
    }

    fn emit_u64(&mut self, key: Key, val: u64) -> slog::Result {
        if key == "takes" {
            self.cost = Some(val);
        }
        Ok(())
    }
}

/// Special struct for slow log cost serializing
pub struct LogCost(pub u64);

impl slog::Value for LogCost {
    fn serialize(
        &self,
        _record: &Record,
        key: Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_u64(key, self.0)
    }
}

/// Dispatches logs to a normal `Drain` or a slow-log specialized `Drain` by tag
pub struct LogDispatcher<N: Drain, S: Drain> {
    normal: N,
    slow: Option<S>,
}

impl<N: Drain, S: Drain> LogDispatcher<N, S> {
    pub fn new(normal: N, slow: Option<S>) -> Self {
        Self { normal, slow }
    }
}

impl<N, S> Drain for LogDispatcher<N, S>
    where
        N: Drain<Ok=(), Err=io::Error>,
        S: Drain<Ok=(), Err=io::Error>,
{
    type Ok = ();
    type Err = io::Error;

    fn log(&self, record: &Record<'_>, values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        let tag = record.tag();
        if self.slow.is_some() && tag.starts_with("slow_log") {
            self.slow.as_ref().unwrap().log(record, values)
        } else {
            self.normal.log(record, values)
        }
    }
}

/// Writes log header to decorator.
fn write_log_header(decorator: &mut dyn RecordDecorator, record: &Record<'_>) -> io::Result<()> {
    decorator.start_timestamp()?;
    write!(
        decorator,
        "[{}]",
        chrono::Local::now().format(TIMESTAMP_FORMAT)
    )?;

    decorator.start_whitespace()?;
    write!(decorator, " ")?;

    decorator.start_level()?;
    write!(decorator, "[{}]", get_unified_log_level(record.level()))?;

    decorator.start_whitespace()?;
    write!(decorator, " ")?;

    // Write source file info.
    decorator.start_msg()?;
    if let Some(path) = Path::new(record.file())
        .file_name()
        .and_then(|path| path.to_str())
    {
        write!(decorator, "[")?;
        formatter::write_file_name(decorator, path)?;
        write!(decorator, ":{}]", record.line())?;
    } else {
        write!(decorator, "[<unknown>]")?;
    }

    Ok(())
}

/// Writes log message to decorator.
fn write_log_msg(decorator: &mut dyn RecordDecorator, record: &Record<'_>) -> io::Result<()> {
    decorator.start_whitespace()?;
    write!(decorator, " ")?;

    decorator.start_msg()?;
    write!(decorator, "[")?;
    let msg = format!("{}", record.msg());
    formatter::write_escaped_str(decorator, &msg)?;
    write!(decorator, "]")?;

    Ok(())
}

/// Writes log fields to decorator.
fn write_log_fields(
    decorator: &mut dyn RecordDecorator,
    record: &Record<'_>,
    values: &OwnedKVList,
) -> io::Result<()> {
    let mut serializer = Serializer::new(decorator);

    record.kv().serialize(record, &mut serializer)?;

    values.serialize(record, &mut serializer)?;

    serializer.finish();

    Ok(())
}

struct Serializer<'a> {
    decorator: &'a mut dyn RecordDecorator,
}

impl<'a> Serializer<'a> {
    fn new(decorator: &'a mut dyn RecordDecorator) -> Self {
        Self { decorator }
    }

    fn write_whitespace(&mut self) -> io::Result<()> {
        self.decorator.start_whitespace()?;
        write!(self.decorator, " ")?;
        Ok(())
    }

    fn finish(self) {}
}

impl<'a> Drop for Serializer<'a> {
    fn drop(&mut self) {}
}

impl<'a> slog::Serializer for Serializer<'a> {
    fn emit_none(&mut self, key: Key) -> slog::Result {
        self.emit_arguments(key, &format_args!("None"))
    }

    fn emit_arguments(&mut self, key: Key, val: &fmt::Arguments<'_>) -> slog::Result {
        self.write_whitespace()?;

        // Write key
        write!(self.decorator, "[")?;
        self.decorator.start_key()?;
        formatter::write_escaped_str(&mut self.decorator, key as &str)?;

        // Write separator
        self.decorator.start_separator()?;
        write!(self.decorator, "=")?;

        // Write value
        let value = format!("{}", val);
        self.decorator.start_value()?;
        formatter::write_escaped_str(self.decorator, &value)?;
        self.decorator.reset()?;
        write!(self.decorator, "]")?;
        Ok(())
    }
}
