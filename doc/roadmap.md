Roadmap
===

## v0.1.0
- [x] Live host detection with UDP/TCP ping
- [x] Simple centralized state management
- [x] mDNS host name resolution
  - [ ] Handle cases where mDNS resolves but ping failed. Re-ping?
- [x] Thread pool
- [x] Refactor to use Ipv4Addr everywhere
- [x] Get network from available interface if no input
- [x] NetBIOS host name resolution
  - branch `netbios_lookup`
- [ ] Create a custom Event enum that encapsulates termion Events/Keys and custom events like ModalYes etc
- [x] Tests setup
  - [x] Unit
  - ~~[ ] integration with [insta-rs](https://docs.rs/insta/1.7.0/insta/)~~
- [x] Program stop control
- [x] New search from within UI (`cidr_input_edit`)
- [x] Non-private range/CIDR validation, validate size of range
- [x] Filter and sort results by all hosts or only live hosts
- [ ] Cleanup, see `CLEANUP`
- [x] Github actions CI setup
- [x] Custom Result type that uses custom Error with trait `From` to handle errors from various function calls
  - See `CUSTOM_ERR` comments
  - https://github.com/dtolnay/anyhow
  - [relevant r/rust comment](https://www.reddit.com/r/rust/comments/8mbtdt/how_do_i_more_neatly_handle_multiple_different/dznl8o7?utm_source=share&utm_medium=web2x&context=3)

## v0.2.0
- [x] Use [crossterm](https://crates.io/crates/crossterm) backend
  - branch `crossterm`
- [ ] Use terminal tick like in https://github.com/fdehau/tui-rs/blob/master/examples/crossterm_demo.rs
- [x] Query-wide TCP port scan results
- [x] Refactor event queue to something similar to the tui-rs [example](https://github.com/fdehau/tui-rs/blob/master/examples/util/event.rs)
- [x] Individual result overview popup modal
  - [x] Additional TCP port scans of individual entry
  - Branch result_func
- [ ] Filter results by hostname
- [x] Clipboard copy functionality
- [ ] Routing, Page system
- [x] Keyboard shortcuts and help menu
- [x] Query by interface rather than only CIDR
- [x] Default port scan option/filtering
- [x] Stdout only option with no UI
  - branch `cli_args`
  - Options parser with Clap
- [x] Help menu with keyboard shortcuts

## v0.3.0
- [ ] User settings for scan type, wait times, thread pool size, UI skins! (`CONFIG`)
- [ ] Menu bar
- [ ] Mouse event support
- [ ] Service search: Multicast DNS to discover available services on the network (airplay, spotify connect, etc)
- [ ] Carmen/traceroute integration for non-private queries
- [ ] IPv6 support?
- [ ] Use async/await?
  - Tokio?
