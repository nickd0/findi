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

## Keys

Use tab/shift tab to cycle focus between the hosts table, query input, and search filter boxes.

While focused on the filter tab, use the up and down arrows to cycle between filters.

While in the hosts table, you can use the up and down arrows to browse hosts, the spacebar and page up/down keys to jump through the list, or use the 'j' and 'k' key to move down and up, respectively (as in Vim). Using shift plus 'j' or 'k' jumps 10 hosts at a time.
