use crate::add_tmp_vars::add_tmp_vars_special_ops;
use crate::ast::{SpecialOp, TransitionStmt, SM};
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use syn::{Expr, Ident};

/// Simplify out `update' statements, including `add_element` etc.
///
/// Note: for 'readonly' stuff, there's less to do because we don't need to handle
/// updates. However, we still need to handle 'guard' and 'have' statements, which will
/// be translated into 'asserts'.

// Implementation:
// We proceed in two passes (although we can skip the first pass for readonly transitions,
// since this first pass only has to do with updates).
//
// Our goal here is basically to remove all the 'update' statements and replace them
// with statements of the form PostCondition(post.field == expr).
//
// The first pass, then, is to determine where the `PostCondition`
// statements should go, each with a dummy placeholder expression.
// This is handled by `add_placeholders`.
//
// In the second pass, `simplify_ops_rec`, we fill in the expressions.
// This is where the meat of the translation is, and where we apply the operational
// definitions of the various special ops.
//
// It's easiest to discuss the second pass first. It works as follows:
// for a field `foo`, we're going to initialize a "temporary variable" to `self.foo`
// at the beginning of the transition. We then symbolically step through the transition
// performing update and other op statements to update the temporary variables, e.g.,
//
//        update(foo, e)        means       temp_foo := e
//        add_element(foo, e)   means       temp_foo := temp_foo + {e}
//        ... and so on
//
// When we reach the PostCondition for `foo`, we then then add
// the postcondition `self.foo == temp_foo` for whatever accumulated `temp_foo` we have.
//
// (As we perform this process, we also remove the update and special op statements from the
// AST, possibly introducing some 'require' or 'assert' statements when necessary,
// depending on the semantics of the given special op. Again, if it's a read-only transition,
// this last part is the only part that actually does anything.)
//
// Thus for the first phase, the key is that the PostCondition statement has to go some
// place where temp_foo has taken on its final value (i.e., the PostCondition can't come
// before any statement which might update the value). Granted, one option would be to
// always put them at the end (in which case one might ask why we bother).
//
// The reason is because of conditionals. Consider,
//
//      if cond {
//          update(foo, x);
//      } else {
//          update(foo, y);
//      }
//
// One option would be to generate a relation that looks like,
//      `post.foo == (if cond { x } else { y })`
// But for better user experience, we'd ideally want one predicate per 'update' statement,
// since more fine-grained predicates make it easier to diagnose errors and each predicate
// could then be associated with the source line of a single 'update' statement.
// So we would place the PostCondition statements like this:
//
//      if cond {
//          update(foo, x);
//          PostCondition(post.foo == x);
//      } else {
//          update(foo, y);
//          PostCondition(post.foo == y);
//      }
//
// Which then generates relations like:
//
//      `cond ==> post.foo == x`
//      `!cond ==> post.foo == y`
//
// Thus the purpose of the first phase is to find these ideal positions for the
// PostCondition statements and mark those positions with placeholders.

pub fn simplify_ops(sm: &SM, ts: &TransitionStmt, is_readonly: bool) -> TransitionStmt {
    let ts = add_tmp_vars_special_ops(ts);
    let ts = if !is_readonly { add_placeholders(sm, &ts) } else { ts };

    let field_map = FieldMap::new(sm);
    let (ts, _field_map) = simplify_ops_rec(&ts, field_map);

    ts
}

// Phase 1. Adding the placeholders for the PostCondition operations.
//
// The key correctness criteria are:
//
//    1. A placeholder for a field `foo` cannot come before a statement that updates `foo`
//    2. Every control-flow path must encounter exactly one PostCondition statement
//
// Other than that, we want the PostCondition to come as soon as possible.
// So we basically just walk the tree backwards, keeping track of which fields we have
// made placements for. When we encounter the first update statement (the first from the end,
// that is) we add the PostCondition.
//
// For each conditional, we have to check if we added a statement on one branch but not
// the other, and if so, resolve.
// Finally at the very end, we add placeholders for any fields that were never updated.

