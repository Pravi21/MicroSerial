#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_POLL_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_POLL_H

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port;

int ms_serial_port_poll(struct ms_serial_port *port);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_POLL_H */
