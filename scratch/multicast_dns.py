import socket
import struct
import sys



def build_dns_header(trans_id, flags=0, recurs_flags=0, question_count=1):

    # DNS header:
    # 16-bit identifier
    msg = bytearray([(trans_id >> 8), trans_id & 0xFF])

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

    return msg


if __name__ == '__main__':

    # cmd line option and addr
    opt = sys.argv[1]
    hostaddr = sys.argv[2]

    msg = build_dns_header(0xFEED)

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

    addr_parts = hostaddr.split('.')
    addr_parts.reverse()
    addr_parts += ['in-addr', 'arpa']
    # 6:12pm needs to be in PTR format!
    # 1.2.3.4.in-addr.arpa
    # Oops it has to be reversed in PTR format
    # ptr_addr = hostaddr + '.in-addr.arpa'

    # Build a separate reusable question message
    question = bytearray()

    for p in addr_parts:
        p_bytes = p.encode()
        question.append(len(p_bytes))
        question += p_bytes

    question.append(0)

    """
    Next section is QTYPE, a 2-byte code indicating the type
    of question we're asking
    (https://en.wikipedia.org/wiki/List_of_DNS_record_types)
    We're asking a PTR type question since we want a reverse
    lookup, that value is 12 (0x0C)
    """
    question.extend([0x00, 0x0C])

    """
    Final field is Query class, another 2-byte code.
    Its often 1 for Internet ("IN")
    see http://www.tcpipguide.com/free/t_DNSNameServerDataStorageResourceRecordsandClasses-3.htm
    for shortlist and
    https://www.iana.org/assignments/dns-parameters/dns-parameters.xhtml
    for long list
    """
    question.extend([0x00, 0x01])

    msg += question


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
    # sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    # sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    # sock.bind((MC_ADDR, MC_PORT))


    # 1:41pm this SO answer (https://stackoverflow.com/a/52791404/3121367) builds a simple listening script that worked with the following options:
    # python multicast_recv.py  --join-mcast-groups '224.0.0.251'  --port 5353 --bind-group '224.0.0.251'

    # 5:35pm updating after above worked:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    # Allow reuse of socket (socket level, reuse option, value 1)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    # Bind on mDNS port on all interfaces (empty host string)
    sock.bind((MC_ADDR, MC_PORT))

    # Next we want to tell the socket that we're multicasting, so
    # we'll use the IP_ADD_MEMBERSHIP socket option on level IPPROTO_IP
    # The value of which will be the struct `struct ip_mreqn`
    # See below:
    # https://linux.die.net/man/7/ip
    # The struct is as follows:
    """
    struct ip_mreqn {
        struct in_addr imr_multiaddr; /* IP multicast group
                                        address */
        struct in_addr imr_address;   /* IP address of local
                                        interface */
        int            imr_ifindex;   /* interface index */
    };
    Since we don't care about the interface address, we use INADDR_ANY as the
    value and the OS will choose an interface for us, so all we need to pack is
    `imr_multiaddr` (our multicast host address) and `imr_address` (INADDR_ANY == 0)
    we'll use the socket function `inet_aton` (https://linux.die.net/man/3/inet_addr)
    to convert our string version of the mDNS address to a binary form in network byte order,
    which the ip_mreqn struct expects in the `imr_multiaddr` field.
    """
    mreqn = struct.pack('4sl', socket.inet_aton(MC_ADDR), socket.INADDR_ANY)
    # mreqn = struct.pack('4s4s', socket.inet_aton(MC_ADDR), socket.inet_aton('192.168.0.10'))

    # Now set the sockopt
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreqn)

    # Now it works! We're recieving multicast DNS packets! (show gif of console prints)

    # 5:50pm So can we just send now? lets try to send our multicast message and then receive

    # 5:55 Added:
    # Set the sending sockopts first, then go back to IP_ADD_MEMBERSHIP
    # sock.sendto(msg, (MC_ADDR, MC_PORT))

    # Setup a different socket, is this a bad idea?
    if opt == '-s':
        MULTICAST_TTL = 20
        sock1 = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
        sock1.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, MULTICAST_TTL)
        sock1.sendto(msg, (MC_ADDR, MC_PORT))
        sock1.settimeout(3)
        try:
            while True:
                val = sock1.recv(10240)
                # print(len(val))
                # unpack header big-endian
                resp_header = struct.unpack(">H2B4h", val[0:12])
                if resp_header[0] == 0xFEED:
                    print("Done")
                    # 10:32am 1/14
                    # Now that we now we've got our response, lets get the index of the answer section
                    # The question section is exactly the same as in the query message, so we can
                    # get the answer offset with the length of the header (12) and question length
                    # We know the metadata for the answer section is 12 long with the format laid out
                    # here https://www2.cs.duke.edu/courses/fall16/compsci356/DNS/DNS-primer.pdf
                    # so lets unpack it and read our DNS answer!
                    ans_idx = 12 + len(question)
                    answer_pre = struct.unpack(">3HLH", val[ans_idx:ans_idx + 12])
                    # TODO check some of the meta data?
                    ans_len = answer_pre[-1]
                    ans_idx = ans_idx + 12
                    answer = ""
                    subs = val[ans_idx:ans_idx + ans_len]
                    bt = subs[0]
                    idx = 0
                    while True:
                        answer += subs[idx + 1:(idx + bt + 1)].decode()
                        if subs[(idx + bt + 1)] == 0x00:
                            break
                        else:
                            answer += "."
                            idx = idx + bt + 1
                            bt = subs[idx]

                    # And we're done!
                    print(answer)

                    exit(0)

        except socket.timeout:
            print("No response")
            exit(1)

        # Manually unpack the question and answer section


    # Hmm, seem to get an immediate reponds on the recv socket!
    # Lets check wireshark to see the structured packets

    # Ok nvm its just the packet I sent :(

    # OK, so perhaps I'm not getting a result, because I'm not actually using a proper
    # domain-name PTR record. I was just translating the IP to strings, but it looks
    # like it may need t be in a specific ARPA-type record format (https://simpledns.plus/help/ptr-records))
    # See replacement of code above with timestamp "6:12pm"
    # Now lets check wireshark again

    # After fixing the PTR format, now I get a response with my transaction ID of 0xFEED!
    # Of course this only works for respolving devices that support mDNS resolution
    # Apple devices do (the original RFC is from Apple engineers in 2013 and became the Bonjour service)
    # Unclear if Windows does? https://stackoverflow.com/a/41019456/3121367
    # exit(0)


    # 12:19am After watching a movie, came back to this to see why I wasn't receiving the response
    # to my mDNS query (but was recieving other mDNS packets). Tried sending then recv'ing on
    # sock1 and that just worked?

    # TODO next unpack response, format:
    # Use wireshark packet breakdown to build the unpack string
    # short - 2x bitfield/int - short - short - short - short - queries sect - answers sect
    # "h2b4hp"
    # Actually this may be hard to unpack because certain bytes indicate variable length
    # Check flags for no error and for authoritative?
    # See https://www2.cs.duke.edu/courses/fall16/compsci356/DNS/DNS-primer.pdf for response format
    # Skip the question section and unpack first

    while True:
        print(sock.recv(10240))
