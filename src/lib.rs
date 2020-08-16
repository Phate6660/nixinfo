use glob::glob;
use std::env;
use std::fs::{metadata, read_to_string, File};
use std::process::Command;

mod cpu;
mod distro;
mod environment;
mod packages;
mod shared_functions;
use shared_functions::{line, read};
mod terminal;
mod uptime;

/// Obtain the temp of the CPU, only tested on rpi, outputs to a string
pub fn temp() -> String {
    if metadata("/sys/class/thermal/thermal_zone0/temp").is_ok() {
        let file = File::open("/sys/class/thermal/thermal_zone0/temp").unwrap();
        let raw_temp = read(file).unwrap().trim().parse::<i64>().unwrap();
        format!("{}", raw_temp / 1000)
    } else {
        "N/A (could not read /sys/class/thermal/thermal_zone0/temp)".to_string()
    }
}

/// Obtain CPU model, outputs to a string
pub fn cpu() -> String {
    if metadata("/proc/cpuinfo").is_ok() {
        let file = File::open("/proc/cpuinfo").unwrap();
        if metadata("/sys/firmware/devicetree/base/model").is_ok() {
            if read_to_string("/sys/firmware/devicetree/base/model")
                .unwrap()
                .starts_with("Raspberry")
            {
                let info = cpu::get(file, 1); // Line 2
                cpu::format(info).trim().to_string().replace("\n", "")
            } else {
                let info = cpu::get(file, 4); // Line 5
                cpu::format(info)
            }
        } else {
            let info = cpu::get(file, 4); // Line 5
            cpu::format(info)
        }
    } else {
        "N/A (could not read /proc/cpuinfo)".to_string()
    }
}

/// Obtain name of device, outputs to a string
pub fn device() -> String {
    if metadata("/sys/devices/virtual/dmi/id/product_name").is_ok() {
        read_to_string("/sys/devices/virtual/dmi/id/product_name").unwrap().trim().to_string().replace("\n", "")
    } else if metadata("/sys/firmware/devicetree/base/model").is_ok() {
        read_to_string("/sys/firmware/devicetree/base/model").unwrap().trim().to_string().replace("\n", "")
    } else {
        "N/A (could not obtain name of device)".to_string()
    }
}

/// Obtain the distro name, outputs to a string
pub fn distro() -> String {
    if metadata("/bedrock/etc/os-release").is_ok() {
        distro::dist("/bedrock/etc/os-release")
    } else if metadata("/etc/os-release").is_ok() {
        distro::dist("/etc/os-release")
    } else if metadata("/usr/lib/os-release").is_ok() {
        distro::dist("/usr/lib/os-release")
    } else {
        "N/A (could not obtain distro name, please file a bug as your os-release file may just be in a weird place)".to_string()
    }
}

/// Obtains the name of the user's DE or WM, outputs to a string
pub fn environment() -> String {
    let de = environment::de();
    if de == "N/A" {
        environment::wm()
    } else {
        de
    }
}

/// Obtain the contents of the env variable specified as an arg, outputs to a string
pub fn env(var: &str) -> String {
    // $SHELL and $USER are set automatically, the only env variable it would fail on is $EDITOR
    env::var(var).unwrap_or_else(|_| "N/A (could not read $EDITOR, are you sure it's set?)".to_string())
}

/// Obtain the name of the GPU, outputs to a string
pub fn gpu() -> String {
    let output = Command::new("sh")
        .args(&["-c", "lspci | grep -I 'VGA\\|Display\\|3D'"])
        .output()
        .expect("Could not run `lspci | grep -I 'VGA\\|Display\\|3D'`, are you sure `lspci` and `grep` are installed?");
    let model = String::from_utf8_lossy(&output.stdout).split(':').collect::<Vec<&str>>()[2].trim().to_string();
    if model.starts_with("Advanced Micro Devices, Inc.") {
        model.split('.').collect::<Vec<&str>>()[1].trim().replace("[", "").replace("]", "").to_string().replace("\n", "")
    } else {
        model.replace("\n", "")
    }
}

/// Obtain the hostname, outputs to a string
pub fn hostname() -> String {
    if metadata("/etc/hostname").is_ok() {
        read_to_string("/etc/hostname").unwrap().trim().to_string()
    } else {
        "N/A (could not read /etc/hostname)".to_string()
    }
}

/// Obtain the kernel version, outputs to a string
pub fn kernel() -> String {
    if metadata("/proc/sys/kernel/osrelease").is_ok() {
        read_to_string("/proc/sys/kernel/osrelease").unwrap().trim().to_string().replace("\n", "")
    } else {
        "N/A (could not obtain kernel version)".to_string()
    }
}

