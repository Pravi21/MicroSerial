#include "MicroSerial/io/ring_buffer.h"

#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>

struct ms_ring_buffer {
    uint8_t *data;
    size_t capacity;
    _Atomic size_t head;
    _Atomic size_t tail;
};

typedef struct ms_ring_buffer ms_ring_buffer_t;

static size_t next_power_of_two(size_t value) {
    if (value < 2) {
        return 2;
    }
    value--;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
#if SIZE_MAX > UINT32_MAX
    value |= value >> 32;
#endif
    value++;
    return value;
}

int ms_ring_buffer_init(ms_ring_buffer_t **buffer, size_t capacity) {
    if (!buffer || capacity == 0) {
        return -1;
    }
    ms_ring_buffer_t *rb = calloc(1, sizeof(*rb));
    if (!rb) {
        return -2;
    }
    rb->capacity = next_power_of_two(capacity);
    rb->data = calloc(rb->capacity, sizeof(uint8_t));
    if (!rb->data) {
        free(rb);
        return -3;
    }
    atomic_store(&rb->head, 0);
    atomic_store(&rb->tail, 0);
    *buffer = rb;
    return 0;
}

void ms_ring_buffer_free(ms_ring_buffer_t *buffer) {
    if (!buffer) {
        return;
    }
    free(buffer->data);
    free(buffer);
}

static size_t ring_distance(size_t head, size_t tail, size_t capacity) {
    return (head - tail) & (capacity - 1);
}

size_t ms_ring_buffer_size(const ms_ring_buffer_t *buffer) {
    if (!buffer) {
        return 0;
    }
    size_t head = atomic_load(&buffer->head);
    size_t tail = atomic_load(&buffer->tail);
    return ring_distance(head, tail, buffer->capacity);
}

size_t ms_ring_buffer_capacity(const ms_ring_buffer_t *buffer) {
    if (!buffer) {
        return 0;
    }
    return buffer->capacity;
}

size_t ms_ring_buffer_write(ms_ring_buffer_t *buffer, const uint8_t *data, size_t length) {
    if (!buffer || !data || length == 0) {
        return 0;
    }
    size_t head = atomic_load_explicit(&buffer->head, memory_order_relaxed);
    size_t tail = atomic_load_explicit(&buffer->tail, memory_order_acquire);
    size_t available = buffer->capacity - ring_distance(head, tail, buffer->capacity) - 1;
    size_t to_write = length < available ? length : available;
    for (size_t i = 0; i < to_write; ++i) {
        buffer->data[head] = data[i];
        head = (head + 1) & (buffer->capacity - 1);
    }
    atomic_store_explicit(&buffer->head, head, memory_order_release);
    return to_write;
}

size_t ms_ring_buffer_read(ms_ring_buffer_t *buffer, uint8_t *data, size_t length) {
    if (!buffer || !data || length == 0) {
        return 0;
    }
    size_t head = atomic_load_explicit(&buffer->head, memory_order_acquire);
    size_t tail = atomic_load_explicit(&buffer->tail, memory_order_relaxed);
    size_t available = ring_distance(head, tail, buffer->capacity);
    size_t to_read = length < available ? length : available;
    for (size_t i = 0; i < to_read; ++i) {
        data[i] = buffer->data[tail];
        tail = (tail + 1) & (buffer->capacity - 1);
    }
    atomic_store_explicit(&buffer->tail, tail, memory_order_release);
    return to_read;
}
