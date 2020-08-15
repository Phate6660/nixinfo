## nixinfo
A lib crate for gathering system info such as cpu, distro, environment, kernel, etc in Rust.

To use: `nixinfo = "0.1.7"` in your `Cargo.toml`.

Every function will output to a String for easy usage.

## Currently supported

- CPU model and temperature
  + `nixinfo::cpu()` and `nixinfo::temp()` respectively
- Distro name
  + `nixinfo::distro()`
- Device name
  + `nixinfo::device()`
- Environment (e.g. DE or WM)
  + `nixinfo::environment()`
- env variables
  + `nixinfo::env("VARIABLE")`
- GPU info (requires `lspci` and `grep` to be installed for now until I find a pure rust solution)
  + `nixinfo::gpu()`
- Hostname
  + `nixinfo::hostname()`
- Total memory in MBs
  + `nixinfo::memory()`
- Music info (only mpd is supported, requires `music` feature to be enabled)
  + `nixinfo::music()`
- Package counts (managers supported are apk, apt, dnf, dpkg, eopkg, pacman, pip, portage, rpm, and xbps)
  + `nixinfo::packages("manager")`
- Terminal being used (unless tmux is used, in which case N/A will be outputted because reasons)
  + `nixnfo::terminal()`
- Uptime of device
  + `nixinfo::uptime()`

## TODO

- Obtain used memory in addition to total memory
