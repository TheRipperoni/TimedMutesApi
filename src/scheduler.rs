use crate::tmute::resolve_timed_mutes;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start_scheduler(cron_schedule: &str) {
    let sched = JobScheduler::new().await.expect("Error scheduling job");
    let job = Job::new_async(cron_schedule, |_uuid, _l| {
        Box::pin(async move {
            resolve_timed_mutes().await;
        })
    })
    .unwrap();

    // Add async job
    sched.add(job).await.expect("Error adding profile job");

    // Start the scheduler
    sched.start().await.expect("Error starting scheduler");
}
