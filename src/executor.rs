use std::future::Future;

#[cfg(feature = "built-in-executor")]
use {async_executor::Executor as AsyncExecutor, async_io::block_on, once_cell::sync::Lazy};

/// Provide you custom async executor to Amiya by impl this trait.
///
/// Amiya instance will use it's [`block_on`] method when listen socket and use [`spawn`] method to
/// start new request handler task.
///
/// See [`Amiya::executor()`].
///
/// [`spawn`]: #method.spawn
//. [`block_on`]: #method.block_on
/// [`Amiya::executor()`]: struct.Amiya.html#method.executor
pub trait Executor: Send + Sync {
    /// Spawn a new task to your executor, let it run in background.
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static);

    /// Run a future until complete and returns it's result.
    fn block_on<T>(&self, future: impl Future<Output = T>) -> T;
}

#[cfg(feature = "built-in-executor")]
static BUILTIN_EXECUTOR: Lazy<AsyncExecutor<'_>> = Lazy::new(|| {
    let ex = AsyncExecutor::new();
    for n in 1..=num_cpus::get() {
        std::thread::Builder::new()
            .name(format!("amiya-builtin-executor-{}", n))
            .spawn(|| loop {
                std::panic::catch_unwind(|| {
                    async_io::block_on(BUILTIN_EXECUTOR.run(std::future::pending::<()>()))
                })
                .ok();
            })
            .expect("cannot spawn executor thread");
    }
    ex
});

/// Amiya built-in multi-thread async executor.
///
/// All instances of this type share one static executor under the hood.
/// The inner executor starts `N` threads to run async task, `N` is count of your cpu cores.
///
/// In most case, you do not used this type directly, all created Amiya server have a instance of
/// it by default.
///
/// ## Notice
///
/// If you disable the `built-in-executor` default feature, you need to call [`Amiya::executor()`]
/// with your custom executor. Otherwise `Amiya::listen()` will not compile.
///
/// [`Amiya::executor()`]: struct.Amiya.html#method.executor
#[derive(Debug, Default, Clone)]
pub struct BuiltInExecutor;

#[cfg(feature = "built-in-executor")]
impl Executor for BuiltInExecutor {
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) {
        BUILTIN_EXECUTOR.spawn(future).detach()
    }

    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        block_on(BUILTIN_EXECUTOR.run(future))
    }
}
