// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

extern "C" void megaton_main();
extern "C" void __megacrt_init() {
    megaton_main();
}
