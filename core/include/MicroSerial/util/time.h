#ifndef MICROSERIAL_UTIL_TIME_H
#define MICROSERIAL_UTIL_TIME_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint64_t ms_time_monotonic_ns(void);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_UTIL_TIME_H */
