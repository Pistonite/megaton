
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

# Disable C++20 module scanning so CMake doesn't inject -fmodules-ts / -fdeps-format / -fmodule-mapper
set(CMAKE_CXX_SCAN_FOR_MODULES OFF)
# Clears paths CMake considers implicit for each language so they aren't
# silently filtered from compile_commands.json (needed for clangd/clang-tidy).
set(CMAKE_CXX_IMPLICIT_INCLUDE_DIRECTORIES "")
set(CMAKE_C_IMPLICIT_INCLUDE_DIRECTORIES "")
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(DEVKITA64_SYSTEM_INCLUDES
    "${DKP_A64}/${DKP_TRIPLE}/include/c++/${DKP_VERSION}"
    "${DKP_A64}/${DKP_TRIPLE}/include/c++/${DKP_VERSION}/${DKP_TRIPLE}"
    "${DKP_A64}/${DKP_TRIPLE}/include/c++/${DKP_VERSION}/backward"
    "${DKP_A64}/lib/gcc/${DKP_TRIPLE}/${DKP_VERSION}/include"
    "${DKP_A64}/lib/gcc/${DKP_TRIPLE}/${DKP_VERSION}/include-fixed"
    "${DKP_A64}/${DKP_TRIPLE}/include"
    "${DEVKITPRO}/libnx/include"
)
