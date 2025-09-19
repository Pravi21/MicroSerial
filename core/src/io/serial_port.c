#include "MicroSerial/io/serial.h"

#include "MicroSerial/io/ring_buffer.h"

#include "serial_internal.h"

#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

int ms_serial_port_open(const char *path, struct ms_serial_port **out_port) {
    if (!path || !out_port) {
        return -EINVAL;
    }

    int fd = open(path, O_RDWR | O_NOCTTY | O_NONBLOCK);
    if (fd < 0) {
        return -errno;
    }

    ms_serial_port_t *port = calloc(1, sizeof(*port));
    if (!port) {
        close(fd);
        return -ENOMEM;
    }

    port->fd = fd;
    port->rx_buffer = NULL;
    port->tx_buffer = NULL;
    port->callbacks.on_data = NULL;
    port->callbacks.on_event = NULL;
    port->user_data = NULL;
    atomic_store(&port->running, 0);
    port->poll_handle = -1;
    if (pipe(port->wake_pipe) < 0) {
        close(fd);
        free(port);
        return -errno;
    }
    fcntl(port->wake_pipe[0], F_SETFL, O_NONBLOCK);
    fcntl(port->wake_pipe[1], F_SETFL, O_NONBLOCK);
    pthread_mutex_init(&port->tx_mutex, NULL);

    *out_port = (struct ms_serial_port *)port;
    return 0;
}

int ms_serial_port_configure(struct ms_serial_port *handle, const struct ms_serial_config *config) {
    if (!handle || !config) {
        return -EINVAL;
    }
    ms_serial_port_t *port = (ms_serial_port_t *)handle;
    int rc = ms_posix_configure_port(port->fd, config);
    if (rc != 0) {
        return rc;
    }
    if (port->rx_buffer) {
        ms_ring_buffer_free(port->rx_buffer);
        port->rx_buffer = NULL;
    }
    if (port->tx_buffer) {
        ms_ring_buffer_free(port->tx_buffer);
        port->tx_buffer = NULL;
    }
    if (ms_ring_buffer_init(&port->rx_buffer, config->rx_buffer_size) != 0) {
        return -ENOMEM;
    }
    if (ms_ring_buffer_init(&port->tx_buffer, config->tx_buffer_size) != 0) {
        ms_ring_buffer_free(port->rx_buffer);
        port->rx_buffer = NULL;
        return -ENOMEM;
    }
    port->config = *config;
    return 0;
}

ssize_t ms_serial_port_write(struct ms_serial_port *handle, const uint8_t *data, size_t length) {
    if (!handle || !data || length == 0) {
        return -EINVAL;
    }
    ms_serial_port_t *port = (ms_serial_port_t *)handle;
    if (!port->tx_buffer) {
        return -EPIPE;
    }
    pthread_mutex_lock(&port->tx_mutex);
    size_t written = ms_ring_buffer_write(port->tx_buffer, data, length);
    pthread_mutex_unlock(&port->tx_mutex);
    if (written == 0) {
        return 0;
    }
    if (write(port->wake_pipe[1], "w", 1) < 0) {
        // Ignored; pipe is non-blocking
    }
    return (ssize_t)written;
}

void ms_serial_port_close(struct ms_serial_port *handle) {
    if (!handle) {
        return;
    }
    ms_serial_port_t *port = (ms_serial_port_t *)handle;
    ms_serial_port_stop(handle);
    if (port->fd >= 0) {
        close(port->fd);
        port->fd = -1;
    }
    if (port->wake_pipe[0] >= 0) {
        close(port->wake_pipe[0]);
        port->wake_pipe[0] = -1;
    }
    if (port->wake_pipe[1] >= 0) {
        close(port->wake_pipe[1]);
        port->wake_pipe[1] = -1;
    }
    if (port->rx_buffer) {
        ms_ring_buffer_free(port->rx_buffer);
    }
    if (port->tx_buffer) {
        ms_ring_buffer_free(port->tx_buffer);
    }
    pthread_mutex_destroy(&port->tx_mutex);
    free(port);
}
