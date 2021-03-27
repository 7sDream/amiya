use std::{future::Future, sync::Arc};

/// Provide you custom async executor to Amiya by impl this trait.
///
/// Amiya instance will use it's [`spawn`] method to run new request handler task.
///
/// See [`Amiya::executor()`].
///
/// [`spawn`]: #method.spawn
/// [`Amiya::executor()`]: struct.Amiya.html#method.executor
// NOTE: clippy::future_not_send will be reported if Executor do not Send
pub trait Executor: Send {
    /// Spawn a new task to your executor
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static);
}

#[derive(Debug, Default)]
pub struct SmolGlobalExecutor;

impl Executor for SmolGlobalExecutor {
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) {
        smol::spawn(future).detach()
    }
}

impl Executor for smol::Executor<'_> {
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) {
        smol::Executor::spawn(self, future).detach()
    }
}

impl<Exec> Executor for Arc<Exec>
where
    Exec: Executor + Sync,
{
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) {
        self.as_ref().spawn(future)
    }
}
