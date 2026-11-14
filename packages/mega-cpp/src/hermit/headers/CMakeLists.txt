# Emit commands to compile headers, otherwise, clang-tidy gets
# confused when checking headers.
# These are here because using FILE_SET HEADERS makes CMake ignores them
set_source_files_properties(${MEGAHERMIT_HEADERS} PROPERTIES LANGUAGE CXX)
add_library(megahermit_headers OBJECT EXCLUDE_FROM_ALL ${MEGAHERMIT_HEADERS})
target_compile_options(megahermit_headers PRIVATE $<TARGET_PROPERTY:megahermit,COMPILE_OPTIONS>)
target_include_directories(megahermit_headers PRIVATE $<TARGET_PROPERTY:megahermit,INCLUDE_DIRECTORIES>)
target_compile_definitions(megahermit_headers PRIVATE $<TARGET_PROPERTY:megahermit,COMPILE_DEFINITIONS>)
target_compile_options(megahermit_headers PRIVATE -Wno-pragma-once-outside-header)
