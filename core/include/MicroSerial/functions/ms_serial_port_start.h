#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_START_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_START_H

#include "MicroSerial/os/event_loop.h"

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port;

int ms_serial_port_start(struct ms_serial_port *port, ms_serial_callbacks_t callbacks, void *user_data);
int ms_serial_port_stop(struct ms_serial_port *port);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_START_H */
