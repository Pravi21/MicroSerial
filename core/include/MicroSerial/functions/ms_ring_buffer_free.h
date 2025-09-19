#ifndef MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_FREE_H
#define MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_FREE_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ms_ring_buffer ms_ring_buffer_t;

void ms_ring_buffer_free(ms_ring_buffer_t *buffer);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_RING_BUFFER_FREE_H */
