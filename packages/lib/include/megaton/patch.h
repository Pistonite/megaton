// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

/**
 * Runtime patching system
 */
#pragma once
#include <exl_armv8/prelude.h>
#include <megaton/__priv/mirror.h>

namespace megaton::patch {

/**
 * Initialize the patching system
 *
 * This is automatically called in the megaton entrypoint, before
 * user-defined megaton_main
 */
void init();

/**
 * Access the main read-only regions (text and rodata)
 */
const __priv::Mirror& main_ro();

/**
 * A Branch instruction payload
 *
 * Used with Stream to write a branch instruction.
 * The relative address will be resolved at write-time.
 */
class Branch {
public:
    explicit Branch(uintptr_t target, bool link) : target(target), link(link) {}

    exl::armv8::InstBitSet get_insn(uintptr_t ro_current_addr) {
        // relative address to jump to the function
        ptrdiff_t rel_addr = target - ro_current_addr;
        if (link) {
            return exl::armv8::inst::BranchLink(rel_addr);
        } else {
            return exl::armv8::inst::Branch(rel_addr);
        }
    }

private:
    uintptr_t target;
    bool link;
};

/** Create a branch to a function to write to a patching stream */
template <typename F> inline_always_ Branch b(F* func) {
    return Branch{reinterpret_cast<uintptr_t>(func), false};
}

/** Create a branch link to a function to write to a patching stream */
template <typename F> inline_always_ Branch bl(F* func) {
    return Branch{reinterpret_cast<uintptr_t>(func), true};
}

/**
 * A repeat payload
 *
 * Writing this payload is equivalent to writing the same instruction multiple
 * times
 */
template <typename T> class Repeat {
public:
    Repeat(T insn, size_t count) : insn(insn), count(count) {}
    T insn;
    size_t count;
};

template <typename T> inline_always_ Repeat<T> repeat(T insn, size_t count) {
    return {insn, count};
}

/**
 * A skip payload
 *
 * Writing thie payload is equivalent to calling skip() on the stream
 */
class Skip {
public:
    explicit Skip(size_t count) : count(count) {}
    size_t count;
};
inline_always_ Skip skip(size_t count) { return Skip{count}; }

/**
 * A patcher stream.
 *
 * The stream starts at an offset of the main module, and
 * the `<<` operator can be used to patch the instructions.
 * The stream will advance by the size of the instruction and
 * will automatically flush when destroyed.
 */
class Stream {
public:
    Stream(const __priv::Mirror& mirror, uintptr_t start_offset)
        : mirror(mirror) {
        rw_start_addr = mirror.rw_start() + start_offset;
        ro_start_addr = mirror.ro_start() + start_offset;
        rw_current_addr = rw_start_addr;
    }

    ~Stream() { flush(); }

    /** Flush the instructions written so far */
    void flush();

    Stream& operator<<(exl::armv8::InstBitSet inst) {
        write(inst);
        return *this;
    }

    Stream& operator<<(Branch branch) {
        // find the actual physical address of the write head
        uintptr_t ro_current_addr =
            rw_current_addr - mirror.rw_start() + mirror.ro_start();
        write(branch.get_insn(ro_current_addr));
        return *this;
    }

    template <typename T> Stream& operator<<(Repeat<T> repeat) {
        for (size_t i = 0; i < repeat.count; i++) {
            write(repeat.insn);
        }
        return *this;
    }

    Stream& operator<<(Skip skip) {
        this->skip(skip.count);
        return *this;
    }

private:
    /** The underlying memory */
    const __priv::Mirror& mirror;
    /** The starting address of the stream in the read-only region */
    uintptr_t ro_start_addr;
    /** The starting address of the stream in the read-write region */
    uintptr_t rw_start_addr;
    /** The current address of the stream (where to write next) in the RW region
     */
    uintptr_t rw_current_addr;

    exl::armv8::InstBitSet& at(const uintptr_t rw_offset) {
        auto ptr = reinterpret_cast<exl::armv8::InstBitSet*>(rw_offset);
        return *ptr;
    }

    /** Write the value at the current position of the stream, and advance */
    void write(exl::armv8::InstBitSet value) {
        at(rw_current_addr) = value;
        rw_current_addr += sizeof(exl::armv8::InstBitSet);
    }

    /** Skip `count` instructions. i.e. advance the stream without writing */
    void skip(size_t count) {
        rw_current_addr += sizeof(exl::armv8::InstBitSet) * count;
    }
};

/** Create a stream to patch the main module at the starting offset */
inline_always_ Stream main_stream(uintptr_t start_offset) {
    return {main_ro(), start_offset};
}

} // namespace megaton::patch
