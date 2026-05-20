// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#include <nn/os/os_Mutex.h>

// these symbols are wrong in the header and shouldn't exist
// might need to be fixed

// this is required to generate the correct destructor for some reason
// otherwise a different symbol is generated and it doesn't match with
// what usage sites expect
namespace nn::os {
Mutex::~Mutex() {
    nn::os::FinalizeMutex(&this->m_Mutex);
}

// these are required for std::lock_guard
void Mutex::lock() {
    this->Lock();
}

void Mutex::unlock() {
    this->Unlock();
}
}
