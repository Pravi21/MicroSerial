#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_OPEN_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_OPEN_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port;
struct ms_serial_config;

int ms_serial_port_open(const char *path, struct ms_serial_port **out_port);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_OPEN_H */