/// Obtain total memory in MBs, outputs to a string
pub fn memory() -> String {
    if metadata("/proc/meminfo").is_ok() {
        let file = File::open("/proc/meminfo").unwrap();
        let total_line = line(file, 0); // MemTotal should be on the first line
        let total_vec: Vec<&str> = total_line.split(':').collect();
        let total = total_vec[1].replace("kB", "");
        let total = total.trim().parse::<i64>().unwrap() / 1024;
        total.to_string() + " MB"
    } else {
        "N/A (could not read /proc/meminfo)".to_string()
    }
}

// Music info
#[cfg(feature = "music")]
/// Connects to mpd, and obtains music info in the format "artist - album (date) - title", outputs to a string
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
/// If the music feature is enabled, it connects to mpd, and obtains music info in the format "artist - album (date) - title", outputs to a string
pub fn music() -> String {
    "N/A (music feature must be used to pull in the mpd dependency)".to_string()
}

/// Obtain list of packages based on what manager is given as an arg, outputs to a string
pub fn packages(manager: &str) -> String {
    match manager {
        "apk" => {
            let output = Command::new("apk")
                .arg("info")
                .output()
                .expect("Could not run apk.");
            format!("{}", packages::count(output))
        }
        "apt" => {
            let output = Command::new("apt")
                .args(&["list", "--installed"])
                .output()
                .expect("Could not run apt.");
            format!("{}", packages::count(output) - 1) // -1 to deal with "Listing..."
        }
        "dnf" => {
            let output = Command::new("dnf")
                .args(&["list", "installed"])
                .output()
                .expect("Could not run dnf.");
            format!("{}", packages::count(output))
        }
        "dpkg" => {
            let output = Command::new("dpkg-query")
                .args(&["-f", "'${binary:Package}\n'", "-W"])
                .output()
                .expect("Could not run dpkg-query.");
            format!("{}", packages::count(output))
        }
        "eopkg" => {
            let output = Command::new("eopkg")
                .arg("list-installed")
                .output()
                .expect("Could not run eopkg.");
            format!("{}", packages::count(output))
        }
        "pacman" => {
            let output = Command::new("pacman")
                .args(&["-Q", "-q"])
                .output()
                .expect("Could not run pacman.");
            format!("{}", packages::count(output))
        }
        "pip" => {
            let output = Command::new("pip")
                .arg("list")
                .output()
                .expect("Could not run pip.");
            format!("{}", packages::count(output) - 2) // -2 to deal with header lines in output
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

            format!(
                "{} (explicit), {} (total)",
                file_vector.iter().count() - 1,
                list.iter().count()
            )
        }
        "rpm" => {
            let output = Command::new("rpm")
                .args(&["-q", "-a"])
                .output()
                .expect("Could not run rpm.");
            format!("{}", packages::count(output))
        }
        "xbps" => {
            let output = Command::new("xbps-query")
                .arg("list-installed")
                .output()
                .expect("Could not run xbps-query.");
            format!("{}", packages::count(output))
        }
        _ => format!("N/A ({} is not supported, please file a bug to get it added!)", manager),
    }
}

/// Obtain the name of the terminal being used, outputs to a string
pub fn terminal() -> String {
    let id = std::process::id();
    let path = format!("/proc/{}/status", id);
    if metadata(path.clone()).is_ok() {
        let process_id = terminal::ppid(File::open(path).unwrap())
            .trim()
            .replace("\n", "");
        let process_name = terminal::name(process_id.clone())
            .trim()
            .replace("\n", "");
        let info = terminal::info(process_name, process_id);
        if info == "systemd" || info == "" {
            "N/A (could not determine the terminal, this could be an issue of using tmux)".to_string()
        } else {
            info
        }
    } else {
        format!("N/A (could not read {})", path)
    }
}

/// Obtains the current uptime of the system, outputs to a string
pub fn uptime() -> String {
    if metadata("/proc/uptime").is_ok() {
        let raw_uptime = read_to_string("/proc/uptime").unwrap();
        let uptime_vec: Vec<&str> = raw_uptime.split('.').collect();
        let uptime = uptime_vec[0].parse::<i64>().unwrap();
        let (days, hours, minutes) = uptime::duration(uptime);
        format!("{} {} {}", days, hours, minutes).trim().to_string()
    } else {
        "N/A (could not obtain read /proc/uptime)".to_string()
    }
}
