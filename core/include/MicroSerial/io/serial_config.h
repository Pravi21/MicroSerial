#ifndef MICROSERIAL_IO_SERIAL_CONFIG_H
#define MICROSERIAL_IO_SERIAL_CONFIG_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Supported parity modes.
 */
typedef enum ms_serial_parity {
    MS_SERIAL_PARITY_NONE = 0,
    MS_SERIAL_PARITY_EVEN,
    MS_SERIAL_PARITY_ODD
} ms_serial_parity_t;

/**
 * @brief Flow control configuration.
 */
typedef enum ms_serial_flow_control {
    MS_SERIAL_FLOW_NONE = 0,
    MS_SERIAL_FLOW_RTS_CTS,
    MS_SERIAL_FLOW_XON_XOFF
} ms_serial_flow_control_t;

/**
 * @brief Serial port configuration descriptor.
 */
typedef struct ms_serial_config {
    uint32_t baud_rate;
    uint8_t data_bits;
    uint8_t stop_bits;
    ms_serial_parity_t parity;
    ms_serial_flow_control_t flow_control;
    uint32_t rx_buffer_size;
    uint32_t tx_buffer_size;
    uint32_t read_timeout_ms;
    uint32_t write_timeout_ms;
} ms_serial_config_t;

#ifdef __cplusplus
}
#endif

#endif /* MICROSERIAL_IO_SERIAL_CONFIG_H */
