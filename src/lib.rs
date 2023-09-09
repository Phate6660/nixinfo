use std::env;
use std::fs::{read_to_string, File};
use std::io::Error;
use std::process::Command;

use glob::glob;
use memory::memory;
use memory::memory_formatter;
use shared_functions::read;

mod cpu;
mod distro;
mod environment;
mod memory;
mod packages;
mod shared_functions;
mod terminal;
mod uptime;

/// Obtain the temp of CPU thermal zones. Outputs to a Result<Vec<(String, String)>>
pub fn temp() -> Result<Vec<(String, String)>, Error> {
    let paths = glob("/sys/class/thermal/thermal_zone*").expect("Failed to read path");
    let mut zone_temps: Vec<(String, String)> = Vec::new();

    for path in paths {
        let path: std::path::PathBuf = path.unwrap();
        let path_str: String = path.as_path().to_string_lossy().to_owned().to_string();
        let zone_name: String = read_to_string(path_str.to_owned() + "/type")?.trim().to_owned();
        let temp: f64 = read_to_string(path_str.to_owned() + "/temp")?
            .trim()
            .parse::<f64>()
            .unwrap()
            / 1000.0;
        zone_temps.push((zone_name, temp.to_string()));
    }
    Ok(zone_temps)
}

/// Obtain CPU model, outputs to a Result<String>
pub fn cpu() -> Result<String, Error> {
    let file = File::open("/proc/cpuinfo")?;
    let model = read_to_string("/sys/firmware/devicetree/base/model");
    fn info(file: File, line: usize) -> Result<String, Error> {
        let info = cpu::get(file, line);
        Ok(cpu::format(info).trim().to_string().replace('\n', ""))
    }
    if let Ok(model) = model {
        match model.starts_with("Raspberry") {
            true => info(file, 1),
            false => info(file, 4),
        }
    } else if shared_functions::exit_code() != 1 {
        info(file, 1)
    } else {
        info(file, 4)
    }
}

/// Obtain name of device, outputs to a string
pub fn device() -> Result<String, Error> {
    if shared_functions::exit_code() != 1 {
        let output_product = std::process::Command::new("sh")
            .args(["-c", "getprop ro.product.name"])
            .output()
            .expect("");
        let product = String::from_utf8_lossy(&output_product.stdout)
            .trim()
            .to_string();
        let output_model = std::process::Command::new("sh")
            .args(["-c", "getprop ro.product.model"])
            .output()
            .expect("");
        let model = String::from_utf8_lossy(&output_model.stdout)
            .trim()
            .to_string();
        let output_device = std::process::Command::new("sh")
            .args(["-c", "getprop ro.product.device"])
            .output()
            .expect("");
        let device = String::from_utf8_lossy(&output_device.stdout)
            .trim()
            .to_string();
        let full = [
            product,
            " ".to_string(),
            model,
            " (".to_string(),
            device,
            ")".to_string(),
        ].concat();
        Ok(full)
    } else {
        let model = read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .or_else(|_| read_to_string("/sys/firmware/devicetree/base/model"))?;
        Ok(model.trim().replace('\n', ""))
    }
}

/// Obtain the distro name, outputs to a string
pub fn distro() -> Result<String, Error> {
    if shared_functions::exit_code() != 1 {
        let output_distro = std::process::Command::new("sh")
            .args(["-c", "getprop ro.build.version.release"])
            .output()
            .expect("");
        let mut distro = String::from_utf8_lossy(&output_distro.stdout)
            .trim()
            .to_string();
        distro = ["Android ".to_string(), distro].concat();
        let output_flavor = std::process::Command::new("sh")
            .args(["-c", "getprop ro.build.flavor"])
            .output()
            .expect("");
        let flavor = String::from_utf8_lossy(&output_flavor.stdout)
            .trim()
            .to_string();
        let full = [distro, " (".to_string(), flavor, ")".to_string()].concat();
        Ok(full)
    } else {
        let distro = distro::dist("/bedrock/etc/os-release")
            .or_else(|_| distro::dist("/etc/os-release"))
            .or_else(|_| distro::dist("/usr/lib/os-release"))?;
        Ok(distro)
    }
}

/// Obtains the name of the user's DE or WM, outputs to a string
pub fn environment() -> Result<String, Error> {
    let de = environment::de().unwrap();
    if de == "N/A" {
        Ok(environment::wm().unwrap())
    } else {
        Ok(de)
    }
}

