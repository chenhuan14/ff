const MODULUS: [u64; 4] = [0xd0970e5ed6f72cb7, 0xa6682093ccc81082, 0x6673b0101343b00, 0xe7db4ea6533afa9];
const INV: u64 = 0x1ba3a358ef788ef9;

#[allow(dead_code)]
#[allow(clippy::too_many_lines)]
#[inline(always)]
#[cfg(all(target_arch = "x86_64", target_feature = "adx"))]
pub fn mont_mul_asm_adx(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    use core::mem::MaybeUninit;

    static ZERO: u64 = 0;
    static MODULUS_0: u64 = 0xd0970e5ed6f72cb7;
    static MODULUS_1: u64 = 0xa6682093ccc81082;
    static MODULUS_2: u64 = 0x6673b0101343b00;
    static MODULUS_3: u64 = 0xe7db4ea6533afa9;
    static INV: u64 = 0x1ba3a358ef788ef9;

    static MODULUS_0_INV: u64 = MODULUS_0.wrapping_neg();
    static MODULUS_1_INV: u64 = MODULUS_1.wrapping_neg();
    static MODULUS_2_INV: u64 = MODULUS_2.wrapping_neg();
    static MODULUS_3_INV: u64 = MODULUS_3.wrapping_neg();

    // this is CIOS multiplication when top bit for top word of modulus is not set

    // let mut result = MaybeUninit::<[u64; 4]>::uninit();

    let mut r0: u64;
    let mut r1: u64;
    let mut r2: u64;
    let mut r3: u64;

    // mulx    dest_hi, dest_lo, src1  
    // use notation of order (hi, lo)

    // |     | b3  | b2  | b1  | b0  |
    // |     |     |     |     | a0  |
    // |---- |---- |---- |---- |---- |
    // |     |     |     | r14 | r13 |
    // |     |     | r9  | r8  |     |
    // |     | r10 | r15 |     |     |
    // | r12 | rdi |     |     |     |
    // |---- |---- |---- |---- |---- |
    // |     |     |     |     |     | // rdx = m, r11 = garbage
    // |     |     |  CF | r14 |     |
    // |  OF | r10 |     |     |     | 
    // |---- |---- |---- |---- |---- |
    // |     | CF  | r15 |     |     |
    // | r12 |     |     |     |     |
    // | CF  | r10 |     |     |     |
    // |---- |---- |---- |---- |---- |
    // | r12 | r10 | r15 | r14 | r13 |

    unsafe {
        asm!(
            // round 0
            "mov rdx, qword ptr [{a_ptr} + 0]",
            "xor r8d, r8d",
            "mulx r14, r13, qword ptr [{b_ptr} + 0]", // (r14, r13) = a[0] * b[0]
            "mulx r9, r8, qword ptr [{b_ptr} + 8]", // (r9, r8) = a[0] * b[1]
            "mulx r10, r15, qword ptr [{b_ptr} + 16]", // (r10, r15) = a[0] * b[2]
            "mulx r12, rdi, qword ptr [{b_ptr} + 24]", // (r12, rdi) = a[0] * b[3]
            // by this moment MULX for a[0] * b[0] is complete (latency = 4)
            "mov rdx, r13", // rdx = r13 = (a[0] * b[0]).l0
            "mov r11, {inv}",
            "mulx r11, rdx, r11", // (r11, rdx) = (a[0] * b[0]).lo * k, so rdx = m (we overwrite rdx cause (a[0] * b[0]).lo is not needed for anything else)
            // "mulx r11, rdx, qword ptr [rip + {inv_ptr}]", // (r11, rdx) = (a[0] * b[0]).lo * k, so rdx = m (we overwrite rdx cause (a[0] * b[0]).lo is not needed for anything else)
            "adcx r14, r8", // r14 = r14 + r8 = (a[0] * b[0]).hi + (a[0] * b[1]).lo, carry flag is set in CF register (CF = carry into 2nd word), 1st word calculation
            "adox r10, rdi", // r10 = r10 + rdi = (a[0] * b[2]).hi + (a[0] * b[3]).lo, carry flag is set in OF register (OF = carry into 4th word), 3rd word calculation 
            "adcx r15, r9", // r15 = r15 + r9 + CF = (a[0] * b[1]).hi + (a[0] * b[2]).lo + CF, 2nd word continuation
            "mov r11, 0",
            "adox r12, r11", // r12 = r12 + OF = 4th word
            "adcx r10, r11", // r10 = r10 + CF, 3rd word continuation
            // "adox r12, qword ptr [rip + {zero_ptr}]", // r12 = r12 + OF = 4th word
            // "adcx r10, qword ptr [rip + {zero_ptr}]", // r10 = r10 + CF, 3rd word continuation
            // "mov r8, {q0}",
            // "mulx r9, r8, r8", // (r9, r8) = m * q0
            "mulx r9, r8, qword ptr [rip + {q0_ptr}]", // (r9, r8) = m * q0
            // "mov rdi, {q1}",
            // "mulx r11, rdi, rdi", // (r11, rdi) = m * q1
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]", // (r11, rdi) = m * q1
            "adox r13, r8", // r13 = t[0] + (m * q0).lo, set OF
            "adcx r14, rdi", // r14 = t[1] + (m * q1).lo, set CF
            "adox r14, r9", // r14 = t[1] + (m * q0).hi + OF, set OF
            "adcx r15, r11", // r15 = t[2] + (m * q1).hi + CF, set CF
            // "mov r8, {q2}",
            // "mulx r9, r8, r8", // (r9, r8) = m * q2
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]", // (r9, r8) = m * q2
            // "mov rdi, {q3}",
            // "mulx r11, rdi, rdi", // (r11, rdi) = m * q3
            "mulx r11, rdi, qword ptr [rip + {q3_ptr}]", // (r11, rdi) = m * q3
            "adox r15, r8", // r15 = t[2] + (m * q2).lo + OF, set OF
            "adcx r10, rdi", // r10 = t[3] + (m * q3).lo + CF, set CF
            "adox r10, r9", // r10 = t[3] + (m * q2).hi + OF, set OF
            "adcx r12, r11", // r12 = t[4] + (m * q3).hi + CF, set CF
            "mov r9, 0",
            "adox r12, r9", // r12 = r12 + OF
            // "adox r12, qword ptr [rip + {zero_ptr}]", // r12 = r12 + OF

            // round 1
            "mov rdx, qword ptr [{a_ptr} + 8]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r14, r8",
            "adox r15, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r15, rdi",
            "adox r10, r11",
            // "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r13, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r10, r8",
            "adox r12, rdi",
            "adcx r12, r9",
            "mov rdi, 0",
            "adox r13, rdi",
            "adcx r13, rdi",
            // "adox r13, qword ptr [rip + {zero_ptr}]",
            // "adcx r13, qword ptr [rip + {zero_ptr}]",
            "mov rdx, r14",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            // "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            // "mov r8, {q0}",
            // "mulx r9, r8, r8",
            "mulx r9, r8, qword ptr [rip + {q0_ptr}]",
            // "mov rdi, {q1}",
            // "mulx r11, rdi, rdi",
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]",
            "adox r14, r8",
            "adcx r15, rdi",
            "adox r15, r9",
            "adcx r10, r11",
            // "mov r8, {q2}",
            // "mulx r9, r8, r8",
            // "mov rdi, {q3}",
            // "mulx r11, rdi, rdi",
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q3_ptr}]",
            "adox r10, r8",
            "adcx r12, r9",
            "adox r12, rdi",
            "adcx r13, r11",
            "mov rdi, 0",
            "adox r13, rdi",
            // "adox r13, qword ptr [rip + {zero_ptr}]",

            // round 2
            "mov rdx, qword ptr [{a_ptr} + 16]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r15, r8",
            "adox r10, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r10, rdi",
            "adox r12, r11",
            // "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r14, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r12, r8",
            "adox r13, r9",
            "adcx r13, rdi",
            "mov r9, 0",
            "adox r14, r9",
            "adcx r14, r9",
            // "adox r14, qword ptr [rip + {zero_ptr}]",
            // "adcx r14, qword ptr [rip + {zero_ptr}]",
            "mov rdx, r15",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            // "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            // "mov r8, {q0}",
            // "mulx r9, r8, r8",
            // "mov rdi, {q1}",
            // "mulx r11, rdi, rdi",
            "mulx r9, r8, qword ptr [rip + {q0_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]",
            "adox r15, r8",
            "adcx r10, r9",
            "adox r10, rdi",
            "adcx r12, r11",
            // "mov r8, {q2}",
            // "mulx r9, r8, r8",
            // "mov rdi, {q3}",
            // "mulx r11, rdi, rdi",
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q3_ptr}]",
            "adox r12, r8",
            "adcx r13, r9",
            "adox r13, rdi",
            "adcx r14, r11",
            "mov rdi, 0",
            "adox r14, rdi",
            // "adox r14, qword ptr [rip + {zero_ptr}]",

            // round 3
            "mov rdx, qword ptr [{a_ptr} + 24]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r10, r8",
            "adox r12, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r12, rdi",
            "adox r13, r11",
            // "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r15, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r13, r8",
            "adox r14, r9",
            "adcx r14, rdi",
            "mov r9, 0",
            "adox r15, r9",
            "adcx r15, r9",
            // "adox r15, qword ptr [rip + {zero_ptr}]",
            // "adcx r15, qword ptr [rip + {zero_ptr}]",
            "mov rdx, r10",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            // "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            // "mov r8, {q0}",
            // "mulx r9, r8, r8",
            // "mov rdi, {q1}",
            // "mulx r11, rdi, rdi",

            "mulx r9, r8, qword ptr [rip + {q0_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]",
            "adox r10, r8",
            "adcx r12, r9",
            "adox r12, rdi",
            "adcx r13, r11",
            // "mov r8, {q2}",
            // "mulx r9, r8, r8",
            // "mov rdi, {q3}",
            // "mulx rdx, rdi, rdi",
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]",
            "mulx rdx, rdi, qword ptr [rip + {q3_ptr}]",
            "adox r13, r8",
            "adcx r14, r9",
            "adox r14, rdi",
            "adcx r15, rdx",
            "mov rdi, 0",
            "adox r15, rdi",
            // "adox r15, qword ptr [rip + {zero_ptr}]",

            // "mov [{out_ptr} + 0], r12",
            // "mov [{out_ptr} + 8], r13",
            // "mov [{out_ptr} + 16], r14",
            // "mov [{out_ptr} + 24], r15",

            // zero_ptr = sym ZERO,
            // inv_ptr = sym INV,
            // q0 = const 0xd0970e5ed6f72cb7u64,
            // q1 = const 0xa6682093ccc81082u64,
            // q2 = const 0x6673b0101343b00u64,
            // q3 = const 0xe7db4ea6533afa9u64,

            q0_ptr = sym MODULUS_0,
            q1_ptr = sym MODULUS_1,
            q2_ptr = sym MODULUS_2,
            q3_ptr = sym MODULUS_3,
            inv = const 0x1ba3a358ef788ef9u64,
            // out_ptr = in(reg) result.as_mut_ptr(),
            a_ptr = in(reg) a.as_ptr(),
            b_ptr = in(reg) b.as_ptr(),
            out("rdx") _, 
            out("rdi") _, 
            out("r8") _, 
            out("r9") _, 
            out("r10") _, 
            out("r11") _, 
            out("r12") r0, 
            out("r13") r1, 
            out("r14") r2, 
            out("r15") r3,
            options(pure, readonly, nostack)
        );
    }

    [r0, r1, r2, r3]

    // let result = unsafe { result.assume_init() };

    // result
}


