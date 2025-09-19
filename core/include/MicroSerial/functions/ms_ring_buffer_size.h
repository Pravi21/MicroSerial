#ifndef MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_SIZE_H
#define MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_SIZE_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ms_ring_buffer ms_ring_buffer_t;

size_t ms_ring_buffer_size(const ms_ring_buffer_t *buffer);
size_t ms_ring_buffer_capacity(const ms_ring_buffer_t *buffer);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_SIZE_H */
