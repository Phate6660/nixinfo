use glob::glob;
use std::env;
use std::fs::{read_to_string, File};
use std::io;
use std::process::Command;

mod cpu;
mod distro;
mod environment;
mod packages;
mod shared_functions;
use shared_functions::read;
mod terminal;
mod uptime;

/// Obtain the temp of the CPU in Celsius, only tested on rpi, outputs to a Result<String>
pub fn temp() -> io::Result<String> {
    Ok(format!("{}", read_to_string("/sys/class/thermal/thermal_zone0/temp")?.trim().parse::<f64>().unwrap() / 1000.0))
}

/// Obtain CPU model, outputs to a Result<String>
pub fn cpu() -> io::Result<String> {
    let file = File::open("/proc/cpuinfo")?;
    let model = read_to_string("/sys/firmware/devicetree/base/model");
    fn info(file: File, line: usize) -> io::Result<String> {
        let info = cpu::get(file, line);
        Ok(cpu::format(info).trim().to_string().replace("\n", ""))
    }
    if let Ok(model) = model {
        match model.starts_with("Raspberry") {
            true => info(file, 1),
            false => info(file, 4),
        }
    } else if shared_functions::exit_code("getprop") != 1 {
        info(file, 1)
    } else {
        info(file, 4)
    }
}

/// Obtain name of device, outputs to a string
pub fn device() -> io::Result<String> {
    if shared_functions::exit_code("getprop") != 1 {
        let output_product = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.product.name"])
            .output()
            .expect("");
        let product = String::from_utf8_lossy(&output_product.stdout).trim().to_string();
        let output_model = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.product.model"])
            .output()
            .expect("");
        let model = String::from_utf8_lossy(&output_model.stdout).trim().to_string();
        let output_device = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.product.device"])
            .output()
            .expect("");
        let device = String::from_utf8_lossy(&output_device.stdout).trim().to_string();
        let full = [
            product, 
            " ".to_string(), 
            model, 
            " (".to_string(), 
            device, 
            ")".to_string()
        ].concat();
        Ok(full)
    } else {
        let model = read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .or_else(|_| read_to_string("/sys/firmware/devicetree/base/model"))?;
        Ok(model.trim().replace("\n", ""))
    }
}

