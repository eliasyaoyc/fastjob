use std::time::Duration;

use chrono::{DateTime, Local};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::thread::LocalKey;
use std::time::SystemTime;

/// Open log file with append mode. Creates a new log file if it doesn't exist
fn open_log_file(path: impl AsRef<Path>) -> io::Result<File> {
    let path = path.as_ref();
    let parent = path.parent().ok_or_else(|| {
        Error::new(
            ErrorKind::Other,
            "Unable to get parent directory of log file",
        )
    })?;

    if !parent.is_dir() {
        fs::create_dir_all(parent)?;
    }
    OpenOptions::new().append(true).create(true).open(path)
}

/// A trait that describes a file rotation operation.
pub trait Rotator: Send {
    /// Check if the option is enabled in configuration.
    /// Return true if the `rotator` is valid.
    fn is_enabled(&self) -> bool;

    /// Call by operator, initializes the states of rotators.
    fn prepare(&mut self, file: &File) -> io::Result<()>;

    /// Return if the file need to be rotated.
    fn should_rotator(&self) -> bool;

    /// Call by operator, update rotators' state while the operator try to write some data.
    fn on_write(&mut self, data: &[u8]) -> io::Result<()>;

    /// Call by operator, update rotator's state while the operator execute a rotation.
    fn on_rotator(&mut self) -> io::Result<()>;
}

/// This `FileLogger` will iterate over a series of `Rotators`,
/// once the context trigger the `Rotator`, it will execute a rotation.
///
/// After rotating, the original log file would be renamed to "{original name}.{%Y-%m-%d-%H:%M:%S}".
/// Note: log file will *not* be compressed or otherwise modified.
pub struct RotatingFileLogger {
    path: PathBuf,
    file: File,
    rename: Box<dyn Send + Fn(&Path) -> io::Result<PathBuf>>,
    rotators: Vec<Box<dyn Rotator>>,
}

/// Builder for `RotatingFileLogger`.
pub struct RotatingFileLoggerBuilder {
    path: PathBuf,
    rename: Box<dyn Send + Fn(&Path) -> io::Result<PathBuf>>,
    rotators: Vec<Box<dyn Rotator>>,
}

impl RotatingFileLoggerBuilder {
    pub fn builder<F>(rename: F) -> Self
    where
        F: 'static + Send + Fn(&Path) -> io::Result<PathBuf>,
    {
        Self {
            path: Default::default(),
            rename: Box::new(rename),
            rotators: vec![],
        }
    }

    pub fn add_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = path.as_ref().to_path_buf();
        self
    }

    pub fn add_rotator<R: 'static + Rotator>(mut self, rotator: R) -> Self {
        if rotator.is_enabled() {
            self.rotators.push(Box::new(rotator));
        }
        self
    }

    pub fn build(mut self) -> io::Result<RotatingFileLogger> {
        let file = open_log_file(&self.path)?;

        for rotator in self.rotators.iter_mut() {
            rotator.prepare(&file)?;
        }

        Ok(RotatingFileLogger {
            path: self.path,
            file,
            rename: self.rename,
            rotators: self.rotators,
        })
    }
}

impl Write for RotatingFileLogger {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for rotator in self.rotators.iter_mut() {
            rotator.on_write(bytes)?;
        }
        self.file.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        for rotator in self.rotators.iter() {
            if rotator.should_rotator() {
                self.file.flush()?;

                let new_path = (self.rename)(&self.path)?;
                fs::rename(&self.path, &new_path)?;
                self.file = open_log_file(&self.path)?;

                for rotator in self.rotators.iter_mut() {
                    rotator.on_rotator()?;
                }
                return Ok(());
            }
        }
        self.file.flush()
    }
}

impl Drop for RotatingFileLogger {
    fn drop(&mut self) {
        let _ = self.file.flush();
    }
}

pub struct RotateByTime {
    rotation_timespan: Duration,
    next_rotation_time: Option<SystemTime>,
}

impl RotateByTime {
    pub fn new(rotation_timespan: Duration) -> Self {
        Self {
            rotation_timespan,
            next_rotation_time: None,
        }
    }

    fn next_rotation_time(begin: SystemTime, duration: Duration) -> io::Result<SystemTime> {
        begin
            .checked_add(duration)
            .ok_or_else(|| Error::new(ErrorKind::Other, "Next rotation time is out of range. "))
    }
}

impl Rotator for RotateByTime {
    fn is_enabled(&self) -> bool {
        !self.rotation_timespan.as_nanos() == 0
    }

    fn prepare(&mut self, file: &File) -> io::Result<()> {
        let metadata = file.metadata()?;
        let created = metadata.created();
        let accessed = metadata.accessed();
        let birth = match (created, accessed) {
            (Err(_), a) => a?,
            (Ok(c), Ok(a)) if a < c => a,
            (Ok(c), _) => c,
        };
        self.next_rotation_time = Some(Self::next_rotation_time(birth, self.rotation_timespan)?);
        Ok(())
    }

    fn should_rotator(&self) -> bool {
        assert!(self.next_rotation_time.is_some());
        Local::now() > DateTime::<Local>::from(self.next_rotation_time.unwrap())
    }

    fn on_write(&mut self, data: &[u8]) -> io::Result<()> {
        Ok(())
    }

    fn on_rotator(&mut self) -> io::Result<()> {
        assert!(self.next_rotation_time.is_some());
        self.next_rotation_time = Some(Self::next_rotation_time(
            SystemTime::now(),
            self.rotation_timespan,
        )?);
        Ok(())
    }
}

pub struct RotateBySize {
    rotation_size: u64,
    file_size: u64,
}

impl RotateBySize {
    pub fn new(rotation_size: u64) -> Self {
        Self {
            rotation_size,
            file_size: 0,
        }
    }
}

impl Rotator for RotateBySize {
    fn is_enabled(&self) -> bool {
        self.rotation_size != 0
    }

    fn prepare(&mut self, file: &File) -> io::Result<()> {
        self.file_size = file.metadata()?.len();
        Ok(())
    }

    fn should_rotator(&self) -> bool {
        self.file_size > self.rotation_size
    }

    fn on_write(&mut self, data: &[u8]) -> io::Result<()> {
        self.file_size += data.len() as u64;
        Ok(())
    }

    fn on_rotator(&mut self) -> io::Result<()> {
        self.file_size = 0;
        Ok(())
    }
}
