use {
    amiya::{Context, Result},
    futures::future,
    smol,
};

pub fn start_smol_workers() {
    for _ in 0..num_cpus::get().max(1) {
        std::thread::spawn(|| smol::run(future::pending::<()>()));
    }
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
