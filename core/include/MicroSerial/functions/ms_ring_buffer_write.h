#ifndef MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_WRITE_H
#define MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_WRITE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ms_ring_buffer ms_ring_buffer_t;

size_t ms_ring_buffer_write(ms_ring_buffer_t *buffer, const uint8_t *data, size_t length);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_WRITE_H */
