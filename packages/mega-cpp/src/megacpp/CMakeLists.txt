
file(GLOB_RECURSE SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/*.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/*.c"
)
add_library(megacpp STATIC ${SOURCES})
target_compile_options(megacpp PRIVATE
    $<$<COMPILE_LANGUAGE:CXX>:${MEGATON_LIB_GENERATED_CXX_FLAGS}>
    $<$<COMPILE_LANGUAGE:C>:${MEGATON_LIB_GENERATED_C_FLAGS}>
)
target_include_directories(megacpp SYSTEM PRIVATE ${DEVKITA64_SYSTEM_INCLUDES})

target_include_directories(megacpp PRIVATE
    "${MEGATON_PUBLIC_INCLUDE}"
    "${MEGATON_PRIVATE_INCLUDE}"
)
