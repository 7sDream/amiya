use amiya::{Context, Result};

#[allow(dead_code)]
pub async fn response<T, D: AsRef<str>>(data: D, mut ctx: Context<'_, T>) -> Result
where
    T: Send + Sync + 'static,
{
    ctx.next().await?;
    ctx.resp.set_body(data.as_ref());
    Ok(())
}
