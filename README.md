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
findi -c 192.168.0.0/24
```
*Note* that the current limitation on network size is 4096 IP addresses, ie `/20` in CIDR notation.

To use without the TUI interface and print live hosts to stdout:
```bash
findi -n
```

Specify a TCP port range with `-p`:
```bash
findi -p 22,80,443,5009
```

See all options with `-h`

## Keys

While in the TUI, you can see the help menu by pressing `?`.

Use tab/shift tab to cycle focus between the hosts table, query input, and search filter boxes. Or press the underlined character in the title of the box.

While focused on the filter tab, use the left and right arrows (or space bar) to cycle between filters.

While in the hosts table, you can use the up and down arrows to browse hosts, the spacebar and page up/down keys to jump through the list, or use the 'j' and 'k' key to move down and up, respectively (as in Vim). Using shift plus 'j' or 'k' jumps 10 hosts at a time.

While highlighting a specific host:

- Press 'c' to copy the IP address to local clipboard (currently works on MacOS and Linux if you have Xorg installed).
- Press 'C' to copy the host name.
- Press enter  to open the host info menu. You can see detailed information and start a TCP port scan of this host.
