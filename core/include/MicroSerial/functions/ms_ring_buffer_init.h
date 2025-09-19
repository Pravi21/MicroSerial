#ifndef MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_INIT_H
#define MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_INIT_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ms_ring_buffer ms_ring_buffer_t;

int ms_ring_buffer_init(ms_ring_buffer_t **buffer, size_t capacity);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_INIT_H */