fn add_placeholders(sm: &SM, ts: &TransitionStmt) -> TransitionStmt {
    let mut ts = ts.clone();

    let mut found = Vec::new();
    add_placeholders_rec(&mut ts, &mut found);

    for field in &sm.fields {
        if !contains_ident(&found, &field.name) {
            let fs = placeholder_stmt(ts.get_span().clone(), field.name.clone());
            append_stmt(&mut ts, fs);
        }
    }

    ts
}

fn add_placeholders_rec(ts: &mut TransitionStmt, found: &mut Vec<Ident>) {
    // First check if this statement is any kind of update-ish statement
    // (that includes 'update' statements, 'init' statements, and any special
    // ops that might modify the field).

    let mut is_update_for = None;
    match &ts {
        TransitionStmt::Block(..) => {}
        TransitionStmt::Let(..) => {}
        TransitionStmt::If(..) => {}
        TransitionStmt::Require(..) => {}
        TransitionStmt::Assert(..) => {}

        TransitionStmt::Initialize(_, f, _) | TransitionStmt::Update(_, f, _) => {
            is_update_for = Some(f.clone());
        }
        TransitionStmt::Special(_, f, op, _) => {
            if op.is_modifier() {
                is_update_for = Some(f.clone());
            }
        }
        TransitionStmt::PostCondition(..) => {
            panic!("PostCondition statement shouldn't exist here");
        }
    }

    match is_update_for {
        Some(f) => {
            if !contains_ident(found, &f) {
                // If it _is_ an update-ish statement, AND we haven't added
                // a placeholder for this field yet, then add a placeholder
                // immediately after the current statement. (And leave the
                // current statement unchanged).

                found.push(f.clone());
                append_stmt(ts, placeholder_stmt(*ts.get_span(), f));
                return;
            }
        }
        None => {}
    }

    // All the other cases. For any other kind of leaf statement, there's nothing
    // else to do. For blocks and branches, we recurse.

    match ts {
        TransitionStmt::Block(_, v) => {
            for t in v.iter_mut().rev() {
                add_placeholders_rec(t, found);
            }
        }
        TransitionStmt::Let(_, _, _, _, child) => {
            add_placeholders_rec(child, found);
        }
        TransitionStmt::If(span, _, e1, e2) => {
            let mut found2 = found.clone();
            let idx = found.len();

            add_placeholders_rec(e1, found);
            add_placeholders_rec(e2, &mut found2);

            // For each side of the conditional, look at any newly-found
            // fields from that conditional (those after `idx`, the original
            // length of the array). For such field, if it wasn't ALSO found
            // in the other branch, then we go ahead and add it to the other
            // branch now. Thus we maintain that, for each field and for each
            // conditional, we will either get a placeholder on both branches,
            // or on neither.

            // Make sure we end with `found` (the &mut argument) containing the
            // union of all the fields that were found on either branch.

            for i in idx..found.len() {
                if !contains_ident(&found2, &found[i]) {
                    append_stmt(e2, placeholder_stmt(*span, found[i].clone()));
                }
            }

            for i in idx..found2.len() {
                if !contains_ident(found, &found2[i]) {
                    found.push(found2[i].clone());
                    append_stmt(e1, placeholder_stmt(*span, found2[i].clone()));
                }
            }
        }

        TransitionStmt::Require(_, _) => {}
        TransitionStmt::Assert(..) => {}
        TransitionStmt::Initialize(_, _, _) => {}
        TransitionStmt::Update(_, _, _) => {}
        TransitionStmt::Special(..) => {}
        TransitionStmt::PostCondition(..) => {
            // We're in the process of adding these; they shouldn't be in here already!
            panic!("PostCondition statement shouldn't exist here");
        }
    }
}

