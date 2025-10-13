
#include "lib.hpp"
#include <string>

int32_t add(int32_t a, int32_t b) {
    return a + b;
}

rust::String hello(rust::Str name) {
    std::string s = "hello, ";
    s.append(name.data(), name.size());
    return rust::String(s);
}
