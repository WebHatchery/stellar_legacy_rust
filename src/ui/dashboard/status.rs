pub(super) fn custodian_status(empathy: f32) -> String {
    let empathy = empathy.clamp(0.0, 1.0);
    let disposition = crate::state::sim::custodian_disposition(empathy);
    format!("{disposition} · EMPATHY {:.0}%", empathy * 100.0)
}
