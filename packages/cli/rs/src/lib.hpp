// packages/cli/rs/src/lib.hpp
#pragma once
#include <cstdint>
#include "rust/cxx.h"

int32_t add(int32_t a, int32_t b);
rust::String hello(rust::Str name);
