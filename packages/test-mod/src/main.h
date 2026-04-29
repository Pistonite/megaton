#pragma once

// see main.cpp
// this header exists to make the symbols available,
// otherwise we have to put the functions in the cpp file
// in a certain order

namespace sead {
class TextWriter;
}
namespace example {
void compute();
void render(sead::TextWriter* p);
}
