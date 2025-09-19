#ifndef MICROSERIAL_SERIAL_INTERNAL_H
#define MICROSERIAL_SERIAL_INTERNAL_H

#include "MicroSerial/io/serial_config.h"
#include "MicroSerial/os/event_loop.h"
#include "MicroSerial/io/ring_buffer.h"

#include <pthread.h>
#include <stdatomic.h>

typedef struct ms_ring_buffer ms_ring_buffer_t;

typedef struct ms_serial_port {
    int fd;
    ms_serial_config_t config;
    ms_ring_buffer_t *rx_buffer;
    ms_ring_buffer_t *tx_buffer;
    ms_serial_callbacks_t callbacks;
    void *user_data;
    pthread_t io_thread;
    _Atomic int running;
    int wake_pipe[2];
    int poll_handle;
    pthread_mutex_t tx_mutex;
} ms_serial_port_t;

int ms_posix_configure_port(int fd, const ms_serial_config_t *config);
int ms_posix_apply_flow_control(int fd, ms_serial_flow_control_t flow);

#endif /* MICROSERIAL_SERIAL_INTERNAL_H */
