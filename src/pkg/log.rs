use crate::validate;
use std::fs::{File,OpenOptions};
use std::io::{self, Write};

pub struct Log{
    log_path:String,
    file: File,
}

impl Log{
    pub fn new(log_path:&str) -> Self {
        Log{log_path:log_path.to_string(),file: init_log(log_path).unwrap()}
    }

    pub fn write_all(& mut self,content:&[u8]) {
        self.file = init_log(&self.log_path).unwrap();
        if let Err(err) = self.file.write_all(content) {
            println!("[Pingway] write log error: {err}")
        };
    }
}

fn init_log(log_path:&str) -> Result<File,io::Error>{
    let acccess_log_dir =  validate::get_parent_directory(log_path).unwrap_or_default();
    validate::ensure_dir_exists(acccess_log_dir.to_str().unwrap()).unwrap();
    let access_log_file = match OpenOptions::new().
    create(true)
    // .write(true)
    .append(true)
    // .read(true)
    .open(log_path) {
        Ok(f) => f,
        Err(err) => panic!("[Pingway] file path: {log_path} ,open access log error: {err}")
    };
    Ok(access_log_file)
}