use crate::erase::ResolvedCall;
use rustc_hir::{Crate, HirId};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use rustc_span::SpanData;
use std::collections::HashMap;
use std::sync::Arc;
use vir::ast::{Expr, Mode, Pattern, Typ};

pub struct ErasureInfo {
    pub(crate) resolved_calls: Vec<(SpanData, ResolvedCall)>,
    pub(crate) resolved_exprs: Vec<(SpanData, Expr)>,
    pub(crate) resolved_pats: Vec<(SpanData, Pattern)>,
    pub(crate) external_functions: Vec<vir::ast::Fun>,
    pub(crate) ignored_functions: Vec<SpanData>,
}

type ErasureInfoRef = std::rc::Rc<std::cell::RefCell<ErasureInfo>>;

pub type Context<'tcx> = Arc<ContextX<'tcx>>;
pub struct ContextX<'tcx> {
    pub(crate) tcx: TyCtxt<'tcx>,
    pub(crate) krate: &'tcx Crate<'tcx>,
    pub(crate) erasure_info: ErasureInfoRef,
    pub(crate) autoviewed_call_typs: HashMap<HirId, Typ>,
}

#[derive(Clone)]
pub(crate) struct BodyCtxt<'tcx> {
    pub(crate) ctxt: Context<'tcx>,
    pub(crate) types: &'tcx TypeckResults<'tcx>,
    pub(crate) mode: Mode,
    pub(crate) external_body: bool,
}
