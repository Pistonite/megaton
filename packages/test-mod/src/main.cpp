// the "prelude" header include basic primitive types
// and common macros, just like a Rust prelude.
// This is part of the megaton (old) project
//
// In the old project, the user is required to clone a copy of the megaton repo,
// which contains the source code for the megaton library. Then,
// they will build it locally into a staticlib. megaton build tool
// will automatically find that lib (can even build it with --lib),
// and link it.
//
// However, this approach has some issues:
// - The user-specific build flags are not passed into megaton,
//   which can have some issues
// - Multiple projects will share the same local megaton lib
//
// To solve this, in the new megaton tool, we will
// tarball the megaton library into the megaton tool (so user
// don't have to clone a copy of the repo), and extract it 
// on-demand into the "target" directory (hidden away from the user),
// then, when the project is built, we will include the megaton library source
// into the build process.
#include <megaton/prelude.h>

namespace nn::fs {
void MountSdCard(const char* path);
}
// these are library headers from botw and botw-symbols
// #include <toolkit/msg/screen.hpp>
// #include <toolkit/tcp.hpp>
// #include <gfx/seadTextWriter.h>

// this is the public header of our mod, include/example-mod/my_mod.h
#include <new-example-mod/my_mod.h>
// this is the generated header from the cxx::bridge in lib.rs
// you can find it in target/megaton-new/none/include
// you will notice that it contains the get_int_from_rust
// function that we declared in rust to be "visible" to C++,
// this is how it's done
#include <lib.h>
#include <rust/cxx.h>
// this contains the rust_main function, I don't really
// have a solid idea for how the bootstraping will work
// in the new library yet. Maybe this can be brainstormed
// by the Library Team
//
// Essentially, here are the different combinations of setup users can have:
// - Only C/C++ code, no Rust
// - Only Rust code, no C/C++
// - Both , but user-perceived entry point is in C
// - Both , but user-perceived entry point is in Rust
// - For simplicity, we probably want to disallow the case
//   where user want entry points in both C and Rust,
//   because initialization order can get tricky in that case.
//
// Since this example uses old megaton, which only has C entry point,
// this header serves as a temporary solution to just call
// our Rust entry point from C entry point
#include "megaton_header.h"
// this is the implementation of calling rust_main.
// As you can see in lib.rs, __megaton_rust_main is implemented
// there, and here we just declare a symbol with extern "C",
// which has the same effect as no_mangle + extern "C" in Rust.
// extern "C" void __megaton_rust_main();
// namespace megaton {
// void rust_main() {
//     __megaton_rust_main();
// }
// }

// here's the main function recognized by old megaton
// i.e. when our binary is loaded by the OS, this will be called
// extern "C" void megaton_main() {
//     megaton::rust_main();
// }

// maybe needed?
extern "C" void __megaton_rs_main();

extern "C" void megaton_main() {
    __megaton_rs_main();
}

#include "main.h"
// #include <string>


extern "C" i64 sys_write(void* fd, u8* buf, usize len) {
    // botw::tcp::sendf("calling write to fd %p\n", fd);
    for (usize i = 0; i < len; i++) {
        // botw::tcp::sendf("%c ", buf[i]);
    }
    // botw::tcp::sendf("called write to fd %p\n", fd);
    return len;
}

extern "C" i64 sys_writev(void* fd, void* iov, usize iovcnt) {
    // botw::tcp::sendf("calling writev to fd %p\n", fd);
    return iovcnt;
}

// static bool tests_run = false;
// static rust::String g_MY_STRING;
static FILE* f;

namespace example {

void write_test_output(rust::Str data) {
    fwrite(data.data(), sizeof(char), data.length(), f);
}

// as you can see in lib.rs, this is the function that will be 
// called by Rust
// (of course, in a real world example, you wouldn't just call
// back and forth like this)
void init_function_in_c() {
    // initialize the screen printer mod
    // botw::msg::screen::init(compute, render);
    // initialize TCP mod
    // botw::tcp::init();
    // start server on port 5001
    // botw::tcp::start_server(5001);
    // g_MY_STRING = std::string(example_rs::run_megaton_tests());
    nn::fs::MountSdCard("sd");
    f = fopen("sd:/test_output.txt", "w");
    example_rs::run_megaton_tests();
    fclose(f);
}


// this computes the data needed for printing
// void compute() {
//     if (!tests_run) {
//         g_MY_STRING = std::string(example_rs::run_megaton_tests());
//         tests_run = true;
//     }
// }

// this prints to the screen
// void render(sead::TextWriter* p) {
//     // null check to be good
//     if (!p) {
//         return;
//     }
//     // note that compute() and render() are guaranteed
//     // to be called from the same thread.
//     // If there are no other writers to g_MY_INT,
//     // then we don't have race condition with the global variable
//     // p->printf("Tests runnings...\n");
//     if (tests_run) {
//         // p->printf("std::string %s\n", g_MY_STRING.c_str());
//     } else {
//         // p->printf("Tests runnings...\n");
//     }
// }

}

// This function is currently unused
// 
// For the Library Team, one of your first task is to figure out
// what to do when a panic happens in Rust.
//
// Rust has 2 panic modes: panic-abort and panic-unwind. You can read about it more,
// but in our case, it doesn't really matter. When a panic occurs, we have do these things
// and these things only:
// - save the panic information in some way (maybe sd card?),
//   so the user can later know the reason of the panic
// - crash the game, this is easy by triggering a segmentation fault
//   megaton old lib has an implementation, see panic_abort.cpp in old megaton lib
//
// The second thing to do, is to setup logging, so you can print stuff
// and see the values. an easy way is to create an ffi binding for
// the botw::tcp::sendf function, and just call that from Rust.
// Then you can use the tcp python script to get the real time logs.
//
// However, the final product probably needs something more mature.
// For example, integration with the `log` crate.
// Maybe user can call `megaton::init_tcp_debugging()` to setup TCP logging,
// and the Build Tool will have a `megaton debug` command to automatically connect
// to the console's IP, this of course, requires you to get tcp working in
// the Rust standard library
//
// You will see a theme like this, where you hack in some method to unblock
// development of the std library feature, and use that feature to properly
// implement the thing you originally wanted.

// extern "C" void handle_panic() {
//     botw::tcp::sendf("rust panicked\n");
// }
// extern "C" void megaton_main() {
// 
// }