#[allow(dead_code)]
#[allow(clippy::too_many_lines)]
#[inline(always)]
#[cfg(all(target_arch = "x86_64", target_feature = "adx"))]
pub fn mont_mul_asm_adx_with_reduction(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    use core::mem::MaybeUninit;

    static ZERO: u64 = 0;
    static MODULUS_0: u64 = 0xd0970e5ed6f72cb7;
    static MODULUS_1: u64 = 0xa6682093ccc81082;
    static MODULUS_2: u64 = 0x6673b0101343b00;
    static MODULUS_3: u64 = 0xe7db4ea6533afa9;
    static INV: u64 = 0x1ba3a358ef788ef9;

    static MODULUS_0_INV: u64 = MODULUS_0.wrapping_neg();
    static MODULUS_1_INV: u64 = MODULUS_1.wrapping_neg();
    static MODULUS_2_INV: u64 = MODULUS_2.wrapping_neg();
    static MODULUS_3_INV: u64 = MODULUS_3.wrapping_neg();

    // this is CIOS multiplication when top bit for top word of modulus is not set

    // let mut result = MaybeUninit::<[u64; 4]>::uninit();

    let mut r0: u64;
    let mut r1: u64;
    let mut r2: u64;
    let mut r3: u64;

    // mulx    dest_hi, dest_lo, src1  
    // use notation of order (hi, lo)

    // |     | b3  | b2  | b1  | b0  |
    // |     |     |     |     | a0  |
    // |---- |---- |---- |---- |---- |
    // |     |     |     | r14 | r13 |
    // |     |     | r9  | r8  |     |
    // |     | r10 | r15 |     |     |
    // | r12 | rdi |     |     |     |
    // |---- |---- |---- |---- |---- |
    // |     |     |     |     |     | // rdx = m, r11 = garbage
    // |     |     |  CF | r14 |     |
    // |  OF | r10 |     |     |     | 
    // |---- |---- |---- |---- |---- |
    // |     | CF  | r15 |     |     |
    // | r12 |     |     |     |     |
    // | CF  | r10 |     |     |     |
    // |---- |---- |---- |---- |---- |
    // | r12 | r10 | r15 | r14 | r13 |

    unsafe {
        asm!(
            // round 0
            "mov rdx, qword ptr [{a_ptr} + 0]",
            "xor r8d, r8d",
            "mulx r14, r13, qword ptr [{b_ptr} + 0]", // (r14, r13) = a[0] * b[0]
            "mulx r9, r8, qword ptr [{b_ptr} + 8]", // (r9, r8) = a[0] * b[1]
            "mulx r10, r15, qword ptr [{b_ptr} + 16]", // (r10, r15) = a[0] * b[2]
            "mulx r12, rdi, qword ptr [{b_ptr} + 24]", // (r12, rdi) = a[0] * b[3]
            // by this moment MULX for a[0] * b[0] is complete (latency = 4)
            "mov rdx, r13", // rdx = r13 = (a[0] * b[0]).l0
            "mov r11, {inv}",
            "mulx r11, rdx, r11", // (r11, rdx) = (a[0] * b[0]).lo * k, so rdx = m (we overwrite rdx cause (a[0] * b[0]).lo is not needed for anything else)
            "adcx r14, r8", // r14 = r14 + r8 = (a[0] * b[0]).hi + (a[0] * b[1]).lo, carry flag is set in CF register (CF = carry into 2nd word), 1st word calculation
            "adox r10, rdi", // r10 = r10 + rdi = (a[0] * b[2]).hi + (a[0] * b[3]).lo, carry flag is set in OF register (OF = carry into 4th word), 3rd word calculation 
            "adcx r15, r9", // r15 = r15 + r9 + CF = (a[0] * b[1]).hi + (a[0] * b[2]).lo + CF, 2nd word continuation
            "mov r11, 0",
            "adox r12, r11", // r12 = r12 + OF = 4th word
            "adcx r10, r11", // r10 = r10 + CF, 3rd word continuation
            "mulx r9, r8, qword ptr [rip + {q0_ptr}]", // (r9, r8) = m * q0
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]", // (r11, rdi) = m * q1
            "adox r13, r8", // r13 = t[0] + (m * q0).lo, set OF
            "adcx r14, rdi", // r14 = t[1] + (m * q1).lo, set CF
            "adox r14, r9", // r14 = t[1] + (m * q0).hi + OF, set OF
            "adcx r15, r11", // r15 = t[2] + (m * q1).hi + CF, set CF
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]", // (r9, r8) = m * q2
            "mulx r11, rdi, qword ptr [rip + {q3_ptr}]", // (r11, rdi) = m * q3
            "adox r15, r8", // r15 = t[2] + (m * q2).lo + OF, set OF
            "adcx r10, rdi", // r10 = t[3] + (m * q3).lo + CF, set CF
            "adox r10, r9", // r10 = t[3] + (m * q2).hi + OF, set OF
            "adcx r12, r11", // r12 = t[4] + (m * q3).hi + CF, set CF
            "mov r9, 0",
            "adox r12, r9", // r12 = r12 + OF

            // round 1
            "mov rdx, qword ptr [{a_ptr} + 8]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r14, r8",
            "adox r15, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r15, rdi",
            "adox r10, r11",
            "mulx r13, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r10, r8",
            "adox r12, rdi",
            "adcx r12, r9",
            "mov rdi, 0",
            "adox r13, rdi",
            "adcx r13, rdi",
            "mov rdx, r14",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            "mulx r9, r8, qword ptr [rip + {q0_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]",
            "adox r14, r8",
            "adcx r15, rdi",
            "adox r15, r9",
            "adcx r10, r11",
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q3_ptr}]",
            "adox r10, r8",
            "adcx r12, r9",
            "adox r12, rdi",
            "adcx r13, r11",
            "mov rdi, 0",
            "adox r13, rdi",

            // round 2
            "mov rdx, qword ptr [{a_ptr} + 16]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r15, r8",
            "adox r10, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r10, rdi",
            "adox r12, r11",
            "mulx r14, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r12, r8",
            "adox r13, r9",
            "adcx r13, rdi",
            "mov r9, 0",
            "adox r14, r9",
            "adcx r14, r9",
            "mov rdx, r15",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            "mulx r9, r8, qword ptr [rip + {q0_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]",
            "adox r15, r8",
            "adcx r10, r9",
            "adox r10, rdi",
            "adcx r12, r11",
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q3_ptr}]",
            "adox r12, r8",
            "adcx r13, r9",
            "adox r13, rdi",
            "adcx r14, r11",
            "mov rdi, 0",
            "adox r14, rdi",

            // round 3
            "mov rdx, qword ptr [{a_ptr} + 24]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r10, r8",
            "adox r12, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r12, rdi",
            "adox r13, r11",
            "mulx r15, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r13, r8",
            "adox r14, r9",
            "adcx r14, rdi",
            "mov r9, 0",
            "adox r15, r9",
            "adcx r15, r9",
            "mov rdx, r10",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            "mulx r9, r8, qword ptr [rip + {q0_ptr}]",
            "mulx r11, rdi, qword ptr [rip + {q1_ptr}]",
            "adox r10, r8",
            "adcx r12, r9",
            "adox r12, rdi",
            "adcx r13, r11",
            "mulx r9, r8, qword ptr [rip + {q2_ptr}]",
            "mulx rdx, rdi, qword ptr [rip + {q3_ptr}]",
            "adox r13, r8",
            "adcx r14, r9",
            "adox r14, rdi",
            "adcx r15, rdx",
            "mov rdi, 0",
            "adox r15, rdi",
            // reduction. We use sub/sbb

            "mov r8, r12",
            "mov rdx, {q0_neg}",
            "sub r8, rdx",
            "mov r9, r13",
            "mov rdx, {q1_neg}",
            "sbb r9, rdx",
            "mov r10, r14",
            "mov rdx, {q2_neg}",
            "sbb r10, rdx",
            "mov r11, r15",
            "mov rdx, {q3_neg}",
            "sbb r11, rdx",

            // if CF == 1 then original result was ok (reduction wa not necessary)
            // so if not carry (CMOVNQ) then we copy 
            "cmovnc r12, r8",
            "cmovnc r13, r9",
            "cmovnc r14, r10",
            "cmovnc r15, r11",  
            q0_neg = const 0xd0970e5ed6f72cb7u64,
            q1_neg = const 0xa6682093ccc81082u64,
            q2_neg = const 0x6673b0101343b00u64,
            q3_neg = const 0xe7db4ea6533afa9u64,
            // end of reduction
            q0_ptr = sym MODULUS_0,
            q1_ptr = sym MODULUS_1,
            q2_ptr = sym MODULUS_2,
            q3_ptr = sym MODULUS_3,
            inv = const 0x1ba3a358ef788ef9u64,
            a_ptr = in(reg) a.as_ptr(),
            b_ptr = in(reg) b.as_ptr(),
            out("rdx") _, 
            out("rdi") _, 
            out("r8") _, 
            out("r9") _, 
            out("r10") _, 
            out("r11") _, 
            out("r12") r0, 
            out("r13") r1, 
            out("r14") r2, 
            out("r15") r3,
            options(pure, readonly, nostack)
        );
    }

    [r0, r1, r2, r3]
}

