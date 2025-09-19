#ifndef MICROSERIAL_OS_EVENT_LOOP_H
#define MICROSERIAL_OS_EVENT_LOOP_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port;

typedef void (*ms_serial_data_callback)(const uint8_t *data, size_t length, void *user_data);
typedef void (*ms_serial_event_callback)(int event_code, const char *message, void *user_data);

typedef struct ms_serial_callbacks {
    ms_serial_data_callback on_data;
    ms_serial_event_callback on_event;
} ms_serial_callbacks_t;

int ms_serial_port_start(struct ms_serial_port *port, ms_serial_callbacks_t callbacks, void *user_data);
int ms_serial_port_stop(struct ms_serial_port *port);
int ms_serial_port_poll(struct ms_serial_port *port);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_OS_EVENT_LOOP_H */
