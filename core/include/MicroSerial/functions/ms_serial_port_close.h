#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_CLOSE_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_CLOSE_H

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port;

void ms_serial_port_close(struct ms_serial_port *port);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_CLOSE_H */
