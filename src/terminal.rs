use crate::shared_functions::line;
use std::fs::File;
use std::io::Error;

pub fn ppid(file: File) -> String {
    let ppid = line(file, 6);
    ppid.split(':').collect::<Vec<&str>>()[1].to_string()
}

pub fn name(ppid: String) -> String {
    let path = format!("/proc/{}/status", ppid.trim());
    let file = File::open(path).unwrap();
    let line = line(file, 0);
    line.split(':').collect::<Vec<&str>>()[1].to_string()
}

pub fn info(process_name: String, process_id: String) -> Result<String, Error> {
    if process_name.ends_with("sh")
        || process_name == "ion"
        || process_name == "screen"
        || process_name == "tmux"
        || process_name.starts_with("tmux")
    {
        let path = format!("/proc/{}/status", process_id);
        let new_ppid = ppid(File::open(path)?).trim().replace('\n', "");
        let new_name = name(new_ppid.clone()).trim().replace('\n', "");
        if new_name.ends_with("sh")
            || new_name == "ion"
            || new_name == "screen"
            || new_name == "tmux"
            || new_name.starts_with("tmux")
        {
            let path = format!("/proc/{}/status", new_ppid);
            let new_ppid = ppid(File::open(path)?).trim().replace('\n', "");
            Ok(name(new_ppid).trim().replace('\n', ""))
        } else {
            Ok(new_name.trim().replace('\n', ""))
        }
    } else {
        Ok(process_name.trim().replace('\n', ""))
    }
}
