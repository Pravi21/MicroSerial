#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_WRITE_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_WRITE_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port;

ssize_t ms_serial_port_write(struct ms_serial_port *port, const uint8_t *data, size_t length);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_WRITE_H */
