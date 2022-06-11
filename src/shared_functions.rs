use std::fs::File;
use std::io::{BufReader, Read};

use crate::error::Error;

/// Returns the exit code of `which getprop > /dev/null 2>&1"`
pub fn exit_code() -> i32 {
    let status = std::process::Command::new("sh")
        .args(&["-c", "which getprop > /dev/null 2>&1"])
        .status()
        .expect("");
    status.code().unwrap()
}

pub fn read(file: File) -> Result<String, Error> {
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn line(file: File, line: usize) -> String {
    let contents = read(file).unwrap();
    contents.split('\n').collect::<Vec<&str>>()[line].to_string()
}
