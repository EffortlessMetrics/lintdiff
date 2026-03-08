use lintdiff_diagnostics::Span;

pub fn select_spans(spans: &[Span]) -> Vec<Span> {
    let prim: Vec<Span> = spans.iter().filter(|s| s.is_primary).cloned().collect();
    if !prim.is_empty() {
        prim
    } else {
        spans.to_vec()
    }
}
