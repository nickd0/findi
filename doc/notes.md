findi notes
===

# Steps

## Networking
- [x] Initial ping concept
- [x] host resolution 
- [ ] Port scanning
- [ ] Multi threaded UDP socket
  - Currently trying to give each thread a copy of the socket, which fine for sending but not receiving,
  - So maybe the solution is to use an `mpsc` channel and transaction IDs of the packets to resolve the ping
    - nvm this wont work because the recv is an error not a udp packet
      - see https://github.com/rustasync/runtime/issues/45#issuecomment-505334600
  
  - Each Ping thread creates a thread-safe UDPSocket
    - Can we just bind to port 0 in every thread?
  - use https://doc.rust-lang.org/std/sync/atomic/ to count threads
    - or a thread pool https://docs.rs/threadpool/1.8.1/threadpool/

## Data
- Wanted to try a sort of single source of truth where a `Mutex<Vec<Host>>` is used by all threads
  - Problem is that each ping thread blocks used of the entire `Vec` so the pinging process blocks every time
TODO:
- [ ] Build a single stream data source that uses `mpsc::channel`s to maintain a central state
  - A sort of redux-y way of managing state
  - Take a functional approach with an immutable state
    - see [r/rust post](https://www.reddit.com/r/rust/comments/8hh8r3/how_would_you_handle_application_state_in_rust/dymu6er?utm_source=share&utm_medium=web2x&context=3)

## UI
- [ ] Simple TUI UI
- [ ] App screen design
- [ ] Inputs
- [ ] Settings popups
  - Local storage of settings
- [ ] Customizable skins
- Handle keyboard and mouse events from a separate thread and dispatch actions to the appstate store?
- A chart of host resolution and ping roundtrip times?
- Traceroute geo ping mapper
- Turn this into more of a network utility tool
  - When certain cli arguments are given it will automatically enter local network finder mode or traceroute mode
  - Or you can choose if no args are given
- Or after a scan, you can highlight over an entry and the following options are available
  - Port scan
  - Traceroute (only or non-local ips)
  - Mention https://freegeoip.app/ in the docs

## App orchestration
- [ ] Networking and UI threads
- [ ] Tokio? (main async `fn`)


## TODO
- Consider crossbeam for concurrency https://github.com/crossbeam-rs/crossbeam

- service search
  - Once you have a list of addresses, you can search for services either by entering the service dns name or by choosing from a preset list (which can be  updated from the configs) ie "Spotify Connect": _spotify-connect._tcp.local, "Airplay": _airplay._tcp.local
    - Or you can search by this from the beginning and it will start with a network search of  this service and display the table. You can then do a full ping scan if you want
