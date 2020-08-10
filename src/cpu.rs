pub fn get(file: std::fs::File, x: usize) -> String {
    let line = crate::shared_functions::line(file, x);
    let line_vec: Vec<&str> = line.split(':').collect();
    line_vec[1].to_string()
}

pub fn format(info: String) -> String {
    info.replace("(TM)", "")
        .replace("(R)", "")
        .replace("     ", " ")
}
