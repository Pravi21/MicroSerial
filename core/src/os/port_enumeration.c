#include "MicroSerial/io/serial_discovery.h"

#include <errno.h>
#include <glob.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static bool path_exists(const char *path) {
    struct stat st;
    return stat(path, &st) == 0;
}

static bool contains_path(ms_serial_port_info_t *list, size_t count, const char *path) {
    for (size_t i = 0; i < count; ++i) {
        if (strncmp(list[i].path, path, sizeof(list[i].path)) == 0) {
            return true;
        }
    }
    return false;
}

static int append_port(ms_serial_port_info_t **list, size_t *count, size_t *capacity, const char *path) {
    if (!path_exists(path)) {
        return 0;
    }
    if (contains_path(*list, *count, path)) {
        return 0;
    }
    if (*count >= *capacity) {
        size_t new_capacity = *capacity == 0 ? 8 : (*capacity * 2);
        ms_serial_port_info_t *resized = realloc(*list, new_capacity * sizeof(ms_serial_port_info_t));
        if (!resized) {
            return -ENOMEM;
        }
        *list = resized;
        *capacity = new_capacity;
    }
    ms_serial_port_info_t *info = &(*list)[*count];
    memset(info, 0, sizeof(*info));
    snprintf(info->path, sizeof(info->path), "%s", path);
    snprintf(info->description, sizeof(info->description), "Serial device %s", path);
    (*count)++;
    return 0;
}

int ms_serial_port_enumerate(ms_serial_port_info_t **out_list, size_t *out_count) {
    if (!out_list || !out_count) {
        return -EINVAL;
    }
    *out_list = NULL;
    *out_count = 0;

#if defined(__APPLE__)
    const char *patterns[] = {"/dev/tty.*", "/dev/cu.*"};
#else
    const char *patterns[] = {"/dev/ttyS*", "/dev/ttyUSB*", "/dev/ttyACM*", "/dev/ttyAMA*", "/dev/ttyPS*", "/dev/tty.*"};
#endif

    ms_serial_port_info_t *list = NULL;
    size_t count = 0;
    size_t capacity = 0;

    for (size_t i = 0; i < sizeof(patterns) / sizeof(patterns[0]); ++i) {
        glob_t results;
        memset(&results, 0, sizeof(results));
        if (glob(patterns[i], GLOB_NOCHECK, NULL, &results) != 0) {
            globfree(&results);
            continue;
        }
        for (size_t j = 0; j < results.gl_pathc; ++j) {
            const char *path = results.gl_pathv[j];
            if (!path || strstr(path, "*") != NULL) {
                continue;
            }
            if (append_port(&list, &count, &capacity, path) != 0) {
                globfree(&results);
                free(list);
                return -ENOMEM;
            }
        }
        globfree(&results);
    }

    *out_list = list;
    *out_count = count;
    return 0;
}

void ms_serial_port_list_free(ms_serial_port_info_t *list, size_t count) {
    (void)count;
    free(list);
}
