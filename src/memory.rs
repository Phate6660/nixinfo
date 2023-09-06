use std::fs::{self};
use std::io::{self, Error};

pub fn memory(mem_value: &str) -> Result<(u64, String), Error> {
    let no_mem_info_error_msg: &str = &format!("No memoryinfo in {} line!", mem_value);
    let no_memline_found_error_msg: &str = &format!("No {} line found in /proc/meminfo", mem_value);
    const MEMINFO: &str = "/proc/meminfo";
    const UNIT: [&str; 5] = ["kB", "MB", "GB", "TB", "PB"];
    const SEPARATOR_COLON: &str = ":";
    const EMPTY_STRING: &str = "";

    pub trait ToIoResult<T> {
        fn to_io_result(self) -> io::Result<T>;
    }

    impl<T, E: ToString> ToIoResult<T> for Result<T, E> {
        fn to_io_result(self) -> io::Result<T> {
            match self {
                Ok(x) => Ok(x),
                Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            }
        }
    }

    let meminfo = fs::read_to_string(MEMINFO)?;
    for line in meminfo.lines() {
        if line.starts_with(mem_value) {
            let mut rsplit = line.rsplit(SEPARATOR_COLON);
            let size = match rsplit.next() {
                Some(x) => x
                    .replace(UNIT[0], EMPTY_STRING)
                    .trim()
                    .parse::<u64>()
                    .to_io_result()?,
                None => Err(io::Error::new(
                    io::ErrorKind::Other,
                    no_memline_found_error_msg,
                ))?,
            };
            let unit: String = if size <= 999 {
                "MB".to_string()
            } else if size >= 1000 {
                "GB".to_string()
            } else {
                "MB".to_string()
            };

            return Ok((size, unit));
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, no_mem_info_error_msg))?
}

pub fn memory_formatter(mem_result: u64, unit: String) -> Result<String, Error> {
    let final_result = match unit.as_str() {
        "MB" => mem_result / 1000,
        "GB" => mem_result / (1000*1000),
        _ => mem_result / 1000
    };
    let output = format!("{} {}", final_result, unit);
    Ok(output)
}
