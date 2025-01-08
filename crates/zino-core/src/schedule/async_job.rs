//! Scheduler for sync and async cron jobs.

use super::{AsyncScheduler, JobContext};
use crate::{datetime::DateTime, extension::TomlTableExt, BoxFuture, Uuid};
use chrono::Local;
use cron::Schedule;
use std::{io, str::FromStr, time::Duration};
use toml::Table;

/// A function pointer of the async cron job.
pub type AsyncCronJob = for<'a> fn(ctx: &'a mut JobContext) -> BoxFuture<'a>;

/// An async schedulable job.
pub struct AsyncJob {
    /// Job context.
    context: JobContext,
    /// Cron expression parser.
    schedule: Schedule,
    /// Cron job to run.
    run: AsyncCronJob,
}

impl AsyncJob {
    /// Creates a new instance.
    ///
    /// # Panics
    ///
    /// Panics if the cron expression is invalid.
    #[inline]
    pub fn new(cron_expr: &str, exec: AsyncCronJob) -> Self {
        let schedule = Schedule::from_str(cron_expr)
            .unwrap_or_else(|err| panic!("invalid cron expression `{cron_expr}`: {err}"));
        Self {
            context: JobContext::new(),
            schedule,
            run: exec,
        }
    }

    /// Creates a new instance with the configuration.
    ///
    /// # Panics
    ///
    /// Panics if the `cron` expression is invalid.
    pub fn with_config(config: &Table, exec: AsyncCronJob) -> Self {
        let cron_expr = config.get_str("cron").unwrap_or_default();
        let schedule = Schedule::from_str(cron_expr)
            .unwrap_or_else(|err| panic!("invalid cron expression `{cron_expr}`: {err}"));
        let mut context = JobContext::new();
        if let Some(disabled) = config.get_bool("disable") {
            context.set_disabled_status(disabled);
        }
        if let Some(immediate) = config.get_bool("immediate") {
            context.set_immediate_mode(immediate);
        }
        if let Some(ticks) = config
            .get_bool("once")
            .and_then(|b| b.then_some(1))
            .or_else(|| config.get_usize("max-ticks"))
        {
            context.set_remaining_ticks(ticks);
        }
        Self {
            context,
            schedule,
            run: exec,
        }
    }

    /// Sets the job name.
    #[inline]
    pub fn name(mut self, name: &'static str) -> Self {
        self.context.set_name(name);
        self
    }

    /// Sets the initial job data.
    #[inline]
    pub fn data<T: Send + 'static>(mut self, data: T) -> Self {
        self.context.set_data(data);
        self
    }

    /// Sets the number of maximum ticks.
    #[inline]
    pub fn max_ticks(mut self, ticks: usize) -> Self {
        self.context.set_remaining_ticks(ticks);
        self
    }

    /// Sets the number of maximum ticks as `1` to ensure that the job can only be executed once.
    #[inline]
    pub fn once(mut self) -> Self {
        self.context.set_remaining_ticks(1);
        self
    }

    /// Enables the flag to indicate whether the job is disabled.
    #[inline]
    pub fn disable(mut self, disabled: bool) -> Self {
        self.context.set_disabled_status(disabled);
        self
    }

    /// Enables the flag to indicate whether the job is executed immediately.
    #[inline]
    pub fn immediate(mut self, immediate: bool) -> Self {
        self.context.set_immediate_mode(immediate);
        self
    }

    /// Pauses the job by setting the `disabled` flag to `true`.
    #[inline]
    pub fn pause(&mut self) {
        self.context.set_disabled_status(true);
    }

    /// Resumes the job by setting the `disabled` flag to `false`.
    #[inline]
    pub fn resume(&mut self) {
        self.context.set_disabled_status(false);
    }

    /// Executes the missed runs asynchronously.
    pub async fn tick(&mut self) {
        let now = Local::now();
        let ctx = &mut self.context;
        let run = self.run;
        if let Some(last_tick) = ctx.last_tick().map(|dt| dt.into()) {
            for event in self.schedule.after(&last_tick) {
                if event > now || ctx.is_fused() {
                    break;
                }
                if !ctx.is_disabled() {
                    ctx.start();
                    run(ctx).await;
                    ctx.finish();
                }
            }
        } else if !ctx.is_disabled() && ctx.is_immediate() && !ctx.is_fused() {
            ctx.start();
            run(ctx).await;
            ctx.finish();
        }
        ctx.set_last_tick(now.into());
    }

    /// Executes the job manually.
    pub async fn execute(&mut self) {
        let now = DateTime::now();
        let ctx = &mut self.context;
        let run = self.run;
        ctx.start();
        run(ctx).await;
        ctx.finish();
        ctx.set_last_tick(now);
    }

    /// Returns a reference to the job context.
    #[inline]
    pub fn context(&self) -> &JobContext {
        &self.context
    }

    /// Returns a mutable reference to the job context.
    #[inline]
    pub fn context_mut(&mut self) -> &mut JobContext {
        &mut self.context
    }
}

