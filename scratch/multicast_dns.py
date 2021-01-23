import socket

# DNS header:
# 16-bit identifier 0xFEED
msg = bytearray([0xFE, 0XED])

"""
Query flag: 0 | 1 bit
Opcode:     0 | 4 bit uint
AA flag:    0 | 1 bit
Truncation: 0 | 1 bit
Recursion desired: 0 | 1 bit

1 byte total
"""
msg.append(0)

"""
Recursion avail: 0 | 1 bit
Reserved: 0 | 3 bits
Response code: 0 | 4 bits

1 byte
"""
msg.append(0)

"""
Question count: uint16 number of questions
In this case 1, so we append [0x00, 0x01]
"""
msg.extend([0x00, 0x01])

"""
Remaining sections are 3 more 2 byte uints (6 bytes total):
- Answer record count (used in response)
- Authority record count (?)
- Additional record count (?)

All are zero for us, so lets append 6 more bytes
"""
msg.extend([0x00] * 6)

"""
Next up is the question section:
The question for us is the local IP address we just discovered,
which we need to encode to DNS message name notation, where any "dot"
separators are replaced by a count of the number of bytes in the 
preceeding section, terminated with 0. It looks like this:
[num bytes of "www"] | w | w | w |
[num bytes of "example"] | e | x | a | m | p | l | e |
[num bytes of "com"] | c | o | m |
0
"""

hostaddr = "192.168.0.10"
addr_parts = hostaddr.split(".")
for p in addr_parts:
    p_bytes = p.encode()
    msg.append(len(p_bytes))
    msg += p_bytes

msg.append(0)

"""
Next section is QTYPE, a 2-byte code indicating the type
of question we're asking
(https://en.wikipedia.org/wiki/List_of_DNS_record_types)
We're asking a PTR type question since we want a reverse
lookup, that value is 12 (0x0C)
"""
msg.extend([0x00, 0x0C])

"""
Final field is Query class, another 2-byte code.
Its often 1 for Internet ("IN")
see http://www.tcpipguide.com/free/t_DNSNameServerDataStorageResourceRecordsandClasses-3.htm
for shortlist and
https://www.iana.org/assignments/dns-parameters/dns-parameters.xhtml
for long list
"""
msg.extend([0x00, 0x01])

# Create the multicast UDP socket
MC_ADDR = "224.0.0.251"
MC_PORT = 5353
# sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
# # sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)
# # 1:20 pm Changed the above to:
# # sock.sendto(msg, (MC_ADDR, MC_PORT))
# sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
# sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 32)
# sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_LOOP, 1)
# sock.bind((MC_ADDR, MC_PORT))

# 1:23pm now getting:
"""
Traceback (most recent call last):
  File "/Users/nick/code/findi/scratch/multicast_dns.py", line 91, in <module>
    sock.bind(('', 5353))
OSError: [Errno 22] Invalid argument
"""
# 1:28pm remove the send to before bind, and now it runs, but doesn't receive anything

# 1:33pm new approach:
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
sock.bind((MC_ADDR, MC_PORT))


# 1:41pm this SO answer (https://stackoverflow.com/a/52791404/3121367) builds a simple listening script that worked with the following options:
# python multicast_recv.py  --iface='192.168.0.10' --join-mcast-groups '224.0.0.251'  --port 5353 --bind-group '224.0.0.251'


while True:
    print(sock.recv(10240))
