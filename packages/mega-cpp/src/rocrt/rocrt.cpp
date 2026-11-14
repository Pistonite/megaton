extern "C" void megaton_main();
extern "C" {
    extern unsigned char            __EX_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __EX_end[] __attribute__((visibility("hidden")));
    extern unsigned char            __tdata_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __tdata_end[] __attribute__((visibility("hidden")));
    extern unsigned char            __tdata_align_abs[] __attribute__((visibility("hidden")));
    extern unsigned char            __tdata_align_rel[] __attribute__((visibility("hidden")));
    extern unsigned char            __tbss_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __tbss_end[] __attribute__((visibility("hidden")));
    extern unsigned char            __tbss_align_abs[] __attribute__((visibility("hidden")));
    extern unsigned char            __tbss_align_rel[] __attribute__((visibility("hidden")));
    extern unsigned char            __rela_dyn_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __rela_dyn_end[] __attribute__((visibility("hidden")));
    extern unsigned char            __rel_dyn_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __rel_dyn_end[] __attribute__((visibility("hidden")));
    extern unsigned char            __rela_plt_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __rela_plt_end[] __attribute__((visibility("hidden")));
    extern unsigned char            __rel_plt_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __rel_plt_end[] __attribute__((visibility("hidden")));
    extern unsigned char            __got_plt_start[] __attribute__((visibility("hidden")));
    extern unsigned char            __got_plt_end[] __attribute__((visibility("hidden")));
    extern unsigned char            _DYNAMIC[] __attribute__((visibility("hidden")));
    int __nnmusl_init_dso(unsigned char *EX_start, unsigned char *EX_end,
                            unsigned char *tdata_start, unsigned char *tdata_end,
                            unsigned char *tdata_align_abs, unsigned char *tdata_align_rel,
                            unsigned char *tbss_start, unsigned char *tbss_end,
                            unsigned char *tbss_align_abs, unsigned char *tbss_align_rel,
                            unsigned char *got_plt_start, unsigned char *got_plt_end,
                            unsigned char *rela_dyn_start, unsigned char *rela_dyn_end,
                            unsigned char *rel_dyn_start, unsigned char *rel_dyn_end,
                            unsigned char *rela_plt_start, unsigned char *rela_plt_end,
                            unsigned char *rel_plt_start, unsigned char *rel_plt_end,
                            unsigned char *DYNAMIC);
    static volatile int nnmuslTlsInitializationPhase __attribute__((section(".data._ZL28nnmuslTlsInitializationPhase"))) = 0;
}
extern "C" void __megacrt_init() {
    if (nnmuslTlsInitializationPhase == 0)
    {
        nnmuslTlsInitializationPhase = __nnmusl_init_dso( __EX_start, __EX_end,
                                        __tdata_start, __tdata_end,
                                        __tdata_align_abs, __tdata_align_rel,
                                        __tbss_start, __tbss_end,
                                        __tbss_align_abs, __tbss_align_rel,
                                        __got_plt_start, __got_plt_end,
                                        __rela_dyn_start, __rela_dyn_end,
                                        __rel_dyn_start, __rel_dyn_end,
                                        __rela_plt_start, __rela_plt_end,
                                        __rel_plt_start, __rel_plt_end,
                                        static_cast<unsigned char *>(_DYNAMIC) );
        if (nnmuslTlsInitializationPhase == 1)
        {
            return;
        }
    }
    megaton_main();
}
