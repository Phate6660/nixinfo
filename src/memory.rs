use std::fs::{self};
use std::io::{self, Error};

pub fn memory(mem_value: &str) -> Result<u64, Error> {
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
            return Ok(size);
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, no_mem_info_error_msg))?
}

pub fn memory_formatter(mem_result: Result<u64, Error>) -> Result<String, Error> {
    if mem_result.is_err() {
        return Err(mem_result.unwrap_err());
    }

    const DIVISOR_U64: u64 = 1024;
    const UNIT_MB: &str = "MB";
    return Ok(format!(
        "{} {}",
        (mem_result.unwrap() / DIVISOR_U64),
        UNIT_MB
    ));
}
