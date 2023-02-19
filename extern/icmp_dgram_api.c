#include "icmp_dgram_api.h"

#include <arpa/inet.h>
#include <assert.h>
#include <errno.h> // TODO: remove
#include <stdbool.h>
#include <string.h> // TODO: remove
#include <sys/socket.h>

int recv_from(int socket, IcmpData *data) {
  int yes = 1;
  if (0 != setsockopt(socket, IPPROTO_IP, IP_RECVTTL, &yes, sizeof(yes))) {
    // error setting socket option receive-TTL
    return -1;
  }

  int ttl_to_set = 255;
  if (0 !=
      setsockopt(socket, IPPROTO_IP, IP_TTL, &ttl_to_set, sizeof(ttl_to_set))) {
    // error setting socket option set-TTL
    return -2;
  }

  struct iovec iov[1] = {{data->data_buffer, data->data_buffer_size}};
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

  ssize_t n_bytes_received = recvmsg(socket, &header, MSG_TRUNC);
  if (n_bytes_received < 0) {
    return -3;
  }
  data->n_data_bytes_received = n_bytes_received;
  if (n_bytes_received == 0) {
    return 0;
  }

  if (header.msg_flags & MSG_CTRUNC) {
    // error: control data truncated
    return -4;
  }

  if (header.msg_flags & MSG_TRUNC) {
    // error: message truncated
    return -5;
  }

  bool is_ttl_received = false;
  struct cmsghdr *cmsg = CMSG_FIRSTHDR(&header);
  for (; cmsg; cmsg = CMSG_NXTHDR(&header, cmsg)) {
    // Socket opetions are not standardized:
    // Linux: IP_TTL
    // BSD: IP_RECVTTL
    if (cmsg->cmsg_level == IPPROTO_IP &&
        // cmsg->cmsg_type == IP_RECVTTL) {
        cmsg->cmsg_type == IP_TTL) {
      // uint8_t *ttl_ptr = (uint8_t *)((cmsg)->__cmsg_data);
      uint8_t *ttl_ptr = (uint8_t *)CMSG_DATA(cmsg);
      data->ttl = *ttl_ptr;
      is_ttl_received = true;
      break;
    }
  }
  if (!is_ttl_received) {
    return -6;
  }

  struct sockaddr *sockaddr_ptr = (struct sockaddr *)header.msg_name;
  // TODO: if sockaddr_ptr->sa_family is AF_INET6, then you cannot use sin_addr.
  //   you would have to use sin6_addr instead. See man sockaddr.
  const char *conversion_success =
      inet_ntop(sockaddr_ptr->sa_family,
                &(((struct sockaddr_in *)sockaddr_ptr)->sin_addr),
                (char *)data->addr_str, INET6_ADDRSTRLEN);

  if (!conversion_success) {
    return -7;
  }

  return n_bytes_received;
}
