#![feature(concat_idents)]
#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

// Run each test for both Invariant/open_invariant and LocalInvariant/open_local_invariant

macro_rules! test_both {
    ($name:ident $name2:ident $test:expr => $p:pat) => {
        test_verify_one_file! {
            #[test] $name $test => $p
        }

        test_verify_one_file! {
            #[test] $name2 ($test
                .replace("Invariant", "LocalInvariant")
                .replace("open_invariant", "open_local_invariant")) => $p
        }
    };
    ($name:ident $name2:ident $test:expr => $p:pat => $e:expr) => {
        test_verify_one_file! {
            #[test] $name $test => $p => $e
        }

        test_verify_one_file! {
            #[test] $name2 ($test
                .replace("Invariant", "LocalInvariant")
                .replace("open_invariant", "open_local_invariant")) => $p => $e
        }
    };
}

test_both! {
    basic_usage basic_usage_local code! {
        use crate::pervasive::invariants::*;

        pub fn X(#[proof] i: Invariant<u8>) {
            requires([
                i.inv(0)
            ]);
            open_invariant!(&i => inner => {
                #[proof] let x = 5;
                #[proof] let x = 6;
                inner = 0;
            });
        }
    } => Ok(())
}

test_both! {
    basic_usage2 basic_usage2_local code! {
        use crate::pervasive::invariants::*;

        pub fn X(#[proof] i: Invariant<u8>) {
            open_invariant!(&i => inner => {
            });
        }
    } => Ok(())
}

