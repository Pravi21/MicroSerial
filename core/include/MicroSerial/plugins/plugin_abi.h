#ifndef MICROSERIAL_PLUGINS_PLUGIN_ABI_H
#define MICROSERIAL_PLUGINS_PLUGIN_ABI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ms_plugin_context {
    uint32_t abi_version;
    void (*log)(ms_log_level_t level, const char *message);
} ms_plugin_context_t;

typedef struct ms_plugin_descriptor {
    const char *identifier;
    const char *name;
    const char *version;
    int (*initialize)(const ms_plugin_context_t *context);
    void (*shutdown)(void);
    size_t (*decode)(const uint8_t *input, size_t input_len, uint8_t *output, size_t output_len);
} ms_plugin_descriptor_t;

#define MS_PLUGIN_ABI_VERSION 1

typedef const ms_plugin_descriptor_t *(*ms_plugin_query_fn)(void);

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_PLUGINS_PLUGIN_ABI_H */
