use glob::glob;
use std::env;
use std::fs::{metadata, read_to_string, File};
use std::io;
use std::process::Command;

mod cpu;
mod distro;
mod environment;
mod packages;
mod shared_functions;
use shared_functions::{line, read};
mod terminal;
mod uptime;

/// Obtain the temp of the CPU in Celsius, only tested on rpi, outputs to a Result<String>
pub fn temp() -> io::Result<String> {
    Ok(format!("{}", read_to_string("/sys/class/thermal/thermal_zone0/temp")?.trim().parse::<f64>().unwrap() / 1000.0))
}

/// Obtain CPU model, outputs to a Result<String>
pub fn cpu() -> io::Result<String> {
    let file = File::open("/proc/cpuinfo")?;
    if metadata("/sys/firmware/devicetree/base/model").is_ok() {
        if read_to_string("/sys/firmware/devicetree/base/model")
            .unwrap()
            .starts_with("Raspberry")
        {
            let info = cpu::get(file, 1); // Line 2
            Ok(cpu::format(info).trim().to_string().replace("\n", ""))
        } else {
            let info = cpu::get(file, 4); // Line 5
            Ok(cpu::format(info).trim().to_string().replace("\n", ""))
        }
    } else {
        let info = cpu::get(file, 4); // Line 5
        Ok(cpu::format(info).trim().to_string().replace("\n", ""))
    }
}

/// Obtain name of device, outputs to a string
pub fn device() -> io::Result<String> {
    let model = read_to_string("/sys/devices/virtual/dmi/id/product_name").or_else(|_| read_to_string("/sys/firmware/devicetree/base/model"))?;
    Ok(model.trim().replace("\n", ""))
}

/// Obtain the distro name, outputs to a string
pub fn distro() -> io::Result<String> {
    let distro = distro::dist("/bedrock/etc/os-release")
        .or_else(|_| distro::dist("/etc/os-release"))
        .or_else(|_| distro::dist("/usr/lib/os-release"))?;
    Ok(distro)
}

/// Obtains the name of the user's DE or WM, outputs to a string
pub fn environment() -> String {
    let de = environment::de();
    if de == "N/A" {
        environment::wm().unwrap()
    } else {
        de
    }
}

/// Obtain the contents of the env variable specified as an arg, outputs to a string
pub fn env(var: &str) -> String {
    env::var(var).unwrap_or_else(|_| format!("N/A (could not read ${}, are you sure it's set?)", var))
}

/// Obtain the name of the GPU, outputs to a string
pub fn gpu() -> io::Result<String> {
    let output = Command::new("sh")
        .args(&["-c", "lspci | grep -I 'VGA\\|Display\\|3D'"])
        .output()?;
    let model = String::from_utf8_lossy(&output.stdout).split(':').collect::<Vec<&str>>()[2].trim().to_string();
    if model.starts_with("Advanced Micro Devices, Inc.") {
        Ok(model.split('.').collect::<Vec<&str>>()[1].trim().replace("[", "").replace("]", "").replace("\n", ""))
    } else {
        Ok(model.replace("\n", ""))
    }
}

/// Obtain the hostname, outputs to a Result<String>
pub fn hostname() -> io::Result<String> {
    Ok(read_to_string("/etc/hostname")?.trim().to_string())
}

/// Obtain the kernel version, outputs to a Result<String>
pub fn kernel() -> io::Result<String> {
    Ok(read_to_string("/proc/sys/kernel/osrelease")?.trim().to_string().replace("\n", ""))
}

/// Obtain total memory in MBs, outputs to a Result<String>
pub fn memory() -> io::Result<String> {
    let file = File::open("/proc/meminfo")?;
    let total_line = line(file, 0); // MemTotal should be on the first line
    let total_vec: Vec<&str> = total_line.split(':').collect();
    let total = total_vec[1].replace("kB", "");
    let total = total.trim().parse::<i64>().unwrap() / 1024;
    Ok(total.to_string() + " MB")
}

