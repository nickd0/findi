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
- [ ] NetBIOS host name resolution
  - branch `netbios_lookup`
- [ ] Create a custom Event enum that encapsulates termion Events/Keys and custom events like ModalYes etc
- [ ] Tests setup
- [x] Program stop control
- [x] New search from within UI (`cidr_input_edit`)
- [x] Non-private range/CIDR validation, validate size of range
- [x] Filter and sort results by all hosts or only live hosts
- [ ] Filter results by hostname
- [ ] Cleanup

## v0.2.0
- [ ] Query-wide TCP port scan results
- [ ] Refactor event queue to something similar to the tui-rs [example](https://github.com/fdehau/tui-rs/blob/master/examples/util/event.rs)
- [ ] Individual result overview popup modal
- [ ] Additional TCP port scans of individual entry
- [ ] Clipboard copy functionality
- [ ] Routing, Page system
- [ ] Keyboard shortcuts and help menu

## v0.3.0
- [ ] User settings for scan type, wait times, thread pool size, UI skins!
- [ ] Menu bar
- [ ] Service search: Multicast DNS to discover available services on the network (airplay, spotify connect, etc)
- [ ] Carmen/traceroute integration for non-private queries
- [ ] IPv6 support?
