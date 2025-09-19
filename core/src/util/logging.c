#include "MicroSerial/util/logging.h"

#include <stdio.h>
#include <stdarg.h>
#include <stdatomic.h>

static _Atomic ms_log_level_t g_log_level = MS_LOG_LEVEL_INFO;

void ms_log_set_level(ms_log_level_t level) {
    atomic_store(&g_log_level, level);
}

void ms_log_message(ms_log_level_t level, const char *fmt, ...) {
    if (level > atomic_load(&g_log_level)) {
        return;
    }
    const char *prefix = "INFO";
    switch (level) {
        case MS_LOG_LEVEL_ERROR:
            prefix = "ERROR";
            break;
        case MS_LOG_LEVEL_WARN:
            prefix = "WARN";
            break;
        case MS_LOG_LEVEL_INFO:
            prefix = "INFO";
            break;
        case MS_LOG_LEVEL_DEBUG:
            prefix = "DEBUG";
            break;
        case MS_LOG_LEVEL_TRACE:
            prefix = "TRACE";
            break;
    }
    va_list args;
    va_start(args, fmt);
    fprintf(stderr, "[MicroSerial][%s] ", prefix);
    vfprintf(stderr, fmt, args);
    fprintf(stderr, "\n");
    va_end(args);
}
