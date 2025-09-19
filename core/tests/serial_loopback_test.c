#define _POSIX_C_SOURCE 200809L

#include "MicroSerial/ms_core.h"

#include <fcntl.h>
#include <pthread.h>
#include <pty.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/select.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/types.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    size_t received;
    uint8_t buffer[1024];
} callback_ctx_t;

static void on_data(const uint8_t *data, size_t length, void *user_data) {
    callback_ctx_t *ctx = (callback_ctx_t *)user_data;
    pthread_mutex_lock(&ctx->mutex);
    size_t copy = length < sizeof(ctx->buffer) ? length : sizeof(ctx->buffer);
    memcpy(ctx->buffer, data, copy);
    ctx->received = copy;
    pthread_cond_signal(&ctx->cond);
    pthread_mutex_unlock(&ctx->mutex);
}

static void on_event(int code, const char *message, void *user_data) {
    (void)code;
    (void)message;
    (void)user_data;
}

static int wait_for_data(callback_ctx_t *ctx) {
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ts.tv_sec += 2;
    pthread_mutex_lock(&ctx->mutex);
    int rc = 0;
    while (ctx->received == 0 && rc == 0) {
        rc = pthread_cond_timedwait(&ctx->cond, &ctx->mutex, &ts);
    }
    pthread_mutex_unlock(&ctx->mutex);
    return rc;
}

int main(void) {
    int master_fd = -1;
    int slave_fd = -1;
    char slave_name[128];
    if (openpty(&master_fd, &slave_fd, slave_name, NULL, NULL) < 0) {
        perror("openpty");
        return EXIT_FAILURE;
    }
    close(slave_fd);

    struct ms_serial_port *port = NULL;
    if (ms_serial_port_open(slave_name, &port) != 0) {
        fprintf(stderr, "failed to open serial port\n");
        return EXIT_FAILURE;
    }

    ms_serial_config_t config = {
        .baud_rate = 115200,
        .data_bits = 8,
        .stop_bits = 1,
        .parity = MS_SERIAL_PARITY_NONE,
        .flow_control = MS_SERIAL_FLOW_NONE,
        .rx_buffer_size = 8192,
        .tx_buffer_size = 8192,
        .read_timeout_ms = 100,
        .write_timeout_ms = 100,
    };

    if (ms_serial_port_configure(port, &config) != 0) {
        fprintf(stderr, "failed to configure\n");
        return EXIT_FAILURE;
    }

    callback_ctx_t ctx;
    pthread_mutex_init(&ctx.mutex, NULL);
    pthread_cond_init(&ctx.cond, NULL);
    ctx.received = 0;

    ms_serial_callbacks_t callbacks = {
        .on_data = on_data,
        .on_event = on_event,
    };

    if (ms_serial_port_start(port, callbacks, &ctx) != 0) {
        fprintf(stderr, "failed to start io\n");
        return EXIT_FAILURE;
    }

    const char inbound[] = "hello core";
    if (write(master_fd, inbound, sizeof(inbound)) < 0) {
        perror("write inbound");
        return EXIT_FAILURE;
    }

    if (wait_for_data(&ctx) != 0) {
        fprintf(stderr, "timeout waiting for inbound\n");
        return EXIT_FAILURE;
    }

    if (ctx.received != sizeof(inbound)) {
        fprintf(stderr, "unexpected inbound length\n");
        return EXIT_FAILURE;
    }

    const char outbound[] = "hello device";
    ssize_t wrote = ms_serial_port_write(port, (const uint8_t *)outbound, sizeof(outbound));
    if (wrote <= 0) {
        fprintf(stderr, "failed to write outbound\n");
        return EXIT_FAILURE;
    }

    char verify[64] = {0};
    ssize_t read_bytes = read(master_fd, verify, sizeof(verify));
    if (read_bytes <= 0) {
        fprintf(stderr, "failed to read outbound data\n");
        return EXIT_FAILURE;
    }

    if (memcmp(verify, outbound, sizeof(outbound)) != 0) {
        fprintf(stderr, "outbound mismatch\n");
        return EXIT_FAILURE;
    }

    ms_serial_port_stop(port);
    ms_serial_port_close(port);

    close(master_fd);
    pthread_mutex_destroy(&ctx.mutex);
    pthread_cond_destroy(&ctx.cond);

    return EXIT_SUCCESS;
}
