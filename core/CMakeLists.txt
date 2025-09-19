cmake_minimum_required(VERSION 3.18)
project(microserial_core C)

option(MICROSERIAL_ENABLE_ASAN "Enable AddressSanitizer" OFF)
option(MICROSERIAL_ENABLE_UBSAN "Enable UndefinedBehaviorSanitizer" OFF)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

if(CMAKE_C_COMPILER_ID MATCHES "Clang" OR CMAKE_C_COMPILER_ID MATCHES "GNU")
    add_compile_options(-Wall -Wextra -Wpedantic -Werror=vla -fstack-protector-strong)
    add_compile_definitions(_FORTIFY_SOURCE=2)
    if(MICROSERIAL_ENABLE_ASAN)
        add_compile_options(-fsanitize=address)
        link_libraries(-fsanitize=address)
    endif()
    if(MICROSERIAL_ENABLE_UBSAN)
        add_compile_options(-fsanitize=undefined)
        link_libraries(-fsanitize=undefined)
    endif()
endif()

set(CORE_HEADERS
    include/MicroSerial/ms_core.h
    include/MicroSerial/io/serial.h
    include/MicroSerial/io/serial_config.h
    include/MicroSerial/io/serial_discovery.h
    include/MicroSerial/io/ring_buffer.h
    include/MicroSerial/util/logging.h
    include/MicroSerial/util/time.h
    include/MicroSerial/os/event_loop.h
    include/MicroSerial/functions/ms_serial_port_open.h
    include/MicroSerial/functions/ms_serial_port_configure.h
    include/MicroSerial/functions/ms_serial_port_start.h
    include/MicroSerial/functions/ms_serial_port_write.h
    include/MicroSerial/functions/ms_serial_port_close.h
    include/MicroSerial/functions/ms_serial_port_poll.h
    include/MicroSerial/functions/ms_serial_port_enumerate.h
    include/MicroSerial/functions/ms_serial_port_list_free.h
    include/MicroSerial/functions/ms_ring_buffer_init.h
    include/MicroSerial/functions/ms_ring_buffer_free.h
    include/MicroSerial/functions/ms_ring_buffer_write.h
    include/MicroSerial/functions/ms_ring_buffer_read.h
    include/MicroSerial/functions/ms_ring_buffer_size.h
    include/MicroSerial/functions/ms_log_set_level.h
    include/MicroSerial/functions/ms_log_message.h
    include/MicroSerial/plugins/plugin_abi.h
)

set(CORE_SOURCES
    src/io/serial_port.c
    src/io/serial_event_loop.c
    src/util/ring_buffer.c
    src/util/logging.c
    src/util/time.c
    src/os/posix_serial.c
    src/os/port_enumeration.c
)

add_library(microserial_core STATIC ${CORE_SOURCES} ${CORE_HEADERS})
target_include_directories(microserial_core
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
    PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
)

target_link_libraries(microserial_core PUBLIC pthread)

include(CTest)
if(BUILD_TESTING)
    add_subdirectory(tests)
endif()
