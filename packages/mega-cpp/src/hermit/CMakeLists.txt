
file(GLOB_RECURSE SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/*.cpp"
)
add_library(megahermit OBJECT ${SOURCES})
target_compile_options(megahermit PRIVATE ${MEGATON_LIB_GENERATED_CXX_FLAGS})
target_include_directories(megahermit SYSTEM PRIVATE ${DEVKITA64_SYSTEM_INCLUDES})

target_include_directories(megahermit PRIVATE
    "${MEGATON_PUBLIC_INCLUDE}"
    "${MEGATON_PRIVATE_INCLUDE}"
)
