#include <megaton/prelude.h>

#include <test-mod/mod.h>
#include <lib.h>
#include <rust/cxx.h>

namespace nn::fs {
    void MountSdCard(const char* path);
}

extern "C" void __megaton_rs_main();

extern "C" void megaton_main() {
    __megaton_rs_main();
}


extern "C" i64 sys_write(void* fd, u8* buf, usize len) {
    return len;
}

extern "C" i64 sys_writev(void* fd, void* iov, usize iovcnt) {
    return iovcnt;
}

static FILE* f;

void write_test_output(rust::Str data) {
    fwrite(data.data(), sizeof(char), data.length(), f);
}

void init_function_in_c() {
    nn::fs::MountSdCard("sd");
    f = fopen("sd:/test_output.txt", "w");
    example_rs::run_megaton_tests();
    fclose(f);
}
