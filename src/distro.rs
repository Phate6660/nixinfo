pub fn exit_code() -> i32 {
    let status = std::process::Command::new("sh")
        .args(&["-c", "which getprop > /dev/null 2>&1"])
        .status()
        .expect("");
    status.code().unwrap()
}

pub fn dist(path: &str) -> std::io::Result<String> {
    let file = std::fs::File::open(path)?;
    let line: String = crate::shared_functions::line(file, 0); // Expects NAME= to be on first line
    let distro_vec: Vec<&str> = line.split('=').collect();
    Ok(String::from(distro_vec[1]))
}
