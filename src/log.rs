pub mod log {
    use std::collections::VecDeque;
    use std::fs::File;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use chrono::prelude::*;

    #[derive(Copy, Clone)]
    pub enum LogLevel {
        EMERG = 0,
        ALERT = 1,
        CRIT = 2,
        ERR = 3,
        WARNING = 4,
        NOTICE = 5,
        INFO = 6,
        DEBUG = 7,
        TRACE = 8,
    }
    pub struct LogEntry {
        pub timestamp: String,
        pub level: LogLevel,
        pub sender: String,
        pub message: String,
    }

    impl LogEntry {
        pub fn new(level: LogLevel, sender: String, message: String) -> Self {
            LogEntry {
                level,
                sender,
                message,
                timestamp: {
                    let now = Utc::now();
                    format!("{}",now.format("%Y-%m-%d %H:%M:%S.%.3f"))
                },
            }
        }
    }
    impl ToString for LogEntry {
        fn to_string(&self) -> String {
            let level = match self.level {
                LogLevel::EMERG => "EMERG",
                LogLevel::ALERT => "ALERT",
                LogLevel::CRIT => "CRIT",
                LogLevel::ERR => "ERR",
                LogLevel::WARNING => "WARNING",
                LogLevel::NOTICE => "NOTICE",
                LogLevel::INFO => "INFO",
                LogLevel::DEBUG => "DEBUG",
                LogLevel::TRACE => "TRACE",
            };
            format!(
                "[{}] [{}] [{}]: {}",
                self.timestamp, level, self.sender, self.message
            )
        }
    }

    #[derive(Clone)]
    pub struct Logger {
        log_queue: Arc<Mutex<VecDeque<LogEntry>>>,
        log_console_level: LogLevel,
        log_file_level: LogLevel,
        log_file_path: String,
    }

    impl Logger {
        fn new(l_file_path: String, l_file_level: LogLevel, c_file_level: LogLevel) -> Self {
            Logger {
                log_queue: Arc::new(Mutex::new(VecDeque::new())),
                log_console_level: c_file_level,
                log_file_level: l_file_level,
                log_file_path: l_file_path,
            }
        }

        fn init_logger(&self) {
            let log_queue = self.log_queue.clone();
            let log_file_path = self.log_file_path.clone();
            let log_file_level = self.log_file_level;
            let log_console_level = self.log_console_level;

            thread::spawn(move || {
                let mut log_file = File::create(&log_file_path).unwrap();
                loop {
                    let mut queue = log_queue.lock().unwrap();
                    if let Some(entry) = queue.pop_front() {
                        let entry_level: i32 = entry.level as i32;
                        if entry_level <= (log_file_level as i32) {
                            log_file.write_all(entry.to_string().as_bytes()).unwrap();
                            log_file.flush().unwrap();
                        }
                        if entry_level <= (log_console_level as i32) {
                            println!("{}", entry.to_string());
                        }
                    }
                }
            });
        }

        pub fn write(&self, entry: LogEntry) {
            let mut queue = self.log_queue.lock().unwrap();
            queue.push_back(entry);
        }
    }

    static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

    pub fn log() -> Logger {
        match LOGGER.lock().unwrap().as_ref() {
            Some(logger) => logger.clone(),
            None => {
                println!("Error: Log service uninitialized");
                panic!("Logger is not initialized");
            }
        }
    }

    pub fn init_log(dir_log_path: String, file_log_level: LogLevel, console_log_level: LogLevel) {
        match LOGGER.lock().unwrap().as_ref() {
            Some(_logger) => {
                println!("Error: Log service uninitialized");
                panic!("Logger is not initialized");
            },
            None => {
                let dt = Utc::now().timestamp().to_string();
                let mut file_log_path: String = dir_log_path;
                file_log_path.push_str("/log_grade_p_");
                file_log_path.push_str(&dt);
                file_log_path.push_str("log");
                *LOGGER.lock().unwrap() =
                    Some(Logger::new(file_log_path,
                                    file_log_level,
                                    console_log_level));
            }
        }
    }
}
