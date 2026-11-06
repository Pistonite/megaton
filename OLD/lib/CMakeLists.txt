cmake_minimum_required(VERSION 4.3)

if(NOT DEFINED ENV{DEVKITPRO})
    message(FATAL_ERROR "DEVKITPRO environment variable is not set")
endif()

set(DEVKITPRO "$ENV{DEVKITPRO}")
set(DKP_A64 "${DEVKITPRO}/devkitA64")
set(DKP_TRIPLE "aarch64-none-elf")
set(DKP_BIN "${DKP_A64}/bin")

# Detect devkitA64 GCC version from the include directory
file(GLOB _DKP_VER_DIRS "${DKP_A64}/${DKP_TRIPLE}/include/c++/*")
if(NOT _DKP_VER_DIRS)
    message(FATAL_ERROR "Could not detect GCC version: no directories found under ${DKP_A64}/${DKP_TRIPLE}/include/c++/")
endif()
list(GET _DKP_VER_DIRS 0 _DKP_VER_DIR)
get_filename_component(DKP_VERSION "${_DKP_VER_DIR}" NAME)
message(STATUS "Detected devkitA64 GCC version: ${DKP_VERSION}")

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR aarch64)
set(CMAKE_C_COMPILER "${DKP_BIN}/${DKP_TRIPLE}-gcc")
set(CMAKE_CXX_COMPILER "${DKP_BIN}/${DKP_TRIPLE}-g++")
set(CMAKE_ASM_COMPILER "${DKP_BIN}/${DKP_TRIPLE}-gcc")
set(CMAKE_AR "${DKP_BIN}/${DKP_TRIPLE}-ar")
set(CMAKE_RANLIB "${DKP_BIN}/${DKP_TRIPLE}-ranlib")
# Use a static library for compiler detection to avoid linker failures on bare-metal
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

project(megaton-lib C CXX ASM)

# Disable C++20 module scanning so CMake doesn't inject -fmodules-ts / -fdeps-format / -fmodule-mapper
set(CMAKE_CXX_SCAN_FOR_MODULES OFF)

# Clears paths CMake considers implicit for each language so they aren't
# silently filtered from compile_commands.json (needed for clangd/clang-tidy).
set(CMAKE_CXX_IMPLICIT_INCLUDE_DIRECTORIES "")
set(CMAKE_C_IMPLICIT_INCLUDE_DIRECTORIES "")

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

file(GLOB_RECURSE LIB_SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/*.c"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/*.s"
)

add_library(megaton STATIC ${LIB_SOURCES})

set_target_properties(megaton PROPERTIES
    CXX_STANDARD 20
    CXX_STANDARD_REQUIRED ON
    CXX_EXTENSIONS OFF
)

# devkitpro system includes, mirroring the -isystem flags added by the CLI
target_include_directories(megaton SYSTEM PRIVATE
    "${DKP_A64}/${DKP_TRIPLE}/include/c++/${DKP_VERSION}"
    "${DKP_A64}/${DKP_TRIPLE}/include/c++/${DKP_VERSION}/${DKP_TRIPLE}"
    "${DKP_A64}/${DKP_TRIPLE}/include/c++/${DKP_VERSION}/backward"
    "${DKP_A64}/lib/gcc/${DKP_TRIPLE}/${DKP_VERSION}/include"
    "${DKP_A64}/lib/gcc/${DKP_TRIPLE}/${DKP_VERSION}/include-fixed"
    "${DKP_A64}/${DKP_TRIPLE}/include"
    "${DEVKITPRO}/libnx/include"
)

target_include_directories(megaton PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/../nnheaders/include"
)

# Need to be kept sync (mostly) with packages/cli/src/cmds/cmd_build/config/build_flag.rs:
target_compile_options(megaton PRIVATE
    # DEFAULT_COMMON
    -march=armv8-a+crc+crypto
    -mtune=cortex-a57
    # -mtp=soft - disabled since clangd/clang-tidy doesn't recognize it
    -fPIC
    -fvisibility=hidden
    -g
    # DEFAULT_C (applied to C, C++, and ASM)
    -Wall
    -Werror
    -ffunction-sections
    -fdata-sections
    -O2
    # DEFAULT_CPP (C++ only)
    $<$<COMPILE_LANGUAGE:CXX>:-std=c++20>
    $<$<COMPILE_LANGUAGE:CXX>:-fno-rtti>
    $<$<COMPILE_LANGUAGE:CXX>:-fno-exceptions>
    $<$<COMPILE_LANGUAGE:CXX>:-fno-asynchronous-unwind-tables>
    $<$<COMPILE_LANGUAGE:CXX>:-fno-unwind-tables>
)

target_compile_definitions(megaton PRIVATE
    MEGATON_LIB
    # MEGART (runtime ones - placeholder)
    MEGART_NX_MODULE_NAME="test"
    MEGART_NX_MODULE_NAME_LEN=4
    MEGART_TITLE_ID=0
    MEGART_TITLE_ID_HEX="0x0"
    # NNSDK defines - add here, e.g.:
    NNSDK
    SWITCH
    __SWITCH__
    # NN_SDK_MAJOR=4
    # NN_SDK_MINOR=4
    # NN_SDK_PATCH=0
    # NN_WARE_MAJOR=4
    # NN_WARE_MINOR=4
    # NN_WARE_PATCH=0
)
