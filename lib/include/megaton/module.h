/**
 * Header intended to be included by the project
 * for module info.
 */
#pragma once
#include <megaton/prelude.h>

#ifdef __cplusplus
extern "C" {
#endif

extern const u8* __megaton_module_name(void);
extern u32 __megaton_module_name_len(void);
extern const u8* __megaton_title_id_hex(void);
extern u64 __megaton_title_id(void);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
namespace megaton {

inline_always_ const u8* module_name() {
    return __megaton_module_name();
}

inline_always_ u32 module_name_len() {
    return __megaton_module_name_len();
}

inline_always_ const u8* title_id_hex() {
    return __megaton_title_id_hex();
}

inline_always_ u64 title_id() {
    return __megaton_title_id();
}

}
#endif
