#define _POSIX_C_SOURCE 200809L

#include "MicroSerial/util/time.h"

#include <time.h>

uint64_t ms_time_monotonic_ns(void) {
    struct timespec ts = {0};
#if defined(CLOCK_MONOTONIC_RAW)
    clock_gettime(CLOCK_MONOTONIC_RAW, &ts);
#else
    clock_gettime(CLOCK_MONOTONIC, &ts);
#endif
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}