/// Obtain the distro name, outputs to a string
pub fn distro() -> io::Result<String> {
    if shared_functions::exit_code("getprop") != 1 {
        let output_distro = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.build.version.release"])
            .output()
            .expect("");
        let mut distro = String::from_utf8_lossy(&output_distro.stdout).trim().to_string();
        distro = ["Android ".to_string(), distro].concat();
        let output_flavor = std::process::Command::new("sh")
            .args(&["-c", "getprop ro.build.flavor"])
            .output()
            .expect("");
        let flavor = String::from_utf8_lossy(&output_flavor.stdout).trim().to_string();
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
pub fn environment() -> io::Result<String> {
    let de = environment::de().unwrap();
    if de == "N/A" {
        Ok(environment::wm().unwrap())
    } else {
        Ok(de)
    }
}

/// Obtain the contents of the env variable specified as an arg, outputs to a string
pub fn env(var: &str) -> Option<String> {
    if shared_functions::exit_code("getprop") != 1 {
        if var == "USER" {
            let output_user = std::process::Command::new("sh")
                .args(&["-c", "whoami"])
                .output()
                .expect("");
            Some(String::from_utf8_lossy(&output_user.stdout).trim().to_string())
        } else {
            Some(
                env::var(var)
                .unwrap_or_else(
                    |_| format!("N/A (could not read ${}, are you sure it's set?)", var)
                    )
                )
        }
    } else {
        Some(
            env::var(var)
            .unwrap_or_else(
                |_| format!("N/A (could not read ${}, are you sure it's set?)", var)
                )
            )
    }
}

fn r#continue(output_check: String) -> io::Result<String> {
    let model = output_check
        .split(':')
        .collect::<Vec<&str>>()[2]
        .trim()
        .to_string();
    if model.starts_with("Advanced Micro Devices, Inc.") {
        Ok(model.split('.').collect::<Vec<&str>>()[1]
            .trim()
            .replace("[", "")
            .replace("]", "")
            .replace("\n", ""))
    } else {
        Ok(model.replace("\n", ""))
    }
}

/// Obtain the name of the GPU, outputs to a string
pub fn gpu() -> io::Result<String> {
    let output = Command::new("sh")
        .args(&["-c", "lspci | grep -I 'VGA\\|Display\\|3D'"])
        .output()?;
    let output_check: String = String::from_utf8_lossy(&output.stdout).to_string();
    if output_check.is_empty() {
        Ok("N/A (could not run lspci/grep, make sure they are installed)".to_string())
    } else {
        r#continue(output_check)
    }
}

/// Obtain the hostname, outputs to a Result<String>
pub fn hostname() -> io::Result<String> {
    if shared_functions::exit_code("getprop") != 1 {
        let output_hostname = std::process::Command::new("sh")
            .args(&["-c", "hostname"])
            .output()
            .expect("");
        Ok(String::from_utf8_lossy(&output_hostname.stdout).trim().to_string())
    } else {
        Ok(read_to_string("/etc/hostname")?.trim().to_string())
    }
}

/// Obtain the kernel version, outputs to a Result<String>
pub fn kernel() -> io::Result<String> {
    Ok(read_to_string("/proc/sys/kernel/osrelease")?
        .trim()
        .to_string()
        .replace("\n", ""))
}

/// Obtain total memory in MBs, outputs to a Result<String>
pub fn memory() -> io::Result<String> {
    const DIVISOR_U64: u64 = 1024;
    const UNIT_MB: &str = "MB";
    
    if shared_functions::exit_code("sysctl") != 1 {
        let file = File::open("/proc/meminfo")?;
        let total_line = shared_functions::line(file, 0); // MemTotal should be on the first line
        let total_vec: Vec<&str> = total_line.split(':').collect();
        let total = total_vec[1].replace("kB", "");
        let total = total.trim().parse::<i64>().unwrap() / 1024;
        Ok(total.to_string() + " MB")
    } else {
        let cmd = format!("sysctl hw.physmem | awk -F  '{{ print $2 }}'");
        let output_memory = std::process::Command::new("sh")
            .args(&["-c", cmd.as_str()])
            .output()
            .expect("");
        let memory = String::from_utf8_lossy(&output_memory.stdout).trim().to_string();
        let size = memory.parse::<u64>().unwrap();
        let out = format!("{} {}", (size / DIVISOR_U64), UNIT_MB);
        Ok(out)
    }
}

// Music info
#[cfg(feature = "music")]
/// Connects to mpd, and obtains music info in the format "artist - album (date) - title", outputs to a String
pub fn music() -> Result<String, Box<dyn std::error::Error>> {
    let mut c = mpd::Client::connect("127.0.0.1:6600")?;
    let song = c.currentsong().unwrap().unwrap();
    let na = "N/A".to_string();
    let tit = song.title.as_ref().unwrap();
    let art = song.tags.get("Artist").unwrap_or(&na);
    let alb = song.tags.get("Album").unwrap_or(&na);
    let dat = song.tags.get("Date").unwrap_or(&na);
    Ok(format!("{} - {} ({}) - {}", art, alb, dat, tit))
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
            let output = Command::new("apk").arg("info").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "apt" => {
            let output = Command::new("apt")
                .args(&["list", "--installed"])
                .output()?;
            Ok(format!("{}", packages::count(output) - 1)) // -1 to deal with "Listing..."
        }
        "dnf" => {
            let output = Command::new("dnf").args(&["list", "installed"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "dpkg" => {
            let output = Command::new("dpkg-query")
                .args(&["-f", "'${binary:Package}\n'", "-W"])
                .output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "eopkg" => {
            let output = Command::new("eopkg").arg("list-installed").output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "flatpak" => {
            let output = Command::new("flatpak").args(&["list"]).output()?;
            Ok(format!("{}", packages::count(output)))
        }
        "pacman" => {
            let output = Command::new("pacman").args(&["-Q", "-q"]).output()?;
            Ok(format!("{}", packages::count(output)))
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
                file_vector.iter().count() - 1,
                list.iter().count()
            ))
        }
        "rpm" => {
            let output = Command::new("rpm").args(&["-q", "-a"]).output()?;
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
pub fn terminal() -> io::Result<String> {
    let id = std::process::id();
    let path = format!("/proc/{}/status", id);
    let process_id = terminal::ppid(File::open(path)?).trim().replace("\n", "");
    let process_name = terminal::name(process_id.clone()).trim().replace("\n", "");
    let info = terminal::info(process_name, process_id).unwrap();
    if info == "systemd" || info.is_empty() {
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
