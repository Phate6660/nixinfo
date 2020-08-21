## nixinfo
A lib crate for gathering system info such as cpu, distro, environment, kernel, etc in Rust.

To use: `nixinfo = "0.1.9"` in your `Cargo.toml`.

## Currently supported

- CPU model and temperature (Celsius)
  + `nixinfo::cpu()` -> `Result<String>`
  + `nixinfo::temp()` -> `Result<String>`
- Device name
  + `nixinfo::device()` -> `Result<String>`
- Distro name
  + `nixinfo::distro()` -> `Result<String>`
- Environment (e.g. DE or WM)
  + `nixinfo::environment()` -> `String`
- env variables
  + `nixinfo::env("VARIABLE")` -> `String`
- GPU info (requires `lspci` and `grep` to be installed for now until I find a pure rust solution)
  + `nixinfo::gpu()` -> `Result<String>`
- Hostname
  + `nixinfo::hostname()` -> `Result<String>`
- Kernel
  + `nixinfo::kernel()` -> `Result<String>`
- Total memory in MBs
  + `nixinfo::memory()` -> `Result<String>`
- Music info (only mpd is supported, requires `music` feature to be enabled)
  + `nixinfo::music()` -> `String`
- Package counts (managers supported are apk, apt, dnf, dpkg, eopkg, pacman, pip, portage, rpm, and xbps)
  + `nixinfo::packages("manager")` -> `Result<String>`
- Terminal being used (unless tmux is used, in which case N/A will be outputted because reasons)
  + `nixnfo::terminal()` -> `Result<String>`
- Uptime of device
  + `nixinfo::uptime()` -> `Result<String>`

## TODO

- Obtain used memory in addition to total memory