test_both! {
    inv_fail inv_fail_local code! {
        use crate::pervasive::invariants::*;
        pub fn X(#[proof] i: Invariant<u8>) {
            open_invariant!(&i => inner => {
                #[proof] let x = 5;
                #[proof] let x = 6;
                inner = 0;
            }); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_both! {
    nested_failure nested_failure_local code! {
        use crate::pervasive::invariants::*;
        pub fn nested(#[proof] i: Invariant<u8>) {
            requires([
                i.inv(0)
            ]);
            open_invariant!(&i => inner => { // FAILS
                open_invariant!(&i => inner2 => {
                    inner2 = 0;
                });
                inner = 0;
            });
        }
    } => Err(err) => assert_one_fails(err)
}

test_both! {
    nested_good nested_good_local code! {
        use crate::pervasive::invariants::*;
        pub fn nested_good(#[proof] i: Invariant<u8>, #[proof] j: Invariant<u8>) {
            requires([
                i.inv(0),
                j.inv(1),
                i.namespace() == 0,
                j.namespace() == 1,
            ]);
            open_invariant!(&i => inner => {
                inner = 0;
                open_invariant!(&j => inner => {
                    inner = 1;
                });
            });
        }
    } => Ok(())
}

test_both! {
    full_call_empty full_call_empty_local code! {
        use crate::pervasive::invariants::*;
        #[proof]
        pub fn callee_mask_empty() {
          opens_invariants_none(); // will not open any invariant
        }
        pub fn t1(#[proof] i: Invariant<u8>) {
          open_invariant!(&i => inner => {
            callee_mask_empty();
          });
        }
    } => Ok(())
}

test_both! {
    open_call_full open_call_full_local code! {
        use crate::pervasive::invariants::*;
        #[proof]
        pub fn callee_mask_full() {
          opens_invariants_any(); // can open any invariant
        }
        pub fn t2(#[proof] i: Invariant<u8>) {
          open_invariant!(&i => inner => { // FAILS
            callee_mask_full();
          });
        }
    } => Err(err) => assert_one_fails(err)
}

test_both! {
    empty_open empty_open_local code! {
        use crate::pervasive::invariants::*;
        #[proof]
        pub fn callee_mask_empty() {
          opens_invariants_none(); // will not open any invariant
        }
        pub fn t3(#[proof] i: Invariant<u8>) {
          opens_invariants_none();
          open_invariant!(&i => inner => { // FAILS
          });
        }
    } => Err(err) => assert_one_fails(err)
}

// mode stuff

test_both! {
    open_inv_in_spec open_inv_in_spec_local code! {
        use crate::pervasive::invariants::*;

        #[spec]
        pub fn open_inv_in_spec(i: Invariant<u8>) {
          open_invariant!(&i => inner => {
          });
        }
    } => Err(err) => assert_vir_error(err)
}

test_both! {
    inv_header_in_spec inv_header_in_spec_local code! {
        use crate::pervasive::invariants::*;

        #[spec]
        pub fn inv_header_in_spec(i: Invariant<u8>) {
          opens_invariants_any();
        }
    } => Err(err) => assert_vir_error(err)
}

test_both! {
    open_inv_in_proof open_inv_in_proof_local code! {
        use crate::pervasive::invariants::*;

        #[proof]
        pub fn open_inv_in_proof(#[proof] i: Invariant<u8>) {
          opens_invariants_any();
          open_invariant!(&i => inner => {
          });
        }
    } => Ok(())
}

test_both! {
    inv_cannot_be_exec inv_cannot_be_exec_local code! {
        use crate::pervasive::invariants::*;

        pub fn X(#[exec] i: Invariant<u8>) {
            open_invariant!(&i => inner => {
            });
        }

    } => Err(err) => assert_vir_error(err)
}

test_both! {
    inv_cannot_be_spec inv_cannot_be_spec_local code! {
        use crate::pervasive::invariants::*;

        pub fn X(#[spec] i: Invariant<u8>) {
            open_invariant!(&i => inner => {
            });
        }

    } => Err(err) => assert_vir_error(err)
}

// This test doesn't apply to LocalInvariant
test_verify_one_file! {
    #[test] exec_code_in_inv_block code! {
        use crate::pervasive::invariants::*;

        pub fn exec_fn() { }

        pub fn X(#[proof] i: Invariant<u8>) {
            open_invariant!(&i => inner => {
                exec_fn();
            });
        }
    } => Err(err) => assert_vir_error(err)
}

test_both! {
    inv_lifetime inv_lifetime_local code! {
        use crate::pervasive::invariants::*;

        #[proof]
        fn throw_away(#[proof] i: Invariant<u8>) {
        }

        pub fn do_nothing(#[proof] i: Invariant<u8>) {
          requires([
            i.inv(0)
          ]);
          open_invariant!(&i => inner => {
            throw_away(i);
          });
        }
    } => Err(_) => ()
}

test_both! {
    return_early return_early_local code! {
        use crate::pervasive::invariants::*;

        pub fn blah(#[proof] i: Invariant<u8>) {
          open_invariant!(&i => inner => {
            return;
          });
        }
    } => Err(err) => assert_vir_error(err)
}

test_both! {
    return_early_nested return_early_nested_local code! {
        use crate::pervasive::invariants::*;

        pub fn blah(#[proof] i: Invariant<u8>, #[proof] j: Invariant<u8>) {
          open_invariant!(&i => inner => {
            open_invariant!(&j => inner => {
              return;
            });
          });
        }
    } => Err(err) => assert_vir_error(err)
}

test_both! {
    break_early break_early_local code! {
        use crate::pervasive::invariants::*;

        pub fn blah(#[proof] i: Invariant<u8>) {
          let mut idx = 0;
          while idx < 5 {
            open_invariant!(&i => inner => {
              break;
            });
          }
        }

    } => Err(err) => assert_vir_error(err)
}

test_both! {
    continue_early continue_early_local code! {
        use crate::pervasive::invariants::*;

        pub fn blah(#[proof] i: Invariant<u8>) {
          let mut idx = 0;
          while idx < 5 {
            open_invariant!(&i => inner => {
              break;
            });
          }
        }

    } => Err(err) => assert_vir_error(err)
}

test_both! {
    return_early_proof return_early_proof_local code! {
        use crate::pervasive::invariants::*;

        #[proof]
        pub fn blah(#[proof] i: Invariant<u8>) {
          open_invariant!(&i => inner => {
            return;
          });
        }
    } => Err(err) => assert_vir_error(err)
}

test_both! {
    break_early_proof break_early_proof_local code! {
        use crate::pervasive::invariants::*;

        #[proof]
        pub fn blah(#[proof] i: Invariant<u8>) {
          let mut idx = 0;
          while idx < 5 {
            open_invariant!(&i => inner => {
              break;
            });
          }
        }

    } => Err(err) => assert_vir_error(err)
}

test_both! {
    continue_early_proof continue_early_proof_local code! {
        use crate::pervasive::invariants::*;

        #[proof]
        pub fn blah(#[proof] i: Invariant<u8>) {
          let mut idx = 0;
          while idx < 5 {
            open_invariant!(&i => inner => {
              break;
            });
          }
        }

    } => Err(err) => assert_vir_error(err)
}

// Check that we can't open a normal Invariant with open_local_invariant and vice-versa

test_verify_one_file! {
    #[test] mixup1 code! {
        use crate::pervasive::invariants::*;

        pub fn X(#[proof] i: LocalInvariant<u8>) {
            open_invariant!(&i => inner => {
            });
        }
    } => Err(err)
}

test_verify_one_file! {
    #[test] mixup2 code! {
        use crate::pervasive::invariants::*;

        pub fn X(#[proof] i: Invariant<u8>) {
            open_local_invariant!(&i => inner => {
            });
        }
    } => Err(err)
}
