use crate::error::Error;

pub fn dist(path: &str) -> Result<String, Error> {
    let file = std::fs::File::open(path)?;
    let line: String = crate::shared_functions::line(file, 0); // Expects NAME= to be on first line
    let distro_vec: Vec<&str> = line.split('=').collect();
    Ok(String::from(distro_vec[1]))
}