// // assumes that 
// macro_rules! branchless_reduce_by_one_modulus {
//     ($q0_neg: literal, $q1_neg: literal, $q2_neg: literal, $q3_neg: literal) => {
        
//     };
// }


#[allow(dead_code)]
#[allow(clippy::too_many_lines)]
#[inline(always)]
#[cfg(target_arch = "x86_64")]
pub fn mont_mul_asm(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    use core::mem::MaybeUninit;

    static INV: u64 = 0x1ba3a358ef788ef9;
    static MODULUS: [u64; 4] = [0xd0970e5ed6f72cb7, 0xa6682093ccc81082, 0x6673b0101343b00, 0xe7db4ea6533afa9];

    // this is CIOS multiplication when top bit for top work of modulus is not set
    let mut result = MaybeUninit::<[u64; 4]>::uninit();
    // mulx dest_hi, dest_lo, src1  
    // use notation of order (hi, lo)

    unsafe {
        asm!(
            // round 0
            "mov rdx, qword ptr [{a_ptr} + 0]",
            "xor r8d, r8d",
            "mulx r14, r13, qword ptr [{b_ptr} + 0]", // (r14, r13) = a[0] * b[0]
            "mulx r9, r8, qword ptr [{b_ptr} + 8]", // (r9, r8) = a[0] * b[1]
            "mulx r10, r15, qword ptr [{b_ptr} + 16]", // (r10, r15) = a[0] * b[2]
            "mulx r12, rdi, qword ptr [{b_ptr} + 24]", // (r12, rdi) = a[0] * b[3]
            "mov rdx, r13", // rdx = r13 = (a[0] * b[0]).l0, r[0] in r13,
            "mulx r11, rdx, qword ptr [rip + {inv_ptr}]", // (r11, rdx) = (a[0] * b[0]).lo * k, so rdx = m (we overwrite rdx cause (a[0] * b[0]).lo is not needed for anything else)
            "add r14, r8", // t[1], set CF 
            "adc r15, r9", // t[2] + CF, set CF
            "adc r10, rdi", // t[3] + CF, set CF
            "adc r12, 0", // t[4] + CF
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]", // (r9, r8) = m * q0
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]", // (r11, rdi) = m * q1
            "add r13, r8", // r[0]
            "adc r14, rdi", // r[1]
            "adc r15, r11", // r[2]
            "adc r10, 0", // r[3]
            "add r14, r9", // continue r[1]
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]", // (r9, r8) = m * q2
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 24]", // (r11, rdi) = m * q3
            "adc r15, r8", // continue r[2]
            "adc r10, rdi", // continue r[3]
            "adc r12, r11", // r[4]
            "add r10, r9", // finish r[3]
            "adc r12, 0", // finish r[4]

            // round 1
            "mov rdx, qword ptr [{a_ptr} + 8]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "add r14, r8",
            "adc r15, rdi",
            "adc r10, r11",
            "adc r12, 0",
            "add r15, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r13, rdi, qword ptr [{b_ptr} + 24]",
            "adc r10, r8",
            "adc r12, rdi",
            "adc r13, 0",
            "add r12, r9",
            "adc r13, 0",
            "mov rdx, r14",
            "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]",
            "add r14, r8",
            "adc r15, rdi",
            "adc r10, r11",
            "adc r12, 0",
            "add r15, r9",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 24]",
            "adc r10, r8",
            "adc r12, r9",
            "adc r13, r11",
            "add r12, rdi",
            "adc r13, 0",

            // round 2
            "mov rdx, qword ptr [{a_ptr} + 16]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "add r15, r8",
            "adc r10, r9",
            "adc r12, r11",
            "adc r13, 0",
            "add r10, rdi",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r14, rdi, qword ptr [{b_ptr} + 24]",
            "adc r12, r8",
            "adc r13, r9",
            "adc r14, 0",
            "add r13, rdi",
            "adc r14, 0",
            "mov rdx, r15",
            "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]",
            "add r15, r8",
            "adc r10, r9",
            "adc r12, r11",
            "adc r13, 0",
            "add r10, rdi",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 24]",
            "adc r12, r8",
            "adc r13, r9",
            "adc r14, r11",
            "add r13, rdi",
            "adc r14, 0",

            // round 3
            "mov rdx, qword ptr [{a_ptr} + 24]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "add r10, r8",
            "adc r12, r9",
            "adc r13, r11",
            "adc r14, 0",
            "add r12, rdi",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r15, rdi, qword ptr [{b_ptr} + 24]",
            "adc r13, r8",
            "adc r14, r9",
            "adc r15, 0",
            "add r14, rdi",
            "adc r15, 0",
            "mov rdx, r10",
            "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]",
            "add r10, r8",
            "adc r12, r9",
            "adc r13, r11",
            "adc r14, 0",
            "add r12, rdi",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]",
            "mulx rdx, rdi, qword ptr [rip + {q_ptr} + 24]",
            "adc r13, r8",
            "adc r14, r9",
            "adc r15, rdx",
            "add r14, rdi",
            "adc r15, 0",
            "mov [{out_ptr} + 0], r12",
            "mov [{out_ptr} + 8], r13",
            "mov [{out_ptr} + 16], r14",
            "mov [{out_ptr} + 24], r15",
            q_ptr = sym MODULUS,
            inv_ptr = sym INV,
            out_ptr = in(reg) result.as_mut_ptr(),
            a_ptr = in(reg) a.as_ptr(),
            b_ptr = in(reg) b.as_ptr(),
            out("rdx") _, 
            out("rdi") _, 
            out("r8") _, 
            out("r9") _, 
            out("r10") _, 
            out("r11") _, 
            out("r12") _, 
            out("r13") _, 
            out("r14") _, 
            out("r15") _,
        );
    }

    let result = unsafe { result.assume_init() };

    result
}


