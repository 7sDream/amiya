mod router;

use {
    crate::{Context, Result},
    async_trait::async_trait,
    std::{future::Future, pin::Pin},
};

pub use router::{MethodRouter, Router};

pub type BoxedResultFut<'x> = Pin<Box<dyn Future<Output = Result> + Send + 'x>>;

#[async_trait]
pub trait Middleware<Ex>: Send + Sync {
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result;
}

#[async_trait]
impl<Ex, F> Middleware<Ex> for F
where
    Ex: Send + Sync + 'static,
    F: Fn(Context<'_, Ex>) -> BoxedResultFut<'_> + Send + Sync,
{
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result {
        (self)(ctx).await
    }
}

#[allow(missing_debug_implementations)]
pub struct Custom<Ex> {
    pub func: Box<dyn Fn(Context<'_, Ex>) -> BoxedResultFut<'_> + Send + Sync>,
}

#[async_trait]
impl<Ex> Middleware<Ex> for Custom<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result<()> {
        (self.func)(ctx).await
    }
}

#[macro_export]
macro_rules! m {
    // Convert a async function to middleware by func's name
    ($func: ident) => {
        $crate::middleware::Custom { func: Box::new(|ctx| Box::pin($func(ctx))) }
    };

    // Convert a block

    ($ctx: ident : $ex: ty => $body: block ) => {
        $crate::middleware::Custom {
            func: Box::new(move |mut $ctx: $crate::Context<'_, $ex>| {
                Box::pin(async move { $body })
            }),
        }
    };

    ($ctx: ident => $body: block) => {
        m!($ctx: () => $body)
    };

    // Convert one expr

    ($ctx: ident : $ex: ty => $body: expr) => {
        m!($ctx: $ex => { $body })
    };

    ($ctx: ident => $body: expr) => {
        m!($ctx => { $body })
    };

    // Convert one stmt

     ($ctx: ident : $ex: ty => $body: stmt $(;)?) => {
        m!($ctx: $ex => { $body ; Ok(()) })
    };

    ($ctx: ident => $body: stmt $(;)?) => {
        m!($ctx => { $body ; Ok(()) })
    };

    // Convert another

    ($ctx: ident : $ex: ty => $($body: tt)+) => {
        m!($ctx: $ex => { $($body)+ })
    };

    ($ctx: ident => $($body: tt)+) => {
        m!($ctx => { $($body)+ })
    };
}
