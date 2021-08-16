//pub fn count(output: std::process::Output) -> usize {
  // -1 to deal with newline at end of output
//  output.stdout.iter().filter(|&&i| i == b'\n').count() - 1
//}


pub fn count(output: std::process::Output) -> usize {
  let raw_list = String::from_utf8_lossy(&output.stdout);
  let list: Vec<&str> = raw_list.split('\n').collect();
  list.iter().count() - 1 // -1 to deal with newline at end of output
}