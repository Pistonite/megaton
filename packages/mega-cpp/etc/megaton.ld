/*
 * Linker script for linking the final module ELF.
 *
 * Credits: 
 * - libnx 
 * - exlaunch by shadowninja108 (https://github.com/shadowninja108)
 */
OUTPUT_FORMAT(elf64-littleaarch64)
OUTPUT_ARCH(aarch64)

/** Entry point*/
ENTRY(_start)

PHDRS
{
    text    PT_LOAD FLAGS(5); /* R + X */
    rodata  PT_LOAD FLAGS(4); /* R */
    data    PT_LOAD FLAGS(6); /* R + W */
    dynamic PT_DYNAMIC;
}

SECTIONS
{
    PROVIDE_HIDDEN(__start__ = 0x0);
    . = __start__;

    /* === Executable Code === */
    __code_start__ = .;

    .text : {
        HIDDEN(__text_start__ = .);
        KEEP (*(.text.rocrt_init))
        KEEP (*(.text.rocrt_info))
        KEEP (*(.text.crt0))
        *(.text .text.*)
        . = ALIGN(8);
        HIDDEN(__text_end__ = .);
    } :text

    .init :
    {
        KEEP( *(.init) )
        . = ALIGN(8);
    } :text

    /* Procedure Linkage Table: trampoline and stuffs */
    .plt : { *(.plt .plt.* .iplt) } :text

    .fini :
    {
        KEEP( *(.fini) )
        . = ALIGN(8);
    } :text

    __code_end__ = .;

    /* ======================== RODATA ======================== */
    . = ALIGN(0x1000);
    PROVIDE_HIDDEN( __rodata_start = . );

    /* module name */
    .nx-module-name : { KEEP (*(.nx-module-name)) } :rodata

    /* All the symbols needed for relocation lookup */
    __rel_dyn_start__ = .;
    .rel.dyn : { *(.rel.dyn) } :rodata
    __rel_dyn_end__ = .;

    __rela_dyn_start__ = .;
    .rela.dyn : { *(.rela.dyn) } :rodata
    __rela_dyn_end__ = .;

    __rel_plt_start__ = .;
    .rel.plt        : { *(.rel.plt) } :rodata
    __rel_plt_end__ = .;

    __rela_plt_start__ = .;
    .rela.plt       : { *(.rela.plt) } :rodata
    __rela_plt_end__ = .;

    .hash     : { *(.hash) } :rodata
    .gnu.hash : { *(.gnu.hash) } :rodata
    .dynsym   : { *(.dynsym .dynsym.*) } :rodata
    .dynstr   : { *(.dynstr .dynstr.*) } :rodata

    /* Make sure everything is aligned */
    . = ALIGN(8);

    .rodata :
    {
        *(.rodata .rodata.* .gnu.linkonce.r.*)
        . = ALIGN(8);
    } :rodata

    .tls.align :
    {
        QUAD( MAX( ALIGNOF(.tdata), ALIGNOF(.tbss) ) )
    } :rodata

    PROVIDE_HIDDEN( __tls_align = ADDR(.tls.align) );

    /* All exception handling sections */
    .gcc_except_table : { *(.gcc_except_table .gcc_except_table.*) } :rodata
    .eh_frame_hdr : {
    HIDDEN(__eh_frame_hdr_start__ = .);
    *(.eh_frame_hdr) *(.eh_frame_entry .eh_frame_entry.*)
    HIDDEN(__eh_frame_hdr_end__ = .);
    } :rodata
    .eh_frame : { KEEP (*(.eh_frame)) *(.eh_frame.*) } :rodata
    .gnu_extab         : { *(.gnu_extab*) } : rodata
    .exception_ranges  : { *(.exception_ranges .exception_ranges*) } :rodata

    /* Misc .rodata stuffs (build-id, etc.) */
    .note.gnu.build-id : { *(.note.gnu.build-id) } :rodata

    /* ======================== RELRO ======================== */
    . = ALIGN(0x1000);
    PROVIDE_HIDDEN( __relro_start = . );

    .preinit_array ALIGN(8) :
    {
        PROVIDE_HIDDEN (__preinit_array_start = .);
        KEEP (*(.preinit_array))
        PROVIDE_HIDDEN (__preinit_array_end = .);
    } :data

    /* App init array */
    .init_array : {
        PROVIDE_HIDDEN (__init_array_start = .);
        KEEP (*(SORT_BY_INIT_PRIORITY(.init_array.*)))
        KEEP (*(.init_array))
        PROVIDE_HIDDEN (__init_array_end  = .);
    } :data

    /* App fini array */
    .fini_array : {
        PROVIDE_HIDDEN (__fini_array_start__ = .);
        KEEP (*(SORT_BY_INIT_PRIORITY(.fini_array.*)))
        KEEP (*(.fini_array))
        PROVIDE_HIDDEN (__fini_array_end__ = .);
    } :data

    .data.rela.ro :
    {
        *(.data.rela.ro.local* .gnu.linkonce.d.rela.ro.local.*)
        *(.data.rela.ro .data.rela.ro.* .gnu.linkonce.d.rela.ro.*)
        . = ALIGN(8);
    } :data
    .data.rel.ro :
    {
        *(.data.rel.ro.local* .gnu.linkonce.d.rel.ro.local.*)
        *(.data.rel.ro .data.rel.ro.* .gnu.linkonce.d.rel.ro.*)
        . = ALIGN(8);
    } :data
    .dynamic : {
        HIDDEN(__dynamic_start__ = .);
        *(.dynamic)
    } :data :dynamic

    __got_start__ = .;
    .got : { *(.got) *(.igot) } :data
    .got.plt : { *(.got.plt) *(.igot.plt) } :data
    __got_end__ = .;

    /* ======================== DATA ======================== */
    . = ALIGN(0x1000);
	PROVIDE_HIDDEN( __data_start = . );

    .data :
    {
        *(.data .data.* .gnu.linkonce.d.*)
        SORT(CONSTRUCTORS)
        . = ALIGN(8);
    } :data

    .tdata :
    {
        *(.tdata .tdata.* .gnu.linkonce.td.*)
        . = ALIGN(8);
    } :data

    PROVIDE_HIDDEN( __tdata_lma     = ADDR(.tdata)                  );
    PROVIDE_HIDDEN( __tdata_lma_end = ADDR(.tdata) + SIZEOF(.tdata) );

    .tbss :
    {
        __tbss_align_abs__ = ABSOLUTE(.);
        __tbss_start__ = .;
        *(.tbss .tbss.* .gnu.linkonce.tb.*)
        *(.tcommon)
        . = ALIGN(8);
        __tbss_end__ = .;
    } :data

    /* BSS section */
    . = ALIGN(0x1000);

    .bss : {
        *(.bss .bss.* .gnu.linkonce.b.*)
        *(COMMON)
        . = ALIGN(8);
    } :data

    /* Reserve space for the TLS segment of the main thread */
    .main.tls ALIGN(MAX(ALIGNOF(.tdata),ALIGNOF(.tbss))) :
    {
        . += SIZEOF(.tdata);
        . = ALIGN(ALIGNOF(.tbss));
        . += SIZEOF(.tbss);
    } :data

    PROVIDE_HIDDEN( __tls_start   = ADDR(.main.tls)                     );
	PROVIDE_HIDDEN( __tls_end     = ADDR(.main.tls) + SIZEOF(.main.tls) );

	PROVIDE_HIDDEN( __bss_start__ = ADDR(.bss) );
	PROVIDE_HIDDEN( __bss_end__   = __tls_end  );

    PROVIDE_HIDDEN( __end__ = ABSOLUTE(.) );

    /* =========== Argument buffer =========== */
    . = ALIGN(0x1000);
    PROVIDE_HIDDEN( __argdata__ = . );

    /DISCARD/ : {
        /* No need of the interpreter */
        *(.interp)
    }

}
