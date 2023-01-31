#include "dgram_socket_api.h"

#include <arpa/inet.h>
#include <assert.h>
#include <netinet/in.h>
#include <stdbool.h>
#include <sys/socket.h>

#include <stdio.h> // TODO: remove

RecvData recv_from(int socket, char *buffer, size_t buffer_len) {
  int yes = 1;
  // TODO: check return type
  setsockopt(socket, IPPROTO_IP, IP_RECVTTL, &yes, sizeof(yes));

  int ttl_to_set = 255;
  // TODO: check return type
  setsockopt(socket, IPPROTO_IP, IP_TTL, &ttl_to_set, sizeof(ttl_to_set));

  int ttl_to_read = 0;
  socklen_t size = sizeof(ttl_to_read);
  getsockopt(socket, IPPROTO_IP, IP_TTL, &ttl_to_read, &size);
  printf("(c) ttl read: %d (anything in [1, 255] is good)\n", ttl_to_read);

  const size_t largest_packet_expected = 256;
  uint8_t recv_buffer[largest_packet_expected];
  struct iovec iov[1] = {{recv_buffer, sizeof(recv_buffer)}};

  struct sockaddr_storage src_address;

  uint8_t ctrl_data_buffer[CMSG_SPACE(sizeof(uint8_t))];

  struct msghdr header = {
      .msg_name = &src_address,
      .msg_namelen = sizeof(src_address),
      .msg_iov = iov,
      .msg_iovlen = 1,
      .msg_control = ctrl_data_buffer,
      .msg_controllen = sizeof(ctrl_data_buffer),
  };

  ssize_t bytes_received = recvmsg(socket, &header, MSG_TRUNC);

  if (header.msg_flags & MSG_CTRUNC) {
    assert(false && "control data truncated");
  }

  if (header.msg_flags & MSG_TRUNC) {
    assert(false && "message truncated");
  }

  bool is_ttl_set = false;
  uint8_t ttl = 0;
  struct cmsghdr *cmsg = CMSG_FIRSTHDR(&header);
  for (; cmsg; cmsg = CMSG_NXTHDR(&header, cmsg)) {
    // Socket opetions are not standardized:
    // Linux: IP_TTL
    // BSD: IP_RECVTTL
    printf("(c) in loop %i\n", cmsg->cmsg_type);
    if (cmsg->cmsg_level == IPPROTO_IP &&
        // cmsg->cmsg_type == IP_RECVTTL) {
        cmsg->cmsg_type == IP_TTL) {
      printf("(c) cmsg->cmsg_len == %zu\n", cmsg->cmsg_len);
      // uint8_t* ttl_ptr = (uint8_t *)CMSG_DATA(cmsg);
      uint8_t *ttl_ptr = (uint8_t *)((cmsg)->__cmsg_data);
      ttl = *ttl_ptr;
      is_ttl_set = true;
      printf("(c) reading ttl OK %d\n", ttl);
      // break;
    }
  }
  if (!is_ttl_set) {
    printf("(c) COULD NOT READ ttl\n");
  }

  for (ssize_t idx = 0; idx < bytes_received; ++idx) {
    buffer[idx] = recv_buffer[idx];
  }

  // TODO: cleanup from here down
  char addr_str[INET6_ADDRSTRLEN];
  const char *conversion_result = NULL;
  struct sockaddr *sockaddr_ptr = (struct sockaddr *)header.msg_name;
  if (sockaddr_ptr->sa_family == AF_INET) {
    printf("(c) in conditional\n");
    conversion_result =
        inet_ntop(AF_INET, &(((struct sockaddr_in *)sockaddr_ptr)->sin_addr),
                  addr_str, INET6_ADDRSTRLEN);
  }
  if (sockaddr_ptr->sa_family == AF_INET6) {
    printf("(c) in conditional\n");
    conversion_result =
        inet_ntop(AF_INET6, &(((struct sockaddr_in *)sockaddr_ptr)->sin_addr),
                  addr_str, INET6_ADDRSTRLEN);
  }

  RecvData result = {.bytes_received = bytes_received, .is_ttl_set = is_ttl_set, .ttl = ttl};

  if (conversion_result == NULL) {
    printf("(c) error\n");
  } else {
    printf("(c) addr: %s\n", addr_str);
    assert(INET_ADDRSTRLEN < 40 && "error");
    for (ssize_t idx = 0; idx < INET_ADDRSTRLEN; ++idx) {
      result.addr_str[idx] = addr_str[idx];
    }
  }

  return result;
}
