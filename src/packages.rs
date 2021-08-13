pub fn count(output: std::process::Output) -> usize {
  // -1 to deal with newline at end of output
  String::from_utf8_lossy(&output.stdout).split('\n').count() - 1
}