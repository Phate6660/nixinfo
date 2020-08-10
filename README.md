## nixinfo
A lib crate for gathering system info such as cpu, distro, environment, kernel, etc in Rust.

## Currently supported

- CPU model and temperature
- Distro name
- Device name
- Environment (e.g. DE or WM)
- env variables
- Hostname
- Total memory in MBs
- Music info (only mpd is supported, requires `music` feature to be enabled)
- Package counts (managers supported are apk, apt, dnf, dpkg, eopkg, pacman, pip, portage, rpm, and xbps)
- Terminal being used (unless tmux is used, in which case N/A will be outputted because reasons)
- Uptime of device

## TODO

- Obtain used memory in addition to total memory
