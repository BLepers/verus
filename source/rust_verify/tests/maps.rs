#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] test1 code! {
        use crate::pervasive::set::*;
        use crate::pervasive::map::*;

        #[proof]
        fn test_map() {
            let s1 = Set::<int>::empty().insert(1).insert(2).insert(3);
            let m1 = s1.mk_map(|k: int| 10 * k);
            assert(m1.index(2) == 20);
            let s2 = Set::<int>::empty().insert(1).insert(3).insert(2);
            let m2 = s2.mk_map(|k: int| 3 * k + 7 * k);
            assert(m1.ext_equal(m2));
        }

        #[proof]
        fn testfun_eq() {
            let s = Set::<int>::empty().insert(1).insert(2).insert(3);
            let m1 = s.mk_map(|x: int| x + 4);
            let m2 = s.mk_map(|y: int| y + (2 + 2));
            // m1 and m2 are equal even without extensional equality:
            assert(equal(m1, m2));
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] test1_fails1 code! {
        use crate::pervasive::set::*;
        use crate::pervasive::map::*;

        #[proof]
        fn test_map() {
            let s1 = Set::<int>::empty().insert(1).insert(2).insert(3);
            let m1 = s1.mk_map(|k: int| 10 * k);
            assert(m1.index(2) == 20);
            assert(m1.index(4) == 40); // FAILS
            let s2 = Set::<int>::empty().insert(1).insert(3).insert(2);
            let m2 = s2.mk_map(|k: int| 3 * k + 7 * k);
            assert(m1.ext_equal(m2));
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test1_fails2 code! {
        use crate::pervasive::set::*;
        use crate::pervasive::map::*;

        #[proof]
        fn test_map() {
            let s1 = Set::<int>::empty().insert(1).insert(2).insert(3);
            let m1 = s1.mk_map(|k: int| 10 * k);
            assert(m1.index(2) == 20);
            let s2 = Set::<int>::empty().insert(1).insert(3).insert(2);
            let m2 = s2.mk_map(|k: int| 3 * k + 7 * k);
            assert(equal(m1, m2)); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test1_fails_subtype code! {
        use crate::pervasive::set::*;
        use crate::pervasive::map::*;

        #[proof]
        fn test_map() {
            let s1 = Set::<int>::empty().insert(1).insert(2).insert(3);
            let m1 = s1.mk_map(|k: int| 10 * k);
            let m3: Map<int, int> = m1;
            let m4: Map<nat, int> = m1; // FAILS: see https://github.com/FStarLang/FStar/issues/1542
        }
    } => Err(_)
}

test_verify_one_file! {
    #[test] test1_fails_eq code! {
        use crate::pervasive::set::*;
        use crate::pervasive::map::*;

        #[proof]
        fn testfun_eq() {
            let s = Set::<int>::empty().insert(1).insert(2).insert(3);
            let m1 = s.mk_map(|x: int| x + 4);
            let m2 = s.mk_map(|y: int| (2 + 2) + y);
            // would require extensional equality:
            assert(equal(m1, m2)); // FAILS
        }
    } => Err(_)
}