// Music info
#[cfg(feature = "music")]
/// Connects to mpd, and obtains music info in the format "artist - album (date) - title", outputs to a String
pub fn music() -> String {
    let mut c = mpd::Client::connect("127.0.0.1:6600").unwrap();
    let song: mpd::Song = c.currentsong().unwrap().unwrap();
    let na = "N/A".to_string();
    let tit = song.title.as_ref().unwrap();
    let art = song.tags.get("Artist").unwrap_or(&na);
    let alb = song.tags.get("Album").unwrap_or(&na);
    let dat = song.tags.get("Date").unwrap_or(&na);
    format!("{} - {} ({}) - {}", art, alb, dat, tit)
}

#[cfg(not(feature = "music"))]
/// If the music feature is enabled, it connects to mpd, and obtains music info in the format "artist - album (date) - title", outputs to a String
pub fn music() -> String {
    "N/A (music feature must be used to pull in the mpd dependency)".to_string()
}

/// Obtain list of packages based on what manager is given as an arg, outputs to a string
pub fn packages(manager: &str) -> io::Result<String> {
    match manager {
        "apk" => {
            let output = Command::new("apk")
                .arg("info")
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "apt" => {
            let output = Command::new("apt")
                .args(&["list", "--installed"])
                .output()?;
            Ok(format!("{}", packages::count(output) - 1)) // -1 to deal with "Listing..."
        }
        "dnf" => {
            let output = Command::new("dnf")
                .args(&["list", "installed"])
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "dpkg" => {
            let output = Command::new("dpkg-query")
                .args(&["-f", "'${binary:Package}\n'", "-W"])
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "eopkg" => {
            let output = Command::new("eopkg")
                .arg("list-installed")
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "pacman" => {
            let output = Command::new("pacman")
                .args(&["-Q", "-q"])
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "pip" => {
            let output = Command::new("pip")
                .arg("list")
                .output()?;
            Ok(format!("{}", packages::count(output) - 2)) // -2 to deal with header lines in output
        }
        "portage" => {
            let content = read(File::open("/var/lib/portage/world").unwrap()).unwrap();
            let file_vector: Vec<&str> = content.split('\n').collect();

            let mut list: Vec<String> = Vec::new();
            for entry in glob("/var/db/pkg/*/*/").expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => list.push(path.display().to_string()),
                    Err(e) => println!("{:?}", e),
                }
            }

            Ok(format!(
                    "{} (explicit), {} (total)",
                    file_vector.iter().count() - 1,
                    list.iter().count()
                ))
        }
        "rpm" => {
            let output = Command::new("rpm")
                .args(&["-q", "-a"])
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "xbps" => {
            let output = Command::new("xbps-query")
                .arg("list-installed")
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        _ => Ok(format!("N/A ({} is not supported, please file a bug to get it added!)", manager)),
    }
}

/// Obtain the name of the terminal being used, outputs to a Result<String>
pub fn terminal() -> io::Result<String> {
    let id = std::process::id();
    let path = format!("/proc/{}/status", id);
    let process_id = terminal::ppid(File::open(path)?)
        .trim()
        .replace("\n", "");
    let process_name = terminal::name(process_id.clone())
        .trim()
        .replace("\n", "");
    let info = terminal::info(process_name, process_id).unwrap();
    if info == "systemd" || info == "" {
        Ok("N/A (could not determine the terminal, this could be an issue of using tmux)".to_string())
    } else {
        Ok(info)
    }
}

/// Obtains the current uptime of the system, outputs to a Result<String>
pub fn uptime() -> io::Result<String> {
    let raw_uptime = read_to_string("/proc/uptime")?;
    let uptime_vec: Vec<&str> = raw_uptime.split('.').collect();
    let uptime = uptime_vec[0].parse::<i64>().unwrap();
    let (days, hours, minutes) = uptime::duration(uptime);
    Ok(format!("{} {} {}", days, hours, minutes).trim().to_string())
}
