#ifndef MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_LIST_FREE_H
#define MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_LIST_FREE_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

struct ms_serial_port_info;

void ms_serial_port_list_free(struct ms_serial_port_info *list, size_t count);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_SERIAL_PORT_LIST_FREE_H */
