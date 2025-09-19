#ifndef MICROSERIAL_IO_SERIAL_DISCOVERY_H
#define MICROSERIAL_IO_SERIAL_DISCOVERY_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ms_serial_port_info {
    char path[256];
    char description[256];
} ms_serial_port_info_t;

#include "MicroSerial/functions/ms_serial_port_enumerate.h"
#include "MicroSerial/functions/ms_serial_port_list_free.h"

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_IO_SERIAL_DISCOVERY_H */
