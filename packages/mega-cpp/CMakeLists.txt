cmake_minimum_required(VERSION 4.3)
include(etc/devkita64.cmake)
include(etc/flags.cmake)

project(megaton C CXX ASM)
# Clears paths CMake considers implicit for each language so they aren't
# silently filtered from compile_commands.json (needed for clangd/clang-tidy).
# These have to be here, since project() loads compiler include directories
set(CMAKE_CXX_IMPLICIT_INCLUDE_DIRECTORIES "")
set(CMAKE_C_IMPLICIT_INCLUDE_DIRECTORIES "")

file(GLOB_RECURSE MEGATON_PUBLIC_HEADERS CONFIGURE_DEPENDS
  "${CMAKE_CURRENT_SOURCE_DIR}/include/*.h"
)
file(GLOB_RECURSE MEGATON_PRIVATE_HEADERS CONFIGURE_DEPENDS
  "${CMAKE_CURRENT_SOURCE_DIR}/src/megacpp/include/*.h"
  "${CMAKE_CURRENT_SOURCE_DIR}/src/megacpp/include/*.hpp"
)

set(MEGATON_PUBLIC_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/include")
set(MEGATON_PRIVATE_INCLUDE
    "${CMAKE_CURRENT_SOURCE_DIR}/src/megacpp/include"
)

add_subdirectory(src/rocrt)
add_subdirectory(src/megacpp)
add_subdirectory(src/hermit)
