#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const IMPORTS: &str = code_str! {
    use crate::pervasive::{atomic::*};
    use crate::pervasive::{modes::*};
    use crate::pervasive::result::*;
};

/// With contradiction_smoke_test, add a final `assert(false)` that is expected to fail at the end
/// of the test, as a cheap way to check that the trusted specs aren't contradictory
fn test_body(tests: &str, contradiction_smoke_test: bool) -> String {
    IMPORTS.to_string()
        + "    fn test() {"
        + tests
        + if contradiction_smoke_test { "assert(false); // FAILS\n" } else { "" }
        + "    }"
}

const ATOMIC_U64: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicU64::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0xffff_ffff_ffff_ffff);
    assert(l == 9);
    assert(perm.value == 8);

    let l = at.fetch_sub_wrapping(&mut perm, 0xffff_ffff_ffff_ffff);
    assert(l == 8);
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assert_bit_vector(!(6 as u64 & 3 as u64) == 0xffff_ffff_ffff_fffd);
    assert(perm.value == 0xffff_ffff_ffff_fffd);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_u64_pass test_body(ATOMIC_U64, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_u64_smoke test_body(ATOMIC_U64, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_U32: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicU32::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0xffff_ffff);
    assert(l == 9);
    assert(perm.value == 8);

    let l = at.fetch_sub_wrapping(&mut perm, 0xffff_ffff);
    assert(l == 8);
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assert_bit_vector(!(6 as u32 & 3 as u32) == 0xffff_fffd);
    assert(perm.value == 0xffff_fffd);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_u32_pass test_body(ATOMIC_U32, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_u32_smoke test_body(ATOMIC_U32, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_U16: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicU16::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0xffff);
    assert(l == 9);
    assert(perm.value == 8);

    let l = at.fetch_sub_wrapping(&mut perm, 0xffff);
    assert(l == 8);
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assume(!(6 as u16 & 3 as u16) == 0xfffd);
    assert(perm.value == 0xfffd);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_u16_pass test_body(ATOMIC_U16, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_u16_smoke test_body(ATOMIC_U16, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_U8: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicU8::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0xff);
    assert(l == 9);
    assert(perm.value == 8);

    let l = at.fetch_sub_wrapping(&mut perm, 0xff);
    assert(l == 8);
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assume(!(6 as u8 & 3 as u8) == 0xfd);
    assert(perm.value == 0xfd);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_u8_pass test_body(ATOMIC_U8, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_u8_smoke test_body(ATOMIC_U8, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_I64: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicI64::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0x7fff_ffff_ffff_ffff);
    assert(l == 9);
    assert(perm.value == -(9223372036854775800 as i64));

    let l = at.fetch_sub_wrapping(&mut perm, 0x7fff_ffff_ffff_ffff);
    assert(l == -(9223372036854775800 as i64));
    assert(perm.value == 9);

    let l = at.fetch_sub_wrapping(&mut perm, -0x7fff_ffff_ffff_ffff);
    assert(l == 9);
    assert(perm.value == -(9223372036854775800 as i64));

    let l = at.fetch_add_wrapping(&mut perm, -0x7fff_ffff_ffff_ffff);
    assert(l == -(9223372036854775800 as i64));
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assume(!(6 as i64 & 3 as i64) == -3);
    assert(perm.value == -3);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_i64_pass test_body(ATOMIC_I64, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_i64_smoke test_body(ATOMIC_I64, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_I32: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicI32::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0x7fff_ffff);
    assert(l == 9);
    assert(perm.value == -2147483640);

    let l = at.fetch_sub_wrapping(&mut perm, 0x7fff_ffff);
    assert(l == -2147483640);
    assert(perm.value == 9);

    let l = at.fetch_sub_wrapping(&mut perm, -0x7fff_ffff);
    assert(l == 9);
    assert(perm.value == -2147483640);

    let l = at.fetch_add_wrapping(&mut perm, -0x7fff_ffff);
    assert(l == -2147483640);
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assume(!(6 as i32 & 3 as i32) == -3);
    assert(perm.value == -3);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_i32_pass test_body(ATOMIC_I32, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_i32_smoke test_body(ATOMIC_I32, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_I16: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicI16::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0x7fff);
    assert(l == 9);
    assert(perm.value == -32760);

    let l = at.fetch_sub_wrapping(&mut perm, 0x7fff);
    assert(l == -32760);
    assert(perm.value == 9);

    let l = at.fetch_sub_wrapping(&mut perm, -0x7fff);
    assert(l == 9);
    assert(perm.value == -32760);

    let l = at.fetch_add_wrapping(&mut perm, -0x7fff);
    assert(l == -32760);
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assume(!(6 as i16 & 3 as i16) == -3);
    assert(perm.value == -3);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_i16_pass test_body(ATOMIC_I16, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_i16_smoke test_body(ATOMIC_I16, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_I8: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicI8::new(5);

    assert(perm.value == 5);

    let l = at.load(&perm);
    assert(l == 5);

    at.store(&mut perm, 6);
    assert(perm.value == 6);

    let l = at.swap(&mut perm, 9);
    assert(l == 6);
    assert(perm.value == 9);

    let l = at.fetch_add_wrapping(&mut perm, 0x7f);
    assert(l == 9);
    assert(perm.value == -120);

    let l = at.fetch_sub_wrapping(&mut perm, 0x7f);
    assert(l == -120);
    assert(perm.value == 9);

    let l = at.fetch_sub_wrapping(&mut perm, -0x7f);
    assert(l == 9);
    assert(perm.value == -120);

    let l = at.fetch_add_wrapping(&mut perm, -0x7f);
    assert(l == -120);
    assert(perm.value == 9);

    let l = at.fetch_or(&mut perm, 2);
    assert(l == 9);
    assert_bit_vector((9 as u64 | 2 as u64) == 11 as u64);
    assert(perm.value == 11);

    let l = at.fetch_and(&mut perm, 6);
    assert(l == 11);
    assert_bit_vector((11 as u64 & 6 as u64) == 2 as u64);
    assert(perm.value == 2);

    let l = at.fetch_xor(&mut perm, 3);
    assert(l == 2);
    assert_bit_vector((2 as u64 ^ 3 as u64) == 1 as u64);
    assert(perm.value == 1);

    let l = at.fetch_max(&mut perm, 5);
    assert(l == 1);
    assert(perm.value == 5);

    let l = at.fetch_max(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 5);

    let l = at.fetch_min(&mut perm, 4);
    assert(l == 5);
    assert(perm.value == 4);

    let l = at.fetch_min(&mut perm, 7);
    assert(l == 4);
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange_weak(&mut perm, 4, 6);
    assert(
        (equal(l, Result::Err(4)) && perm.value == 4) ||
        (equal(l, Result::Ok(4)) && perm.value == 6)
    );

    at.store(&mut perm, 4);
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 5, 6);
    assert(equal(l, Result::Err(4)));
    assert(perm.value == 4);

    let l = at.compare_exchange(&mut perm, 4, 6);
    assert(equal(l, Result::Ok(4)));
    assert(perm.value == 6);

    let l = at.fetch_nand(&mut perm, 3);
    assert(l == 6);
    assume(!(6 as i8 & 3 as i8) == -3);
    assert(perm.value == -3);

    at.store(&mut perm, 6);
    let l = at.into_inner(perm);
    assert(l == 6);
};

test_verify_one_file! {
    #[test] test_atomic_i8_pass test_body(ATOMIC_I8, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_i8_smoke test_body(ATOMIC_I8, true) => Err(e) => assert_one_fails(e)
}

const ATOMIC_BOOL: &str = code_str! {
    let (at, Proof(mut perm)) = PAtomicBool::new(false);

    assert(perm.value == false);

    let l = at.load(&perm);
    assert(l == false);

    at.store(&mut perm, true);
    assert(perm.value == true);

    let l = at.swap(&mut perm, false);
    assert(l == true);
    assert(perm.value == false);

    // fetch_or

    let l = at.fetch_or(&mut perm, false);
    assert(l == false);
    assert(perm.value == false);

    let l = at.fetch_or(&mut perm, true);
    assert(l == false);
    assert(perm.value == true);

    let l = at.fetch_or(&mut perm, false);
    assert(l == true);
    assert(perm.value == true);

    let l = at.fetch_or(&mut perm, true);
    assert(l == true);
    assert(perm.value == true);

    // fetch_and

    let l = at.fetch_and(&mut perm, true);
    assert(l == true);
    assert(perm.value == true);

    let l = at.fetch_and(&mut perm, false);
    assert(l == true);
    assert(perm.value == false);

    let l = at.fetch_and(&mut perm, false);
    assert(l == false);
    assert(perm.value == false);

    let l = at.fetch_and(&mut perm, true);
    assert(l == false);
    assert(perm.value == false);

    // fetch_xor

    let l = at.fetch_xor(&mut perm, false);
    assert(l == false);
    assert(perm.value == false);

    let l = at.fetch_xor(&mut perm, true);
    assert(l == false);
    assert(perm.value == true);

    let l = at.fetch_xor(&mut perm, false);
    assert(l == true);
    assert(perm.value == true);

    let l = at.fetch_xor(&mut perm, true);
    assert(l == true);
    assert(perm.value == false);

    // fetch_nand

    let l = at.fetch_nand(&mut perm, false);
    assert(l == false);
    assert(perm.value == true);

    let l = at.fetch_nand(&mut perm, false);
    assert(l == true);
    assert(perm.value == true);

    let l = at.fetch_nand(&mut perm, true);
    assert(l == true);
    assert(perm.value == false);

    let l = at.fetch_nand(&mut perm, true);
    assert(l == false);
    assert(perm.value == true);

    // compare_exchange

    let l = at.compare_exchange_weak(&mut perm, false, false);
    assert(equal(l, Result::Err(true)));
    assert(perm.value == true);

    let l = at.compare_exchange_weak(&mut perm, true, false);
    assert(
        (equal(l, Result::Err(true)) && perm.value == true) ||
        (equal(l, Result::Ok(true)) && perm.value == false)
    );

    at.store(&mut perm, false);
    assert(perm.value == false);

    let l = at.compare_exchange(&mut perm, true, false);
    assert(equal(l, Result::Err(false)));
    assert(perm.value == false);

    let l = at.compare_exchange(&mut perm, false, true);
    assert(equal(l, Result::Ok(false)));
    assert(perm.value == true);

    let l = at.into_inner(perm);
    assert(l == true);
};

test_verify_one_file! {
    #[test] test_atomic_bool_pass test_body(ATOMIC_BOOL, false) => Ok(())
}

test_verify_one_file! {
    #[test] test_atomic_bool_smoke test_body(ATOMIC_BOOL, true) => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] test_unsigned_add_overflow_fail
    IMPORTS.to_string() + code_str! {
        pub fn do_nothing() {
            let (at, Proof(mut perm)) = PAtomicU32::new(0xf000_0000);

            at.fetch_add(&mut perm, 0xf000_0000); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test_unsigned_sub_underflow_fail
    IMPORTS.to_string() + code_str! {
        pub fn do_nothing() {
            let (at, Proof(mut perm)) = PAtomicU32::new(5);

            at.fetch_sub(&mut perm, 6); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test_signed_add_overflow_fail
    IMPORTS.to_string() + code_str! {
        pub fn do_nothing() {
            let (at, Proof(mut perm)) = PAtomicI32::new(0x7000_0000);

            at.fetch_add(&mut perm, 0x7000_0000); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test_signed_add_underflow_fail
    IMPORTS.to_string() + code_str! {
        pub fn do_nothing() {
            let (at, Proof(mut perm)) = PAtomicI32::new(-0x7000_0000);

            at.fetch_add(&mut perm, -0x7000_0000); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test_signed_sub_overflow_fail
    IMPORTS.to_string() + code_str! {
        pub fn do_nothing() {
            let (at, Proof(mut perm)) = PAtomicI32::new(0x7000_0000);

            at.fetch_sub(&mut perm, -0x7000_0000); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test_signed_sub_underflow_fail
    IMPORTS.to_string() + code_str! {
        pub fn do_nothing() {
            let (at, Proof(mut perm)) = PAtomicI32::new(-0x7000_0000);

            at.fetch_sub(&mut perm, 0x7000_0000); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}
