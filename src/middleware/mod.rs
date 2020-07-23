//! Built-in middleware

mod router;

use {
    crate::{Context, Result},
    async_trait::async_trait,
    std::{future::Future, pin::Pin},
};

pub use router::{router, MethodRouter, Router};

type BoxedResultFut<'x> = Pin<Box<dyn Future<Output = Result> + Send + 'x>>;

/// Use your custom type as a middleware by implement this trait.
///
/// You need [`async_trait`] to implement this.
///
/// See [`examples/measurer.rs`] for a example of process time usage measurer middleware.
///
/// [`async_trait`]: https://github.com/dtolnay/async-trait
/// [`examples/measurer.rs`]: https://github.com/7sDream/amiya/blob/master/examples/measurer.rs
#[async_trait]
pub trait Middleware<Ex>: Send + Sync {
    /// You middleware's handler function, it will be called when request reach this middleware
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result;
}

/// The wrapper for use async function or closure as a middleware.
///
/// This is the type when you use macro [`m`] , **Do Not** use this type directly!
///
/// [`m`]: ../macro.m.html
#[allow(missing_debug_implementations)]
pub struct M<Ex> {
    /// the code in macro [`m`], converted to a boxed async func.
    ///
    /// **Do Not** set this field by hand, use macro [`m`] instead!
    ///
    /// [`m`]: ../macro.m.html
    pub func: Box<dyn Fn(Context<'_, Ex>) -> BoxedResultFut<'_> + Send + Sync>,
}

#[async_trait]
impl<Ex> Middleware<Ex> for M<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result<()> {
        (self.func)(ctx).await
    }
}

/// Writer middleware easily.
///
/// It's a macro to let you easily write middleware use closure and syntax like Javascript's
/// arrow function, or convert a async fn to a middleware use the `m!(async_func_name)` syntax.
///
/// It returns a [`M`] instance, which implement [`Middleware`] trait.
///
/// ## Examples
///
/// TODO: example
///
/// [`M`]: middleware/struct.M.html
/// [`Middleware`]: middleware/trait.Middleware.html
#[macro_export]
macro_rules! m {
    // Convert a async function to middleware by func's name

    ($func: ident) => {
        $crate::middleware::M { func: Box::new(|ctx| Box::pin($func(ctx))) }
    };

    // Convert a block

    ($ctx: ident : $ex: ty => $body: block ) => {
        $crate::middleware::M {
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
