#pragma once

#include <rust/cxx.h>
// look at this file and "src/lib.rs" at the same time

// the user will probably have some "outmost" namespace
// that they put all of their code in.
// Here we assume it's "example"
namespace example {

// here, we define some functions that are exposed to Rust
// we also need to dupe the declaration on the Rust side
void write_test_output(rust::Str data);
void init_function_in_c();

}
