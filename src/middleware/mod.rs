use {
    crate::{Context, Result},
    async_trait::async_trait,
    futures::future::BoxFuture,
};

pub type BoxedResultFut<'x> = BoxFuture<'x, Result>;

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, ctx: Context<'_>) -> Result;
}

#[async_trait]
impl<F> Middleware for F
where
    F: Fn(Context<'_>) -> BoxedResultFut<'_> + Send + Sync,
{
    async fn handle(&self, ctx: Context<'_>) -> Result {
        (self)(ctx).await
    }
}