/// Obtain the contents of the env variable specified as an arg, outputs to a string
pub fn env(var: &str) -> Option<String> {
    if shared_functions::exit_code() != 1 {
        if var == "USER" {
            let output_user = std::process::Command::new("sh")
                .args(["-c", "whoami"])
                .output()
                .expect("");
            Some(String::from_utf8_lossy(&output_user.stdout).trim().to_string())
        } else {
            Some(env::var(var).unwrap_or_else(|_| {format!("N/A (could not read ${}, are you sure it's set?)", var)}))
        }
    } else {
        Some(
            env::var(var).unwrap_or_else(|_| {format!("N/A (could not read ${}, are you sure it's set?)", var)}))
    }
}

/// Obtain a vector containing the names of the GPUs, outputs to a `Result<Vec<String>>`
pub fn gpu() -> Result<Vec<String>, Error> {
    let mut gpu_dev_vec: Vec<String> = Vec::new();
    let mut gpu_vendor_vec: Vec<String> = Vec::new();
    for entry in glob("/sys/class/drm/card?/device/").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let device_path = format!("{}/device", path.display());
                let vendor_path = format!("{}/vendor", path.display());
                let device_id = read_to_string(device_path)?;
                let vendor_id = read_to_string(vendor_path)?;
                let true_device_id = device_id.trim();
                let true_vendor_id = vendor_id.trim();
                let searchable_device_id = true_device_id.split('x').collect::<Vec<&str>>()[1];
                let searchable_vendor_id = true_vendor_id.split('x').collect::<Vec<&str>>()[1];
                gpu_dev_vec.push(searchable_device_id.to_string());
                gpu_vendor_vec.push(searchable_vendor_id.to_string());
            },
            Err(e) => println!("{:?}", e),
        }
    }
    let all_ids = read_to_string("/usr/share/hwdata/pci.ids")?;
    let mut gpu_names_vec: Vec<String> = Vec::new();
    let mut found_vendor = false;
    let id_vec: Vec<&str> = all_ids.split('\n').collect();
    for line in id_vec {
        if gpu_vendor_vec.iter().any(|vendor| line.starts_with(vendor)) {
            found_vendor = true;
        }
        if found_vendor {
            if gpu_dev_vec.iter().any(|device| line.contains(device)) {
                let split_line = line.split("  ").collect::<Vec<&str>>()[1];
                gpu_names_vec.push(split_line.trim().to_string());
                found_vendor=false;
            } else {
                continue;
            }
        }
    }
    Ok(gpu_names_vec)
}

/// Obtain the hostname, outputs to a Result<String>
pub fn hostname() -> Result<String, Error> {
    if shared_functions::exit_code() != 1 {
        let output_hostname = std::process::Command::new("sh")
            .args(["-c", "hostname"])
            .output()
            .expect("");
        Ok(String::from_utf8_lossy(&output_hostname.stdout)
            .trim()
            .to_string())
    } else {
        Ok(read_to_string("/etc/hostname")?.trim().to_string())
    }
}

/// Obtain the kernel version, outputs to a Result<String>
pub fn kernel() -> Result<String, Error> {
    Ok(read_to_string("/proc/sys/kernel/osrelease")?
        .trim()
        .to_string()
        .replace('\n', ""))
}

// Obtain free physical memory in MBs, outputs to a Result<String>
pub fn memory_free() -> Result<String, Error> {
    let (size, unit) = memory("MemFree").unwrap();
    memory_formatter(size, unit)
}

// Obtain available memory for applications (without swap) in MBs, outputs to a Result<String>
pub fn memory_available() -> Result<String, Error> {
    let (size, unit) = memory("MemAvailable").unwrap();
    memory_formatter(size, unit)
}

/// Obtain total memory in MBs, outputs to a Result<String>
pub fn memory_total() -> Result<String, Error> {
    let (size, unit) = memory("MemTotal").unwrap();
    memory_formatter(size, unit)
}

// Obtain used memory in MBs by subtracting free memory from total memory, outputs to a Result<string>
pub fn memory_used() -> Result<String, Error> {
    let (total_size, unit) = memory("MemTotal").unwrap();
    let (free_size, _) = memory("MemFree").unwrap();

    memory_formatter(total_size - free_size, unit)
}

