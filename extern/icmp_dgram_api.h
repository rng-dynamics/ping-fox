#include <netinet/in.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct IcmpData {
  uint8_t *data_buffer;
  uint64_t data_buffer_size;
  size_t n_data_bytes_received;
  uint8_t ttl;
  uint8_t addr_str[INET6_ADDRSTRLEN];
} IcmpData;

/**
 * @brief Receive message from a socket.
 * @param socket The socket file descriptor.
 * @param data Points to an ImcpData structure acting as the buffer to store the
 * incoming message and its metadata.
 * @return On success, recv_from will return the number of bytes received. If
 * no messages are available to be received and the peer has performed an
 * orderly shutdown, recv_from will return 0. Otherwise, a negative number
 * will be returned.
 */
int recv_from(int socket, IcmpData *data);
