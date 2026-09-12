use super::*;

#[test]
fn waiting_positions_include_paused_jobs_but_skip_running_and_ended_jobs() {
    let statuses = [
        ProjectStatus::Completed,
        ProjectStatus::Paused,
        ProjectStatus::Running,
        ProjectStatus::Queued,
        ProjectStatus::Cancelled,
        ProjectStatus::Queued,
    ];
    let mut state = ProjectState::default();
    for (index, status) in statuses.into_iter().enumerate() {
        let mut job = ProjectInstance::queued(index as u64, "restore_hull", None, 0);
        job.status = status;
        state.jobs.push(job);
    }
    assert_eq!(state.waiting_position(1), Some((1, 3)));
    assert_eq!(state.waiting_position(3), Some((2, 3)));
    assert_eq!(state.waiting_position(5), Some((3, 3)));
    for id in [0, 2, 4, 100] {
        assert_eq!(state.waiting_position(id), None);
    }
    state.jobs.swap(1, 5);
    assert_eq!(state.waiting_position(1), Some((3, 3)));
    assert_eq!(state.waiting_position(5), Some((1, 3)));
}
