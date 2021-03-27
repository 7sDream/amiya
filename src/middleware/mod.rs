//! Built-in middleware.

mod m;
mod router;

use {
    crate::{Context, Result},
    async_trait::async_trait,
};

pub use {
    m::M,
    router::{MethodRouter, Router, RouterSetter},
};

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
    /// Your middleware handler function, it will be called when request reach this middleware
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result;
}
