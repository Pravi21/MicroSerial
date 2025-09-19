#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_CONFIGURE_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_CONFIGURE_H

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port;
struct ms_serial_config;

int ms_serial_port_configure(struct ms_serial_port *port, const struct ms_serial_config *config);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_CONFIGURE_H */
