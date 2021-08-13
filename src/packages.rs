pub fn count(output: std::process::Output) -> usize {
  // -1 to deal with newline at end of output
  output.stdout.iter().filter(|&&i| i == b'\n').count() - 1
}