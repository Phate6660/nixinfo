use std::fs::File;
use std::io::{BufReader, Read};

/// Returns the exit code of (`which {} > /dev/null 2>&1"`, cmd)
pub fn exit_code(cmd: &str) -> i32 {
    let command = format!("which {} > /dev/null 2>&1", cmd);
    let status = std::process::Command::new("sh")
        .args(&["-c", command.as_str()])
        .status()
        .expect("");
    status.code().unwrap()
}

pub fn read(file: File) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn line(file: File, line: usize) -> String {
    let contents = read(file).unwrap();
    contents.split('\n').collect::<Vec<&str>>()[line].to_string()
}
