use fast_log::consts::LogSize;
use log::Level;
use fast_log::init_custom_log;
use fast_log::appender::{LogAppender, FastLogFormatRecord, FastLogRecord};
use fast_log::filter::NoFilter;
use fast_log::plugin::file_split::{FileSplitAppender, RollingType};

pub enum OutputLevel {
    Stdout,
    File,
    All,
}

#[derive(Debug)]
pub struct FastJobLog {
    pub log_level: log::Level,
    pub log_dir_path: std::path::PathBuf,
    pub output_level: OutputLevel,
    pub log_cap: usize,
    pub max_temp_size: LogSize,
}

impl FastJobLog {
    pub fn build_from_config(
        log_level: log::Level,
        log_dir_path: std::path::PathBuf,
        output_level: OutputLevel,
    ) -> Self {
        Self {
            log_level,
            log_dir_path,
            output_level,
            log_cap: 1000,
            max_temp_size: LogSize::MB(1),
        }
    }

    pub fn init(&self) {
        let mut log_appender: Vec<Box<dyn LogAppender>> = vec![];
        match self.output_level {
            OutputLevel::Stdout => log_appender.push(Box::new(StdoutLog {})),
            OutputLevel::File => log_appender.push(Box::new(
                log_appender.push(Box::new(FileSplitAppender::new(
                self.log_dir_path.to_str().unwrap(),
                self.max_temp_size.clone(),
                RollingType::All,
                true,
                1,
            ))); )),
            OutputLevel::All => {
                log_appender.push(Box::new(StdoutLog {}));
                log_appender.push(Box::new(FileSplitAppender::new(
                    self.log_dir_path.to_str().unwrap(),
                    self.max_temp_size.clone(),
                    RollingType::All,
                    true,
                    1,
                )));
            }
        }

        init_custom_log(
            log_appender,
            self.log_cap,
            self.log_level,
            Box::new(NoFilter),
            Box::new(FastLogFormatRecord {}),
        );
    }

    pub fn info(&self, content: String) {
        log::info!(content);
    }

    pub fn warn(&self, content: String) {
        log::warn!(content);
    }

    pub fn debug(&self, content: String) {
        log::debug!(content);
    }
    pub fn error(&self, content: String) {
        log::error!(content);
    }
}


struct StdoutLog {}

impl LogAppender for StdoutLog {
    fn do_log(&self, record: &FastLogRecord) {
        let data;
        match record.level {
            log::Level::Warn | log::Level::Error => {
                data = format!(
                    "{} {} {} - {}  {}\n",
                    &record.now,
                    record.level,
                    record.module_path,
                    record.args,
                    record.format_line()
                );
            }
            _ => {
                data = format!(
                    "{} {} {} - {}\n",
                    &record.now, record.level, record.module_path, record.args
                );
            }
        }
        print!("{}", data);
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use fast_log::appender::{LogAppender, FastLogRecord, FastLogFormatRecord};
    use fast_log::filter::NoFilter;
    use fast_log::{init_custom_log, init_split_log};
    use std::thread::sleep;
    use fast_log::consts::LogSize;
    use fast_log::plugin::file_split::RollingType;


    struct CustomLog {}

    impl LogAppender for CustomLog {
        fn do_log(&self, record: &FastLogRecord) {
            let data;
            match record.level {
                log::Level::Warn | log::Level::Error => {
                    data = format!(
                        "{} {} {} - {}  {}\n",
                        &record.now,
                        record.level,
                        record.module_path,
                        record.args,
                        record.format_line()
                    );
                }
                _ => {
                    data = format!(
                        "{} {} {} - {}\n",
                        &record.now, record.level, record.module_path, record.args
                    );
                }
            }
            print!("{}", data);
        }
    }

    #[test]
    pub fn test_custom() {
        init_custom_log(
            vec![Box::new(CustomLog {})],
            1000,
            log::Level::Info,
            Box::new(NoFilter {}),
            Box::new(FastLogFormatRecord {}),
        );
        log::info!("Commencing yak shaving");
        log::error!("Commencing error");
        sleep(Duration::from_secs(1));
    }

    #[test]
    pub fn test_file_compation() {
        init_split_log(
            "/Users/eliasyao/Desktop/logs/fastjob/",
            1000,
            LogSize::MB(1),
            false,
            RollingType::All,
            log::Level::Info,
            None,
            true,
        );
        for _ in 0..20000 {
            log::info!("Commencing yak shaving");
        }
        sleep(Duration::from_secs(1));
    }
}