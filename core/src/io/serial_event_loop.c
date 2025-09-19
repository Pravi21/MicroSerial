#include "MicroSerial/os/event_loop.h"

#include "MicroSerial/io/ring_buffer.h"
#include "MicroSerial/util/logging.h"

#include "serial_internal.h"

#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#if defined(__linux__)
#include <sys/epoll.h>
#elif defined(__APPLE__)
#include <sys/event.h>
#endif

#define MS_SERIAL_MAX_EVENTS 4
#define MS_SERIAL_IO_CHUNK 4096

static void ms_serial_emit_event(ms_serial_port_t *port, int code, const char *message) {
    if (port->callbacks.on_event) {
        port->callbacks.on_event(code, message, port->user_data);
    }
}

static void dispatch_rx(ms_serial_port_t *port) {
    uint8_t buffer[MS_SERIAL_IO_CHUNK];
    for (;;) {
        ssize_t n = read(port->fd, buffer, sizeof(buffer));
        if (n > 0) {
            ms_ring_buffer_write(port->rx_buffer, buffer, (size_t)n);
            if (port->callbacks.on_data) {
                port->callbacks.on_data(buffer, (size_t)n, port->user_data);
            }
        } else if (n == 0) {
            ms_serial_emit_event(port, 1, "remote closed");
            break;
        } else {
            if (errno == EAGAIN || errno == EWOULDBLOCK) {
                break;
            }
            ms_serial_emit_event(port, -errno, "read error");
            break;
        }
    }
}

static void dispatch_tx(ms_serial_port_t *port) {
    if (!port->tx_buffer) {
        return;
    }
    uint8_t buffer[MS_SERIAL_IO_CHUNK];
    for (;;) {
        size_t available = ms_ring_buffer_read(port->tx_buffer, buffer, sizeof(buffer));
        if (available == 0) {
            break;
        }
        size_t offset = 0;
        while (offset < available) {
            ssize_t written = write(port->fd, buffer + offset, available - offset);
            if (written > 0) {
                offset += (size_t)written;
            } else if (written < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
                ms_ring_buffer_write(port->tx_buffer, buffer + offset, available - offset);
                return;
            } else {
                ms_serial_emit_event(port, -errno, "write error");
                return;
            }
        }
    }
}

static void *ms_serial_io_thread(void *arg) {
    ms_serial_port_t *port = (ms_serial_port_t *)arg;
    while (atomic_load(&port->running)) {
        ms_serial_port_poll((struct ms_serial_port *)port);
    }
    return NULL;
}

#if defined(__linux__)
static int configure_epoll(ms_serial_port_t *port) {
    int epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    if (epoll_fd < 0) {
        return -errno;
    }
    struct epoll_event ev = {0};
    ev.events = EPOLLIN | EPOLLOUT | EPOLLERR | EPOLLHUP;
    ev.data.fd = port->fd;
    if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, port->fd, &ev) < 0) {
        int err = -errno;
        close(epoll_fd);
        return err;
    }
    struct epoll_event wake_ev = {0};
    wake_ev.events = EPOLLIN;
    wake_ev.data.fd = port->wake_pipe[0];
    if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, port->wake_pipe[0], &wake_ev) < 0) {
        int err = -errno;
        close(epoll_fd);
        return err;
    }
    port->poll_handle = epoll_fd;
    return 0;
}
#elif defined(__APPLE__)
static int configure_kqueue(ms_serial_port_t *port) {
    int kq = kqueue();
    if (kq < 0) {
        return -errno;
    }
    struct kevent changes[2];
    EV_SET(&changes[0], port->fd, EVFILT_READ, EV_ADD | EV_ENABLE, 0, 0, NULL);
    EV_SET(&changes[1], port->fd, EVFILT_WRITE, EV_ADD | EV_ENABLE, 0, 0, NULL);
    if (kevent(kq, changes, 2, NULL, 0, NULL) < 0) {
        int err = -errno;
        close(kq);
        return err;
    }
    struct kevent wake_change;
    EV_SET(&wake_change, port->wake_pipe[0], EVFILT_READ, EV_ADD | EV_ENABLE, 0, 0, NULL);
    if (kevent(kq, &wake_change, 1, NULL, 0, NULL) < 0) {
        int err = -errno;
        close(kq);
        return err;
    }
    port->poll_handle = kq;
    return 0;
}
#endif

