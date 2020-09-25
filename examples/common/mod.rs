use {
    amiya::{Context, Result},
    once_cell::sync::Lazy,
    smol::Executor,
    std::future::Future,
};

pub static EXECUTOR: Lazy<Executor> = Lazy::new(|| {
    let ex = Executor::new();
    for n in 1..=num_cpus::get() {
        std::thread::Builder::new()
            .name(format!("amiya-test-smol-executor-{}", n))
            .spawn(|| loop {
                std::panic::catch_unwind(|| {
                    smol::block_on(EXECUTOR.run(smol::future::pending::<()>()))
                })
                .ok();
            })
            .expect("cannot spawn executor thread");
    }
    return ex;
});

pub struct GlobalMultiThreadExecutor;

impl amiya::Executor for GlobalMultiThreadExecutor {
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) -> () {
        EXECUTOR.spawn(future).detach()
    }
}

#[allow(dead_code)]
pub fn run<T>(future: impl Future<Output = T>) -> T {
    smol::future::block_on(EXECUTOR.run(future))
}

#[allow(dead_code)]
pub async fn response<T, D: AsRef<str>>(data: D, mut ctx: Context<'_, T>) -> Result
where
    T: Send + Sync + 'static,
{
    ctx.next().await?;
    ctx.resp.set_body(data.as_ref());
    Ok(())
}
