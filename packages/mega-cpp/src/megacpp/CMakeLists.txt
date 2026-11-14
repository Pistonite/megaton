# # megacpp
#
# The main megaton cpp library

file(GLOB_RECURSE SOURCES CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/*.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/*.c"
)
file(GLOB_RECURSE MEGATON_PUBLIC_HEADERS CONFIGURE_DEPENDS
    "${CMAKE_SOURCE_DIR}/include/megaton/*.h"
)
file(GLOB_RECURSE MEGACPP_PRIVATE_HEADERS CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/include/megaton-priv/*.h"
    # vapours headers do not pass verfication (ams_target)
    # "${CMAKE_CURRENT_SOURCE_DIR}/include/vapours/*.hpp"
)
add_library(megacpp STATIC ${SOURCES})
target_compile_options(megacpp PRIVATE
    $<$<COMPILE_LANGUAGE:CXX>:${MEGATON_LIB_GENERATED_CXX_FLAGS}>
    $<$<COMPILE_LANGUAGE:C>:${MEGATON_LIB_GENERATED_C_FLAGS}>
)

# Header verfication: make sure they compile by themselves
target_sources(megacpp PRIVATE
  FILE_SET private_headers TYPE HEADERS
  BASE_DIRS include/
  FILES ${MEGACPP_PRIVATE_HEADERS}
)
target_sources(megacpp PUBLIC
  FILE_SET public_headers TYPE HEADERS
  BASE_DIRS ${CMAKE_SOURCE_DIR}/include/
  FILES ${MEGATON_PUBLIC_HEADERS}
)
# private check already checks both public and private, and we don't
# have any interface headers
set_target_properties(megacpp PROPERTIES VERIFY_PRIVATE_HEADER_SETS ON)
add_subdirectory(headers)