#[allow(dead_code)]
#[allow(clippy::too_many_lines)]
#[inline(always)]
#[cfg(target_arch = "x86_64")]
pub fn mont_mul_asm_through_registers(mut a0: u64, mut a1: u64, mut a2: u64, mut a3: u64, b: &[u64; 4]) -> (u64, u64, u64, u64) {
    static INV: u64 = 0x1ba3a358ef788ef9;
    static MODULUS: [u64; 4] = [0xd0970e5ed6f72cb7, 0xa6682093ccc81082, 0x6673b0101343b00, 0xe7db4ea6533afa9];

    let mut r0: u64;
    let mut r1: u64;
    let mut r2: u64;
    let mut r3: u64;

    unsafe {
        asm!(
            // round 0
            "mov rdx, r12", // move a0 to rdx
            "xor r12d, r12d", // clear flags before we begin
            // a0 is in rdx
            "mulx r14, r13, qword ptr [{b_ptr} + 0]", // (r14, r13) = a[0] * b[0]
            "mulx r9, r8, qword ptr [{b_ptr} + 8]", // (r9, r8) = a[0] * b[1]
            "mulx r10, r15, qword ptr [{b_ptr} + 16]", // (r10, r15) = a[0] * b[2]
            "mulx r12, rdi, qword ptr [{b_ptr} + 24]", // (r12, rdi) = a[0] * b[3]
            "mov rdx, r13", // rdx = r13 = (a[0] * b[0]).l0, r[0] in r13,
            "mulx r11, rdx, qword ptr [rip + {inv_ptr}]", // (r11, rdx) = (a[0] * b[0]).lo * k, so rdx = m (we overwrite rdx cause (a[0] * b[0]).lo is not needed for anything else)
            "add r14, r8", // t[1], set CF 
            "adc r15, r9", // t[2] + CF, set CF
            "adc r10, rdi", // t[3] + CF, set CF
            "adc r12, 0", // t[4] + CF
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]", // (r9, r8) = m * q0
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]", // (r11, rdi) = m * q1
            "add r13, r8", // r[0]
            "adc r14, rdi", // r[1]
            "adc r15, r11", // r[2]
            "adc r10, 0", // r[3]
            "add r14, r9", // continue r[1]
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]", // (r9, r8) = m * q2
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 24]", // (r11, rdi) = m * q3
            "adc r15, r8", // continue r[2]
            "adc r10, rdi", // continue r[3]
            "adc r12, r11", // r[4]
            "add r10, r9", // finish r[3]
            "adc r12, 0", // finish r[4]

            // round 1
            "mov rdx, {a_1}",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "add r14, r8",
            "adc r15, rdi",
            "adc r10, r11",
            "adc r12, 0",
            "add r15, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r13, rdi, qword ptr [{b_ptr} + 24]",
            "adc r10, r8",
            "adc r12, rdi",
            "adc r13, 0",
            "add r12, r9",
            "adc r13, 0",
            "mov rdx, r14",
            "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]",
            "add r14, r8",
            "adc r15, rdi",
            "adc r10, r11",
            "adc r12, 0",
            "add r15, r9",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 24]",
            "adc r10, r8",
            "adc r12, r9",
            "adc r13, r11",
            "add r12, rdi",
            "adc r13, 0",

            // round 2
            "mov rdx, {a_2}",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "add r15, r8",
            "adc r10, r9",
            "adc r12, r11",
            "adc r13, 0",
            "add r10, rdi",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r14, rdi, qword ptr [{b_ptr} + 24]",
            "adc r12, r8",
            "adc r13, r9",
            "adc r14, 0",
            "add r13, rdi",
            "adc r14, 0",
            "mov rdx, r15",
            "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]",
            "add r15, r8",
            "adc r10, r9",
            "adc r12, r11",
            "adc r13, 0",
            "add r10, rdi",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 24]",
            "adc r12, r8",
            "adc r13, r9",
            "adc r14, r11",
            "add r13, rdi",
            "adc r14, 0",

            // round 3
            "mov rdx, {a_3}",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "add r10, r8",
            "adc r12, r9",
            "adc r13, r11",
            "adc r14, 0",
            "add r12, rdi",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r15, rdi, qword ptr [{b_ptr} + 24]",
            "adc r13, r8",
            "adc r14, r9",
            "adc r15, 0",
            "add r14, rdi",
            "adc r15, 0",
            "mov rdx, r10",
            "mulx r8, rdx, qword ptr [rip + {inv_ptr}]",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 0]",
            "mulx r11, rdi, qword ptr [rip + {q_ptr} + 8]",
            "add r10, r8",
            "adc r12, r9",
            "adc r13, r11",
            "adc r14, 0",
            "add r12, rdi",
            "mulx r9, r8, qword ptr [rip + {q_ptr} + 16]",
            "mulx rdx, rdi, qword ptr [rip + {q_ptr} + 24]",
            "adc r13, r8",
            "adc r14, r9",
            "adc r15, rdx",
            "add r14, rdi",
            "adc r15, 0",
            q_ptr = sym MODULUS,
            inv_ptr = sym INV,
            b_ptr = in(reg) b.as_ptr(),
            a_1 = in(reg) a1,
            a_2 = in(reg) a2,
            a_3 = in(reg) a2,
            inout("r12") a0 => r0,
            out("rdx") _, 
            out("rdi") _, 
            out("r8") _, 
            out("r9") _, 
            out("r10") _, 
            out("r11") _, 
            out("r13") r1, 
            out("r14") r2, 
            out("r15") r3,
            options(pure, readonly, nostack)
        );
    }

    (r0, r1, r2, r3)
}


