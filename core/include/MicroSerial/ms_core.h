#ifndef MICROSERIAL_MS_CORE_H
#define MICROSERIAL_MS_CORE_H

/**
 * @file ms_core.h
 * @brief Primary include header for MicroSerial core API.
 */

#ifdef __cplusplus
extern "C" {
#endif

#include "MicroSerial/io/serial.h"
#include "MicroSerial/io/serial_config.h"
#include "MicroSerial/io/serial_discovery.h"
#include "MicroSerial/util/logging.h"
#include "MicroSerial/util/time.h"
#include "MicroSerial/plugins/plugin_abi.h"

#define MICROSERIAL_CORE_API_VERSION_MAJOR 0
#define MICROSERIAL_CORE_API_VERSION_MINOR 1
#define MICROSERIAL_CORE_API_VERSION_PATCH 0

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_MS_CORE_H */
