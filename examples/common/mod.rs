use {
    amiya::{Context, Result},
    multitask::Executor,
};

pub fn global_executor() -> Executor {
    let ex = Executor::new();

    // Create two executor threads.
    for _ in 0..num_cpus::get().max(1) {
        let (p, u) = parking::pair();
        let ticker = ex.ticker(move || u.unpark());
        std::thread::spawn(move || loop {
            if !ticker.tick() {
                p.park();
            }
        });
    }

    ex
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