#[allow(dead_code)]
#[allow(clippy::too_many_lines)]
#[inline(always)]
#[cfg(all(target_arch = "x86_64", target_feature = "adx"))]
pub fn mont_mul_asm_adx_for_proth_prime(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    let mut r0: u64;
    let mut r1: u64;
    let mut r2: u64;
    let mut r3: u64;

    // this is CIOS multiplication when top bit for top work of modulus is not set

    // mulx    dest_hi, dest_lo, src1  
    // use notation of order (hi, lo)

    // |     | b3  | b2  | b1  | b0  |
    // |     |     |     |     | a0  |
    // |---- |---- |---- |---- |---- |
    // |     |     |     | r14 | r13 |
    // |     |     | r9  | r8  |     |
    // |     | r10 | r15 |     |     |
    // | r12 | rdi |     |     |     |
    // |---- |---- |---- |---- |---- |
    // |     |     |     |     |     | // rdx = m, r11 = garbage
    // |     |     |  CF | r14 |     |
    // |  OF | r10 |     |     |     | 
    // |---- |---- |---- |---- |---- |
    // |     | CF  | r15 |     |     |
    // | r12 |     |     |     |     |
    // | CF  | r10 |     |     |     |
    // |---- |---- |---- |---- |---- |
    // | r12 | r10 | r15 | r14 | r13 |

    unsafe {
        asm!(
            // round 0
            "mov rdx, qword ptr [{a_ptr} + 0]",
            "xor r8d, r8d",
            "mulx r14, r13, qword ptr [{b_ptr} + 0]", // (r14, r13) = a[0] * b[0]
            "mulx r9, r8, qword ptr [{b_ptr} + 8]", // (r9, r8) = a[0] * b[1]
            "mulx r10, r15, qword ptr [{b_ptr} + 16]", // (r10, r15) = a[0] * b[2]
            "mulx r12, rdi, qword ptr [{b_ptr} + 24]", // (r12, rdi) = a[0] * b[3]
            // by this moment MULX for a[0] * b[0] is complete (latency = 4)
            "mov rdx, r13", // rdx = r13 = (a[0] * b[0]).l0
            "mov r11, {inv}",
            "mulx r11, rdx, r11", // (r11, rdx) = (a[0] * b[0]).lo * k, so rdx = m (we overwrite rdx cause (a[0] * b[0]).lo is not needed for anything else)
            "adcx r14, r8", // r14 = r14 + r8 = (a[0] * b[0]).hi + (a[0] * b[1]).lo, carry flag is set in CF register (CF = carry into 2nd word), 1st word calculation
            "adox r10, rdi", // r10 = r10 + rdi = (a[0] * b[2]).hi + (a[0] * b[3]).lo, carry flag is set in OF register (OF = carry into 4th word), 3rd word calculation 
            "adcx r15, r9", // r15 = r15 + r9 + CF = (a[0] * b[1]).hi + (a[0] * b[2]).lo + CF, 2nd word continuation
            "mov r11, 0",
            "adox r12, r11", // r12 = r12 + OF = 4th word
            "adcx r10, r11", // r10 = r10 + CF, 3rd word continuation
            "adox r13, rdx", // r13 = t[0] + (m * q0).lo, set OF
            "adcx r14, r11", // r14 = t[1] + (m * q1).lo, set CF
            "adox r14, r11", // r14 = t[1] + (m * q0).hi + OF, set OF
            "adcx r15, r11", // r15 = t[2] + (m * q1).hi + CF, set CF
            "mov r8, {q_3}",
            "mulx r9, rdi, r8", // (r11, rdi) = m * q3
            "adox r15, r11", // r15 = t[2] + (m * q2).lo + OF, set OF
            "adcx r10, rdi", // r10 = t[3] + (m * q3).lo + CF, set CF
            "adox r10, r11", // r10 = t[3] + (m * q2).hi + OF, set OF
            "adcx r12, r9", // r12 = t[4] + (m * q3).hi + CF, set CF
            "adox r12, r11", // r12 = r12 + OF

            // round 1
            "mov rdx, qword ptr [{a_ptr} + 8]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r14, r8",
            "adox r15, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r15, rdi",
            "adox r10, r11",
            // "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r13, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r10, r8",
            "adox r12, rdi",
            "adcx r12, r9",
            "mov r11, 0",
            "adox r13, r11",
            "adcx r13, r11",
            "mov rdx, r14",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            "adox r14, rdx",
            "adcx r15, r11",
            "adox r15, r11",
            "adcx r10, r11",
            "mov r8, {q_3}",
            "mulx r9, rdi, r8",
            "adox r10, r11",
            "adcx r12, r11",
            "adox r12, rdi",
            "adcx r13, r9",
            "adox r13, r11",

            // round 2
            "mov rdx, qword ptr [{a_ptr} + 16]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r15, r8",
            "adox r10, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r10, rdi",
            "adox r12, r11",
            // "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r14, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r12, r8",
            "adox r13, r9",
            "adcx r13, rdi",
            "mov r11, 0",
            "adox r14, r11",
            "adcx r14, r11",
            "mov rdx, r15",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            "adox r15, rdx",
            "adcx r10, r11",
            "adox r10, r11",
            "adcx r12, r11",
            "mov r8, {q_3}",
            "mulx r9, rdi, r8",
            "adox r12, r11",
            "adcx r13, r11",
            "adox r13, rdi",
            "adcx r14, r9",
            "adox r14, r11",

            // round 3
            "mov rdx, qword ptr [{a_ptr} + 24]",
            "mulx r9, r8, qword ptr [{b_ptr} + 0]",
            "mulx r11, rdi, qword ptr [{b_ptr} + 8]",
            "adcx r10, r8",
            "adox r12, r9",
            "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "adcx r12, rdi",
            "adox r13, r11",
            // "mulx r9, r8, qword ptr [{b_ptr} + 16]",
            "mulx r15, rdi, qword ptr [{b_ptr} + 24]",
            "adcx r13, r8",
            "adox r14, r9",
            "adcx r14, rdi",
            "mov r11, 0",
            "adox r15, r11",
            "adcx r15, r11",
            "mov rdx, r10",
            "mov r8, {inv}",
            "mulx r8, rdx, r8",
            "adox r10, rdx",
            "adcx r12, r11",
            "adox r12, r11",
            "adcx r13, r11",
            "mov r8, {q_3}",
            "mulx r9, rdi, r8",
            "adox r13, r11",
            "adcx r14, r11",
            "adox r14, rdi",
            "adcx r15, r9",
            "adox r15, r11",
            q_3 = const 0xe7db4ea6533afa9u64,
            inv = const 0xffffffffffffffffu64,
            a_ptr = in(reg) a.as_ptr(),
            b_ptr = in(reg) b.as_ptr(),
            out("rdx") _, 
            out("rdi") _, 
            out("r8") _, 
            out("r9") _, 
            out("r10") _, 
            out("r11") _, 
            out("r12") r0, 
            out("r13") r1, 
            out("r14") r2, 
            out("r15") r3,
            options(pure, nomem, nostack)
        );
    }

    [r0, r1, r2, r3]
}