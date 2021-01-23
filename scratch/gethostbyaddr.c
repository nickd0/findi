#include <stdio.h>
#include <netdb.h>
#include <arpa/inet.h>

int main(int argc, char const *argv[]) {
  if (argc < 2) {
    printf("Please include an address\n");
    return 1;
  }

  const char *ipaddr = argv[1];
  struct in_addr ip;
  inet_aton(ipaddr, &ip);

  struct hostent *ret = gethostbyaddr((const char *)&ip, sizeof ip, AF_INET);
  if (ret != NULL) {
    printf("Hostname: %s\n", ret->h_name);
  } else {
    printf("Couldn't get hostname\n");
  }
  return 0;
}
