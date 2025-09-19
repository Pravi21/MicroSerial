#define _DEFAULT_SOURCE

#include "io/serial_internal.h"

#include "MicroSerial/util/logging.h"

#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

static speed_t baud_to_speed(uint32_t baud) {
    switch (baud) {
        case 9600:
            return B9600;
        case 19200:
            return B19200;
        case 38400:
            return B38400;
        case 57600:
            return B57600;
        case 115200:
            return B115200;
        case 230400:
            return B230400;
        case 460800:
            return B460800;
        case 921600:
            return B921600;
        default:
            return B115200;
    }
}

static int apply_baud_rate(int fd, speed_t speed) {
    struct termios tio;
    if (tcgetattr(fd, &tio) < 0) {
        return -errno;
    }
    cfsetispeed(&tio, speed);
    cfsetospeed(&tio, speed);
    if (tcsetattr(fd, TCSANOW, &tio) < 0) {
        return -errno;
    }
    return 0;
}

int ms_posix_apply_flow_control(int fd, ms_serial_flow_control_t flow) {
    struct termios tio;
    if (tcgetattr(fd, &tio) < 0) {
        return -errno;
    }
    tio.c_iflag &= ~(IXON | IXOFF | IXANY);
#ifdef CRTSCTS
    tio.c_cflag &= ~CRTSCTS;
#endif
    switch (flow) {
        case MS_SERIAL_FLOW_RTS_CTS:
#ifdef CRTSCTS
            tio.c_cflag |= CRTSCTS;
#else
            ms_log_message(MS_LOG_LEVEL_WARN, "RTS/CTS not supported on this platform");
#endif
            break;
        case MS_SERIAL_FLOW_XON_XOFF:
            tio.c_iflag |= IXON | IXOFF;
            break;
        case MS_SERIAL_FLOW_NONE:
        default:
            break;
    }
    if (tcsetattr(fd, TCSANOW, &tio) < 0) {
        return -errno;
    }
    return 0;
}

int ms_posix_configure_port(int fd, const ms_serial_config_t *config) {
    if (!config) {
        return -EINVAL;
    }
    struct termios tio;
    if (tcgetattr(fd, &tio) < 0) {
        return -errno;
    }

    cfmakeraw(&tio);

    tio.c_cflag &= ~CSIZE;
    switch (config->data_bits) {
        case 5:
            tio.c_cflag |= CS5;
            break;
        case 6:
            tio.c_cflag |= CS6;
            break;
        case 7:
            tio.c_cflag |= CS7;
            break;
        case 8:
        default:
            tio.c_cflag |= CS8;
            break;
    }

    if (config->stop_bits == 2) {
        tio.c_cflag |= CSTOPB;
    } else {
        tio.c_cflag &= ~CSTOPB;
    }

    tio.c_cflag &= ~(PARENB | PARODD);
    if (config->parity == MS_SERIAL_PARITY_EVEN) {
        tio.c_cflag |= PARENB;
        tio.c_cflag &= ~PARODD;
    } else if (config->parity == MS_SERIAL_PARITY_ODD) {
        tio.c_cflag |= PARENB;
        tio.c_cflag |= PARODD;
    }

    tio.c_cc[VMIN] = 0;
    tio.c_cc[VTIME] = (config->read_timeout_ms + 99) / 100;

    if (tcsetattr(fd, TCSANOW, &tio) < 0) {
        return -errno;
    }

    int rc = apply_baud_rate(fd, baud_to_speed(config->baud_rate));
    if (rc != 0) {
        return rc;
    }
    rc = ms_posix_apply_flow_control(fd, config->flow_control);
    if (rc != 0) {
        return rc;
    }

    tcflush(fd, TCIOFLUSH);
    return 0;
}
