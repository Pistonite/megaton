
add_library(megarocrt OBJECT rocrt0.s rocrt.cpp)
target_compile_options(megarocrt PRIVATE ${MEGATON_LIB_GENERATED_AS_FLAGS})

target_include_directories(megarocrt PRIVATE
    "${MEGATON_PUBLIC_INCLUDE}"
    "${MEGATON_PRIVATE_INCLUDE}"
)
