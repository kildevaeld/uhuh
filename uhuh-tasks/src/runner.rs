use async_trait::async_trait;
use infinitask::{InifiniTask, TaskId, TaskList};

struct RunnerDelegate;

#[async_trait(?Send)]
impl<C> infinitask::Delegate<C> for RunnerDelegate {
    async fn task_registered(&self, ctx: &C, id: TaskId, task: &dyn infinitask::Task<C>) {}
    async fn task_started(&self, ctx: &C, task: TaskId) {}
    async fn task_finished(&self, ctx: &C, task: TaskId, error: Option<infinitask::TaskError>) {}
}

pub struct Runner<C> {
    tasks: InifiniTask<C, RunnerDelegate>,
}

impl<C: Send + Sync + Clone + 'static> Runner<C> {
    pub fn new() -> Runner<C> {
        Runner {
            tasks: InifiniTask::new(RunnerDelegate),
        }
    }

    pub async fn run_many(&self, ctx: C, tasks: impl TaskList<C>) {
        self.tasks.run_many(ctx, tasks).await
    }

    pub async fn run(&self, ctx: C, tasks: impl infinitask::Task<C> + Send + 'static) {
        self.tasks.run(ctx, tasks).await
    }
}
