findi
======
Probe your local network for live hosts and open ports. Check out the [roadmap](doc/roadmap.md) for future features. Currently only supports local IPv4 addresses.

![Animated gif of findi network tool](doc/recording_v010.gif)

# Build
You'll need `rustup` and `cargo` installed locally to build. See [here](https://doc.rust-lang.org/cargo/getting-started/installation.html) for instructions.

## With UI
```bash
cargo build --features "ui"
```
Or to run with a specific IPv4 CIDR range:
```bash
cargo run --features "ui" -- 192.168.0.0/24
```
# Usage
To run with your active IPv4 interface, simply run the command with no arguments:
```bash
findi
```

To run with a specific, private subnet range in CIDR notation, run:
```bash
findi 192.168.0.0/24
```
Note that the current limitation on network size is 4096 IP addresses, ie `/20` in CIDR notation.
