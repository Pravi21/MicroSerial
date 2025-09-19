#ifndef MICROSERIAL_FUNCTIONS_MS_LOG_MESSAGE_H
#define MICROSERIAL_FUNCTIONS_MS_LOG_MESSAGE_H

#include <stdarg.h>
#include "MicroSerial/functions/ms_log_set_level.h"

#ifdef __cplusplus
extern "C" {
#endif

void ms_log_message(ms_log_level_t level, const char *fmt, ...);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_LOG_MESSAGE_H */
