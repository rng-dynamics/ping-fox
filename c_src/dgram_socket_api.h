#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct RecvData {
    uint64_t bytes_received;
    bool is_ttl_set;
    uint8_t ttl;
    uint8_t addr_str[40];
} RecvData;

RecvData recv_from(int socket, char* buffer, size_t buffer_len);
