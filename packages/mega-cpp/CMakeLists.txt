cmake_minimum_required(VERSION 4.3)
include(etc/devkita64.cmake)
include(etc/flags.cmake)

project(megaton C CXX ASM)

set(MEGATON_PUBLIC_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/include")
set(MEGATON_PRIVATE_INCLUDE
    "${CMAKE_CURRENT_SOURCE_DIR}/src/megacpp/include"
    # TODO - libraries, add here
)

add_subdirectory(src/rocrt)
add_subdirectory(src/megacpp)
add_subdirectory(src/hermit)
