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

## UI
- [ ] Simple TUI UI
- [ ] App screen design
- [ ] Inputs
- [ ] Settings popups

## App orchestration
- [ ] Networking and UI threads
- [ ] Tokio? (main async `fn`)
