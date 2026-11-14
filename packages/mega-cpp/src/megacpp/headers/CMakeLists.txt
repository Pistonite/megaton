# Emit commands to compile headers, otherwise, clang-tidy gets
# confused when checking headers.
# These are here because using FILE_SET HEADERS makes CMake ignores them
set_source_files_properties(${MEGACPP_PRIVATE_HEADERS} PROPERTIES LANGUAGE CXX)
add_library(megacpp_headers OBJECT EXCLUDE_FROM_ALL ${MEGACPP_PRIVATE_HEADERS})
target_compile_options(megacpp_headers PRIVATE $<TARGET_PROPERTY:megacpp,COMPILE_OPTIONS>)
target_include_directories(megacpp_headers PRIVATE $<TARGET_PROPERTY:megacpp,INCLUDE_DIRECTORIES>)
target_compile_definitions(megacpp_headers PRIVATE $<TARGET_PROPERTY:megacpp,COMPILE_DEFINITIONS>)
target_compile_options(megacpp_headers PRIVATE -Wno-pragma-once-outside-header)