// Music info
/// Connects to mpd, and obtains music info in the format "artist - album (date) - title", outputs to a String
#[cfg(feature = "music_mpd")]
pub fn music() -> Result<String, Box<dyn std::error::Error>> {
    let mut c = mpd::Client::connect("127.0.0.1:6600")?;
    let song = c.currentsong().unwrap().unwrap();
    let na = "N/A".to_string();
    let tit = song.title.as_ref().unwrap();
    let art = song.artist.unwrap_or(na);
    // To find the correct index of `Vec<(String, String)>` containing the proper metadata
    // we're looking for, we iterate over it and match the position of the element containing
    // metadata string we're looking for in the first element of the `tuple`.
    let alb_index = &song.tags.iter().position(|x| x.0 == "Album").unwrap();
    let dat_index = &song.tags.iter().position(|x| x.0 == "Date").unwrap();
    let alb = &song.tags[*alb_index].1;
    let dat = &song.tags[*dat_index].1;
    Ok(format!("{} - {} ({}) - {}", art, alb, dat, tit))
}

#[cfg(feature = "music_playerctl")]
/// Gets music info from `playerctl` in the format "artist - album - title", outputs to a String
pub fn music() -> Result<String, Box<dyn std::error::Error>> {
    let child = std::process::Command::new("playerctl")
        .args(&["metadata", "-f", "{{artist}} - {{album}} - {{title}}"])
        .output();
    let output;
    if child.is_ok() {
        output = String::from_utf8_lossy(&child.unwrap().stdout).to_string();
    } else {
        output = String::from("N/A (failed to collect output from `playerctl`)");
    }
    Ok(output)
}

/// If neither `music_mpd` nor `music_playerctl` is used.
#[cfg(not(feature = "music_mpd"))]
#[cfg(not(feature = "music_playerctl"))]
pub fn music() -> String {
    "N/A (music feature must be used to pull in the mpd dependency)".to_string()
}

/// Obtain list of packages based on what manager is given as an arg, outputs to a string
pub fn packages(manager: &str) -> Result<String, Error> {
    match manager {
        "apk" => {
            let output = Command::new("apk").arg("info").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "apt" | "dpkg" => {
            let file = File::open("/var/lib/dpkg/status")?;
            let file_contents = read(file)?;
            let content_split = file_contents.split('\n');
            let mut installed_vec: Vec<&str> = Vec::new();
            for line in content_split {
                if line.contains("install ok installed") {
                    installed_vec.push(line);
                }
            }
            let count = installed_vec.len();
            Ok(format!("{}", count))
        }
        "dnf" => {
            let output = Command::new("dnf").args(["list", "installed"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "eopkg" => {
            let output = Command::new("eopkg").arg("list-installed").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "flatpak" => {
            let output = Command::new("flatpak").args(["list"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "pacman" => {
            let mut list: Vec<String> = Vec::new();
            for entry in glob("/var/lib/pacman/local/*").expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => list.push(path.display().to_string()),
                    Err(e) => println!("{:?}", e),
                }
            }
            let total = list.len() - 1; // -1 to deal with `ALPM_DB_VERSION` file
            Ok(format!("{}", total))
        }
        "pip" => {
            let output = Command::new("pip").arg("list").output()?;
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
                file_vector.len() - 1,
                list.len()
            ))
        }
        "rpm" => {
            let output = Command::new("rpm").args(["-q", "-a"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "xbps" => {
            let output = Command::new("xbps-query").arg("-l").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        _ => Ok(format!(
            "N/A ({} is not supported, please file a bug to get it added!)",
            manager
        )),
    }
}

/// Obtain the name of the terminal being used, outputs to a Result<String>
pub fn terminal() -> Result<String, Error> {
    let id = std::process::id();
    let path = format!("/proc/{}/status", id);
    let process_id = terminal::ppid(File::open(path)?).trim().replace('\n', "");
    let process_name = terminal::name(process_id.clone()).trim().replace('\n', "");
    let info = terminal::info(process_name, process_id).unwrap();
    if info == "systemd" || info.is_empty() {
        Ok("N/A (could not determine the terminal, this could be an issue of using tmux)".to_string())
    } else {
        Ok(info)
    }
}

/// Obtains the current uptime of the system, outputs to a Result<String>
pub fn uptime() -> Result<String, Error> {
    let raw_uptime = read_to_string("/proc/uptime")?;
    let uptime_vec: Vec<&str> = raw_uptime.split('.').collect();
    let uptime = uptime_vec[0].parse::<i64>().unwrap();
    let (days, hours, minutes) = uptime::duration(uptime);
    Ok(format!("{} {} {}", days, hours, minutes).trim().to_string())
}
