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

## UI
- [ ] Simple TUI UI
- [ ] App screen design
- [ ] Inputs
- [ ] Settings popups

## App orchestration
- [ ] Networking and UI threads
- [ ] Tokio? (main async `fn`)