int ms_serial_port_start(struct ms_serial_port *handle, ms_serial_callbacks_t callbacks, void *user_data) {
    if (!handle) {
        return -EINVAL;
    }
    ms_serial_port_t *port = (ms_serial_port_t *)handle;
    if (!port->rx_buffer || !port->tx_buffer) {
        return -EINVAL;
    }
    if (atomic_exchange(&port->running, 1) == 1) {
        return 0;
    }
    port->callbacks = callbacks;
    port->user_data = user_data;
#if defined(__linux__)
    int rc = configure_epoll(port);
#elif defined(__APPLE__)
    int rc = configure_kqueue(port);
#else
    int rc = -ENOTSUP;
#endif
    if (rc != 0) {
        atomic_store(&port->running, 0);
        return rc;
    }
    int thread_rc = pthread_create(&port->io_thread, NULL, ms_serial_io_thread, port);
    if (thread_rc != 0) {
        atomic_store(&port->running, 0);
        close(port->poll_handle);
        port->poll_handle = -1;
        return -thread_rc;
    }
    return 0;
}

int ms_serial_port_stop(struct ms_serial_port *handle) {
    if (!handle) {
        return -EINVAL;
    }
    ms_serial_port_t *port = (ms_serial_port_t *)handle;
    if (atomic_exchange(&port->running, 0) == 0) {
        return 0;
    }
    if (write(port->wake_pipe[1], "s", 1) < 0) {
        // ignore
    }
    pthread_join(port->io_thread, NULL);
    if (port->poll_handle >= 0) {
        close(port->poll_handle);
        port->poll_handle = -1;
    }
    return 0;
}

int ms_serial_port_poll(struct ms_serial_port *handle) {
    if (!handle) {
        return -EINVAL;
    }
    ms_serial_port_t *port = (ms_serial_port_t *)handle;
#if defined(__linux__)
    struct epoll_event events[MS_SERIAL_MAX_EVENTS];
    int timeout = (int)port->config.read_timeout_ms;
    int n = epoll_wait(port->poll_handle, events, MS_SERIAL_MAX_EVENTS, timeout > 0 ? timeout : -1);
    if (n < 0) {
        if (errno == EINTR) {
            return 0;
        }
        ms_serial_emit_event(port, -errno, "epoll_wait failed");
        return -errno;
    }
    for (int i = 0; i < n; ++i) {
        int fd = events[i].data.fd;
        if (fd == port->wake_pipe[0]) {
            uint8_t buf[16];
            while (read(fd, buf, sizeof(buf)) > 0) {
            }
            continue;
        }
        if (events[i].events & (EPOLLERR | EPOLLHUP)) {
            ms_serial_emit_event(port, -1, "device error");
        }
        if (events[i].events & EPOLLIN) {
            dispatch_rx(port);
        }
        if (events[i].events & EPOLLOUT) {
            dispatch_tx(port);
        }
    }
    return 0;
#elif defined(__APPLE__)
    struct kevent events[MS_SERIAL_MAX_EVENTS];
    struct timespec timeout = {0};
    struct timespec *timeout_ptr = NULL;
    if (port->config.read_timeout_ms > 0) {
        timeout.tv_sec = port->config.read_timeout_ms / 1000;
        timeout.tv_nsec = (port->config.read_timeout_ms % 1000) * 1000000L;
        timeout_ptr = &timeout;
    }
    int n = kevent(port->poll_handle, NULL, 0, events, MS_SERIAL_MAX_EVENTS, timeout_ptr);
    if (n < 0) {
        if (errno == EINTR) {
            return 0;
        }
        ms_serial_emit_event(port, -errno, "kevent failed");
        return -errno;
    }
    for (int i = 0; i < n; ++i) {
        if (events[i].ident == (uintptr_t)port->wake_pipe[0]) {
            uint8_t buf[16];
            while (read(port->wake_pipe[0], buf, sizeof(buf)) > 0) {
            }
            continue;
        }
        if (events[i].filter == EVFILT_READ) {
            dispatch_rx(port);
        }
        if (events[i].filter == EVFILT_WRITE) {
            dispatch_tx(port);
        }
        if (events[i].flags & EV_ERROR) {
            ms_serial_emit_event(port, events[i].data, "device error");
        }
    }
    return 0;
#else
    (void)port;
    return -ENOTSUP;
#endif
}