// 'Placeholder' for the PostCondition statement
// We store the field name in order to track which field the placeholder is for.
// we will update the expression later in phase 2.

fn placeholder_stmt(span: Span, f: Ident) -> TransitionStmt {
    TransitionStmt::PostCondition(span, Expr::Verbatim(quote! { #f }))
}

fn get_field_for_placeholder(e: &Expr) -> String {
    match e {
        Expr::Verbatim(stream) => stream.to_string(),
        _ => panic!("get_field_for_placeholder found invalid placeholder"),
    }
}

fn contains_ident(v: &Vec<Ident>, id: &Ident) -> bool {
    for id0 in v {
        if id0.to_string() == id.to_string() {
            return true;
        }
    }
    return false;
}

/// Sequences t1 and t2, mutating *t1 to store the result.

fn append_stmt(t1: &mut TransitionStmt, t2: TransitionStmt) {
    match t1 {
        TransitionStmt::Block(_span, v) => {
            return v.push(t2);
        }
        _ => {}
    }
    *t1 = TransitionStmt::Block(t1.get_span().clone(), vec![t1.clone(), t2]);
}

// Phase 2. Primary logic of the translation
//
// The `field_map` we pass around contains the "temporary" variables as we
// described above.
//
// This phase gives meaning to all the special op statements by:
//
//   1. updating the `field_map` as necessary
//   2. translating to `require` and `assert` statements as necessary
//
// See `docs/command-reference.md` for the command reference and rationale
// for their definitions.
//
// TODO this is kind of jank and doesn't support all cases right now, and it's also
// somewhat difficult due to the reality of manipulating opaque Rust Exprs which could cause
// problems if they are moved into or out of a let-scope that changes the results of path
// lookups. (Currently, this issue is prevented because we introduce tmp_* variables for
// the expressions in SpecialOps.)
// This would be much easier with VIR support (e.g., if we could have 'mut' local
// variables in spec expressions, it would be a lot easier to represent the
// update definitions).

#[derive(Clone)]
struct FieldMap {
    // Each entry has a counter to track when the expression changed
    pub field_map: HashMap<String, (u64, Expr)>,
}

impl FieldMap {
    pub fn new(sm: &SM) -> FieldMap {
        let mut field_map = HashMap::new();
        for field in &sm.fields {
            let ident = &field.name;
            field_map.insert(ident.to_string(), (0, Expr::Verbatim(quote! { self.#ident })));
        }
        FieldMap { field_map }
    }

    pub fn get<'a>(&'a self, s: &String) -> &'a Expr {
        match self.field_map.get(s).as_ref() {
            Some((_, e)) => e,
            None => panic!("simplification failed, perhaps a let-variable went out-of-scope?"),
        }
    }

    pub fn set(&mut self, s: String, e: Expr) {
        let counter = self.field_map[&s].0;
        self.field_map.insert(s, (counter + 1, e));
    }

    pub fn remove_changed(old: FieldMap, new: FieldMap) -> FieldMap {
        let mut res = HashMap::new();
        for (field, (old_counter, old_e)) in old.field_map.iter() {
            match new.field_map.get(field) {
                Some((new_counter, _new_e)) => {
                    if old_counter == new_counter {
                        res.insert(field.clone(), (*old_counter, old_e.clone()));
                    }
                }
                None => {}
            }
        }
        FieldMap { field_map: res }
    }

    /// Merge two value maps at the end of a conditional.
    pub fn merge(old: FieldMap, new1: FieldMap, new2: FieldMap) -> FieldMap {
        let mut merged = HashMap::new();
        for (field, (old_counter, old_e)) in old.field_map.iter() {
            match (new1.field_map.get(field), new2.field_map.get(field)) {
                (Some((new1_counter, _new1_e)), Some((new2_counter, _new2_e))) => {
                    if new1_counter == old_counter && new2_counter == old_counter {
                        // Case: The expression wasn't changed in either branch.
                        merged.insert(field.clone(), (*old_counter, old_e.clone()));
                    } else {
                        // Case: The expression was changed in some branch.
                        // So, technically, we should construct something
                        // like `if cond { e1 } else { e2 }` here.
                        //
                        // Due to current constraints, it happens that
                        // if a field is updated inside an 'if' statement, then
                        // our temp var should never be accessed again after this point.
                        // (Special ops are forbidden in conditionals for unrelated reasons,
                        // and we only allow one 'update' per field.)
                        // So, we just leave it out of the newly constructed map.
                        //
                        // If/when this assumption turns out to not be right, then
                        // we should get a 'panic' when we try to access it later.
                    }
                }
                _ => {}
            }
        }
        FieldMap { field_map: merged }
    }
}

fn simplify_ops_rec(ts: &TransitionStmt, field_map: FieldMap) -> (TransitionStmt, FieldMap) {
    match ts {
        TransitionStmt::PostCondition(span, placeholder_e) => {
            // We found a placeholder PostCondition.
            // Update its expression.

            let f_string = get_field_for_placeholder(placeholder_e);
            let e = &field_map.get(&f_string);
            let f = Ident::new(&f_string, *span);
            let ts = TransitionStmt::PostCondition(
                *span,
                Expr::Verbatim(quote! {
                    ::builtin::equal(post.#f, #e)
                }),
            );
            return (ts, field_map);
        }
        _ => {}
    }

    match ts {
        TransitionStmt::Block(span, v) => {
            let mut field_map = field_map;
            let mut res = Vec::new();
            for t in v {
                let (t, fm) = simplify_ops_rec(t, field_map);
                field_map = fm;
                res.push(t);
            }
            (TransitionStmt::Block(*span, res), field_map)
        }
        TransitionStmt::Let(span, id, lk, e, child) => {
            let (new_child, new_map) = simplify_ops_rec(child, field_map.clone());
            // We call `remove_changed` to remove any field that has been modified
            // inside this block. We do this because the new expression could possibly
            // refer to the bound variable here which is about to go out-of-scope.
            (
                TransitionStmt::Let(*span, id.clone(), lk.clone(), e.clone(), Box::new(new_child)),
                FieldMap::remove_changed(field_map, new_map),
            )
        }
        TransitionStmt::If(span, cond, e1, e2) => {
            let (new_e1, field_map1) = simplify_ops_rec(e1, field_map.clone());
            let (new_e2, field_map2) = simplify_ops_rec(e2, field_map.clone());
            (
                TransitionStmt::If(*span, cond.clone(), Box::new(new_e1), Box::new(new_e2)),
                FieldMap::merge(field_map, field_map1, field_map2),
            )
        }
        TransitionStmt::Require(..) => (ts.clone(), field_map),
        TransitionStmt::Assert(..) => (ts.clone(), field_map),

        TransitionStmt::Initialize(span, f, e) | TransitionStmt::Update(span, f, e) => {
            let mut field_map = field_map;
            field_map.set(f.to_string(), e.clone());
            (TransitionStmt::Block(*span, Vec::new()), field_map)
        }

        TransitionStmt::Special(span, f, SpecialOp::HaveSome(e), _) => {
            let cur = field_map.get(&f.to_string());
            let prec = Expr::Verbatim(quote! {
                ::builtin::equal(
                    #cur,
                    crate::pervasive::option::Option::Some(#e)
                )
            });
            (TransitionStmt::Require(*span, prec), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::AddSome(e), proof) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    crate::pervasive::option::Option::Some(#e)
                }),
            );
            let safety = Expr::Verbatim(quote! {
                (#cur).is_None()
            });
            (TransitionStmt::Assert(*span, safety, proof.clone()), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::RemoveSome(e), _) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    crate::pervasive::option::Option::None
                }),
            );
            let prec = Expr::Verbatim(quote! {
                ::builtin::equal(
                    #cur,
                    crate::pervasive::option::Option::Some(#e)
                )
            });
            (TransitionStmt::Require(*span, prec), field_map)
        }

        TransitionStmt::Special(span, f, SpecialOp::GuardSome(e), proof) => {
            let cur = field_map.get(&f.to_string());
            let prec = Expr::Verbatim(quote! {
                ::builtin::equal(
                    #cur,
                    crate::pervasive::option::Option::Some(#e)
                )
            });
            (TransitionStmt::Assert(*span, prec, proof.clone()), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::DepositSome(e), proof) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    crate::pervasive::option::Option::Some(#e)
                }),
            );
            let safety = Expr::Verbatim(quote! {
                (#cur).is_None()
            });
            (TransitionStmt::Assert(*span, safety, proof.clone()), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::WithdrawSome(e), proof) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    crate::pervasive::option::Option::None
                }),
            );
            let prec = Expr::Verbatim(quote! {
                ::builtin::equal(
                    #cur,
                    crate::pervasive::option::Option::Some(#e)
                )
            });
            (TransitionStmt::Assert(*span, prec, proof.clone()), field_map)
        }

        TransitionStmt::Special(span, f, SpecialOp::HaveKV(key, val), _) => {
            let cur = field_map.get(&f.to_string());
            let prec = Expr::Verbatim(quote! {
                (#cur).contains_pair(#key, #val)
            });
            (TransitionStmt::Require(*span, prec), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::AddKV(key, val), proof) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    (#cur).insert(#key, #val)
                }),
            );
            let safety = Expr::Verbatim(quote! {
                !(#cur).dom().contains(#key)
            });
            (TransitionStmt::Assert(*span, safety, proof.clone()), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::RemoveKV(key, val), _) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    (#cur).remove(#key)
                }),
            );
            let prec = Expr::Verbatim(quote! {
                (#cur).contains_pair(#key, #val)
            });
            (TransitionStmt::Require(*span, prec), field_map)
        }

        TransitionStmt::Special(span, f, SpecialOp::GuardKV(key, val), proof) => {
            let cur = field_map.get(&f.to_string());
            let prec = Expr::Verbatim(quote! {
                (#cur).contains_pair(#key, #val)
            });
            (TransitionStmt::Assert(*span, prec, proof.clone()), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::DepositKV(key, val), proof) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    (#cur).insert(#key, #val)
                }),
            );
            let safety = Expr::Verbatim(quote! {
                !(#cur).dom().contains(#key)
            });
            (TransitionStmt::Assert(*span, safety, proof.clone()), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::WithdrawKV(key, val), proof) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    (#cur).remove(#key)
                }),
            );
            let prec = Expr::Verbatim(quote! {
                (#cur).contains_pair(#key, #val)
            });
            (TransitionStmt::Assert(*span, prec, proof.clone()), field_map)
        }

        TransitionStmt::Special(span, f, SpecialOp::HaveElement(e), _) => {
            let cur = field_map.get(&f.to_string());
            let prec = Expr::Verbatim(quote! {
                (#cur).count(#e) >= 1
            });
            (TransitionStmt::Require(*span, prec), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::AddElement(e), _) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    (#cur).insert(#e)
                }),
            );
            (TransitionStmt::Block(*span, Vec::new()), field_map)
        }
        TransitionStmt::Special(span, f, SpecialOp::RemoveElement(e), _) => {
            let mut field_map = field_map;
            let cur = field_map.get(&f.to_string()).clone();
            field_map.set(
                f.to_string(),
                Expr::Verbatim(quote! {
                    (#cur).remove(#e)
                }),
            );
            let prec = Expr::Verbatim(quote! {
                (#cur).count(#e) >= 1
            });
            (TransitionStmt::Require(*span, prec), field_map)
        }

        TransitionStmt::PostCondition(..) => {
            panic!("PostCondition statement shouldn't exist here");
        }
    }
}
