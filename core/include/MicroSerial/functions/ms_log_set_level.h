#ifndef MICROSERIAL_FUNCTIONS_MS_LOG_SET_LEVEL_H
#define MICROSERIAL_FUNCTIONS_MS_LOG_SET_LEVEL_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum ms_log_level {
    MS_LOG_LEVEL_ERROR = 0,
    MS_LOG_LEVEL_WARN,
    MS_LOG_LEVEL_INFO,
    MS_LOG_LEVEL_DEBUG,
    MS_LOG_LEVEL_TRACE
} ms_log_level_t;

void ms_log_set_level(ms_log_level_t level);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_FUNCTIONS_MS_LOG_SET_LEVEL_H */