/// A type contains and executes the async scheduled jobs.
#[derive(Default)]
pub struct AsyncJobScheduler {
    /// A list of async jobs.
    jobs: Vec<AsyncJob>,
}

impl AsyncJobScheduler {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// Adds an async job to the scheduler and returns the job ID.
    pub fn add(&mut self, job: AsyncJob) -> Uuid {
        let job_id = job.context().job_id();
        self.jobs.push(job);
        job_id
    }

    /// Removes an async job by ID from the scheduler.
    pub fn remove(&mut self, job_id: Uuid) -> bool {
        let position = self
            .jobs
            .iter()
            .position(|job| job.context().job_id() == job_id);
        if let Some(index) = position {
            self.jobs.remove(index);
            true
        } else {
            false
        }
    }

    /// Returns a reference to the job with the ID.
    #[inline]
    pub fn get(&self, job_id: Uuid) -> Option<&AsyncJob> {
        self.jobs
            .iter()
            .find(|job| job.context().job_id() == job_id)
    }

    /// Returns a mutable reference to the job with the ID.
    #[inline]
    pub fn get_mut(&mut self, job_id: Uuid) -> Option<&mut AsyncJob> {
        self.jobs
            .iter_mut()
            .find(|job| job.context().job_id() == job_id)
    }

    /// Returns the duration till the next job is supposed to run.
    pub fn time_till_next_job(&self) -> Option<Duration> {
        if self.jobs.is_empty() {
            Some(Duration::from_millis(500))
        } else {
            let mut duration = chrono::Duration::zero();
            let now = Local::now();
            for job in self.jobs.iter() {
                for event in job.schedule.after(&now).take(1) {
                    let interval = event - now;
                    if duration.is_zero() || interval < duration {
                        duration = interval;
                    }
                }
            }
            duration.to_std().ok()
        }
    }

    /// Increments time for the scheduler and executes any pending jobs asynchronously.
    /// It is recommended to sleep for at least 500 milliseconds between invocations of this method.
    #[inline]
    pub async fn tick(&mut self) {
        let mut fused_jobs = Vec::new();
        for job in &mut self.jobs {
            job.tick().await;

            let ctx = job.context();
            if ctx.is_fused() {
                fused_jobs.push(ctx.job_id());
            }
        }
        for job_id in fused_jobs {
            self.remove(job_id);
        }
    }

    /// Executes all the job manually.
    pub async fn execute(&mut self) {
        for job in &mut self.jobs {
            job.execute().await;
        }
    }
}

impl AsyncScheduler for AsyncJobScheduler {
    #[inline]
    fn is_ready(&self) -> bool {
        !self.jobs.is_empty()
    }

    #[inline]
    fn is_blocking(&self) -> bool {
        false
    }

    #[inline]
    fn time_till_next_job(&self) -> Option<Duration> {
        self.time_till_next_job()
    }

    #[inline]
    async fn tick(&mut self) {
        self.tick().await;
    }

    #[inline]
    async fn run(self) -> io::Result<()> {
        Ok(())
    }
}
