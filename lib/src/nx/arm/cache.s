/*
Copyright 2017-2018 libnx Authors

Permission to use, copy, modify, and/or distribute this software for 
any purpose with or without fee is hereby granted, provided that 
the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES 
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF 
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR 
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES 
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN 
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF 
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/
.macro CODE_BEGIN name
	.section .text.\name, "ax", %progbits
	.global \name
	.type \name, %function
	.align 2
	.cfi_startproc
\name:
.endm

.macro CODE_END
	.cfi_endproc
.endm

CODE_BEGIN armDCacheFlush
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

	mov w1, #1
	mrs x0, tpidrro_el0
	strb w1, [x0, #0x104]

armDCacheFlush_L0:
	dc  civac, x8
	add x8, x8, x9
	cmp x8, x10
	bcc armDCacheFlush_L0

	dsb sy

	strb wzr, [x0, #0x104]

	ret
CODE_END

CODE_BEGIN armDCacheClean
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

	mov w1, #1
	mrs x0, tpidrro_el0
	strb w1, [x0, #0x104]

armDCacheClean_L0:
	dc  cvac, x8
	add x8, x8, x9
	cmp x8, x10
	bcc armDCacheClean_L0

	dsb sy

	strb wzr, [x0, #0x104]

	ret
CODE_END

CODE_BEGIN armICacheInvalidate
	add x1, x1, x0
	mrs x8, CTR_EL0
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

	mov w1, #1
	mrs x0, tpidrro_el0
	strb w1, [x0, #0x104]

armICacheInvalidate_L0:
	ic  ivau, x8
	add x8, x8, x9
	cmp x8, x10
	bcc armICacheInvalidate_L0

	dsb sy
	isb

	strb wzr, [x0, #0x104]

	ret
CODE_END

CODE_BEGIN armDCacheZero
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

	mov w1, #1
	mrs x0, tpidrro_el0
	strb w1, [x0, #0x104]

armDCacheZero_L0:
	dc  zva, x8
	add x8, x8, x9
	cmp x8, x10
	bcc armDCacheZero_L0

	dsb sy

	strb wzr, [x0, #0x104]

	ret
CODE_END
