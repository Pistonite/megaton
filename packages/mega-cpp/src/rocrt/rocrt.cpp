extern "C" void megaton_main();
extern "C" void __megacrt_init() {
    megaton_main();
}
