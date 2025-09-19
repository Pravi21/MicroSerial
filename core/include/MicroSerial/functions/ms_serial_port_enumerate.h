#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_ENUMERATE_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_ENUMERATE_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port_info;

int ms_serial_port_enumerate(struct ms_serial_port_info **out_list, size_t *out_count);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_ENUMERATE_H */
