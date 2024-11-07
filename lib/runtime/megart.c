#include <megaton/prelude.h>

/**
 * module name needs to put at nx-module-name section
 */
struct module_name_t {
    i32 unknown;
    i32 name_len;
    u8 name[MEGART_MODULE_NAME_LEN + 1];
};

__attribute__((section(".nx-module-name"))) __attribute__((used))
const module_name_t s_module_name = {.unknown = 0,
                                     .name_len = MEGART_MODULE_NAME_LEN,
                                     .name = MEGART_MODULE_NAME};

const u8* __megaton_module_name() { return MEGART_NX_MODULE_NAME; }

u32 __megaton_module_name_len() { return MEGART_NX_MODULE_NAME_LEN; }

u64 __megaton_title_id() { return MEGART_TITLE_ID; }

const u8* __megaton_title_id_hex() { return MEGART_TITLE_ID_HEX; }

void __megaton_lib_init();
void __megaton_librs_init();
void megaton_main();

// real module entry point
void __megaton_module_entry() {
    // initialize C lib
    __megaton_lib_init();

    // initialize Rust lib
#ifdef MEGART_RUST
    __megaton_librs_init();
#endif

    // bootstrap done

    // call module main
    megaton_main();
}

// project can allow megaton to provide a main shim
// to directly call the rust side
#ifdef MEGART_RUST_MAIN
void __megaton_rs_main();
void megaton_main() { __megaton_rs_main(); }
#endif
