use std::env;
use std::io::Error;

pub fn de() -> Result<String, Error> {
    Ok(env::var("XDG_DESKTOP_SESSION")
        .or_else(|_| env::var("XDG_CURRENT_DESKTOP"))
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "N/A".to_string()))
}

pub fn wm() -> Result<String, Error> {
    let path = format!("{}/.xinitrc", env::var("HOME").unwrap());
    let file = std::fs::File::open(path)?;
    let contents = crate::shared_functions::read(file).unwrap();
    let line = contents.lines().last().unwrap();
    Ok(line.split(' ').last().unwrap().to_string())
}
