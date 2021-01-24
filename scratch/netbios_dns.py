import socket
import sys
import struct
import string

from multicast_dns import build_dns_header

NBNS_RESP_BASE_LEN = 57
NBNS_RESP_NAME_LEN = 15

"""
Using NetBIOS Name service (NBNS) to resolve names
with unicast UDP packets

DNS packet structure is the same, but the naming convention is weird.
see https://www.ietf.org/rfc/rfc1001.txt section 17.2.
for unicast NetBIOS name destination

NBSTAT is the netbios message used to ask NBNS questions:
https://osqa-ask.wireshark.org/questions/2824/unexplained-netbios-traffic/2851

It seems like this is a pretty antiquated service (RFC is from 1987),
maybe we can find a more modern way to reverse lookup node names for
Windows devices?

We will be using the NBSTAT question type and IN question type

Encoding of the address is kind of insane ("second-level encoding"), it involves encoding into a string
of upper case characteers between A and P by splitting each character and arithmatically adding the
ASCII value of 'A' (65 or 0x41) and then decoding that to an ASCII string, so the letters 'CAKE' becomes 'EDEBELEF'
See the functiom below
"""


def second_level_encode(str):
    out = ""
    for c in str:
        # Get ascii val
        cval = ord(c)

        # split into two nibbles
        cval1 = cval >> 4
        cval2 = cval & 0x0F

        # Add the ASCII value of 'A' to each char
        # And add that to the string
        out += chr(cval1 + 65)
        out += chr(cval2 + 65)

    return out


def extract_names(num_names: int, resp: bytearray) -> list:
    out = []
    idx = 0
    for n in range(num_names):
        name = resp[idx:(idx + NBNS_RESP_NAME_LEN)].decode().strip()
        out.append(name)
        idx += NBNS_RESP_NAME_LEN + 3
    return out


def main():
    trans_id = 0xF00D
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    msg = build_dns_header(trans_id)
    NBNS_PORT = 137
    NBNS_ADDR = sys.argv[1]
    enc_name = second_level_encode("*" + ("\x00" * 15))
    msg.append(len(enc_name))
    msg += enc_name.encode()

    # Termination byte
    msg.append(0x00)

    # Add QUESTION_TYPE == NBSTAT https://tools.ietf.org/html/rfc1002#section-4.2.1.2
    msg += bytes([0x00, 0x21])

    # Add QUESTION_CLASS == IN
    msg += bytes([0x00, 0x01])

    sock.sendto(msg, (NBNS_ADDR, NBNS_PORT))

    sock.settimeout(2)

    try:
        while True:
            resp = sock.recv(10240)
            tid = struct.unpack(">H", resp[0:2])[0]
            if tid == trans_id:
                num_names = resp[NBNS_RESP_BASE_LEN - 1]
                names = extract_names(num_names, resp[NBNS_RESP_BASE_LEN:])
                print(names)
                break

    except socket.timeout:
        print('No response in time')


if __name__ == '__main__':
    main()

"""
6pm
First try: Wireshark shows a malformed packet, but does register it is NBNS
probably from the port

Getting type and class unknown, maybe I forgot the terminating null byte?

Ah, termination null byte goes at the end of the address before the class and type
not at the end of the packet

Now it recognizes the packet type

When I turn on my windows PC, I can see a bunch of NBNS packets flying around (MINIPIG),
mostly to the broadcast address (192.168.0.255)

Lets try to send this packet to the windows PC

And the minipig responds! And with our 0xF00D transaction code!
Now time to unpack the info

Once again, with Wireshark we can easily understand how the bytes translate to each field,
and extracting the hostname from the message should be farily trivial (right?)

Seems like netbios gives us a lot more info than multicast

We'll first try to unpack in the following way:
Looks like the first field we care about (number of names)
is the 56th (0 indexed) item. Since in this situation, the name is always the same
length, seems like we can rely on this to be the case everytime (in our situation)

Starting right after the number of names byte, we'll just read until we get a
non-printable character and use that as the first name, since it doesn't seem
to give us the length of each name, although it does give us the length of the
total data section

We might get the MAC address out of this too

We can also see that netBIOS names are at max 15 characters long
https://docs.microsoft.com/en-us/troubleshoot/windows-server/identity/naming-conventions-for-computer-domain-site-ou#netbios-computer-names
with 3 additional bytes of flags

So, n number of times, lets the get the 15 characters, and skip 3

see extract_names
"""
