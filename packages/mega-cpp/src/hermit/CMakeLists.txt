# # megahermit
#
# This is the implementation of support APIs for the hermit ABI,
# used for supporting the aarch64-unknown-hermit Rust target.
# These files are only linked if Rust support is enabled, which is why
# this is a separate CMake target from the main megacpp library.

file(GLOB_RECURSE MEGAHERMIT_SOURCES CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/*.cpp"
)
file(GLOB_RECURSE MEGAHERMIT_HEADERS CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/*.h"
)

# Compile as objects, since hermit is pre-linked into the Cargo-produced
# static lib, to localize symbols from Rust and hermit ABI to avoid conflicts.
# Packing a staticlib here just means we have to unpack it later
add_library(megahermit OBJECT ${MEGAHERMIT_SOURCES})
target_compile_options(megahermit PRIVATE ${MEGATON_LIB_GENERATED_CXX_FLAGS})
target_include_directories(megahermit PRIVATE
    "${MEGATON_PUBLIC_INCLUDE}"
    "${MEGATON_PRIVATE_INCLUDE}"
)

# Header verfication: make sure they compile by themselves
target_sources(megahermit PRIVATE
  FILE_SET HEADERS
  BASE_DIRS ./
  FILES ${MEGAHERMIT_HEADERS}
)
set_target_properties(megahermit PROPERTIES VERIFY_PRIVATE_HEADER_SETS ON)
add_subdirectory(headers)
