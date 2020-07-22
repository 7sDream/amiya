//! Amiya is a experimental middleware-based minimalism async HTTP server framework built up on the
//! [`smol`] async runtime.
//!
//! As a newbie to rust's async world, It's a personal study project to learn async related concept
//! and practice.
//!
//! It's currently still working in progress and in a very early alpha stage.
//!
//! API design may changes every day, **DO NOT** use it in any condition except for test or study!
//!
//! ## Goal
//!
//! The goal of this project is try to build a (by importance order):
//!
//! - Safe
//! - Async
//! - Minimalism
//! - Easy to use
//! - Easy to extend
//!
//! HTTP framework for myself to write simple web services.
//!
//! Amiya uses [`async-h1`] to parse and process requests, so only HTTP version 1.1 is supported
//! for now. HTTP 2.0 is not in goal list, at least in the near future.
//!
//! Performance is NOT in the list too, after all, it's just a experimental. Amiya use many heap
//! alloc (Box) and Dynamic Dispatch (Trait Object) so there may be some performance loss compare to
//! use `async-h1` directly.
//!
//! ## Examples
//!
//! To start a very simple HTTP service to return Hello World to the client in all path:
//!
//! ```rust
//! use amiya::m;
//!
//! fn main() {
//!     let app = amiya::new().uses(m!(ctx =>
//!         ctx.resp.set_body(format!("Hello World from: {}", ctx.path()));
//!     ));
//!
//!     let fut = app.listen("[::]:8080");
//!
//!     // ... start a async runtime and block on `fut` ...
//! }
//! ```
//!
//! You can await or block on this `fut` to start the service.
//!
//! Notice any future need a async runtime to do this, and that's not amiya's goal too. But you
//! can refer to [`examples/hello.rs`] for a minimal example of how to start [`smol`] runtime.
//!
//! You can check other examples for:
//!
//! - Understand onion model of Amiya's middleware system: [`examples/middleware.rs`]
//! - How to store extra data in context: [`examples/extra.rs`]
//! - Use [`Router`] middleware for request diversion: [`examples/router.rs`]
//! - Use another Amiya service as a middleware: [`examples/subapp.rs`]
//!
//! [`smol`]: https://github.com/stjepang/smol
//! [`async-h1`]: https://github.com/http-rs/async-h1
//! [`examples/hello.rs`]: https://github.com/7sDream/amiya/blob/master/examples/hello.rs
//! [`examples/middleware.rs`]: https://github.com/7sDream/amiya/blob/master/examples/middleware.rs
//! [`examples/extra.rs`]: https://github.com/7sDream/amiya/blob/master/examples/extra.rs
//! [`examples/router.rs`]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
//! [`examples/subapp.rs`]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
//! [`Router`]: middleware/struct.Router.html

#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_debug_implementations, rust_2018_idioms)]
#![forbid(unsafe_code, missing_docs)]
#![allow(clippy::module_name_repetitions)]

mod context;
pub mod middleware;

use {
    async_net::{AsyncToSocketAddrs, TcpListener},
    async_trait::async_trait,
    middleware::Middleware,
    std::{fmt::Debug, io, sync::Arc},
};

pub use {
    context::Context,
    http_types::{Method, Request, Response, StatusCode},
};

/// The Result type all middleware should returns
pub type Result<T = ()> = http_types::Result<T>;

type MiddlewareList<Ex> = Vec<Arc<dyn Middleware<Ex>>>;

/// Create a [`Amiya`] instance
///
/// [`Amiya`]: struct.Amiya.html
pub fn new<Ex>() -> Amiya<Ex> {
    Amiya::default()
}

/// Amiya App type.
///
/// TODO: write document and example
#[allow(missing_debug_implementations)]
pub struct Amiya<Ex = ()> {
    middleware_list: MiddlewareList<Ex>,
}

impl<Ex> Default for Amiya<Ex> {
    fn default() -> Self {
        Self { middleware_list: vec![] }
    }
}

impl<Ex> Amiya<Ex> {
    /// Create a [`Amiya`] instance
    ///
    /// [`Amiya`]: struct.Amiya
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Ex> Amiya<Ex>
where
    Ex: Send + Sync + 'static,
{
    /// Add a middleware to the end, you can create middleware by
    /// implement the [`Middleware`] trait for your custom type or use the
    /// [`m`] macro to convert a async func or closure.
    ///
    /// [`Middleware`]: middleware/trait.Middleware.html
    /// [`m`]: macro.m.html
    pub fn uses<M: Middleware<Ex> + 'static>(mut self, middleware: M) -> Self {
        self.middleware_list.push(Arc::new(middleware));
        self
    }
}

impl<Ex> Amiya<Ex>
where
    Ex: Default + Send + Sync + 'static,
{
    async fn serve(tail: Arc<MiddlewareList<Ex>>, req: Request) -> Result<Response> {
        let mut ex = Ex::default();
        let mut resp = Response::new(StatusCode::Ok);
        let mut ctx = Context {
            req: &req,
            resp: &mut resp,
            ex: &mut ex,
            tail: &tail,
            remain_path: &req.url().path()[1..],
        };
        ctx.next().await?;
        Ok(resp)
    }

    /// Start Amiya app on `addr`
    pub async fn listen<A: AsyncToSocketAddrs + Debug>(self, addr: A) -> io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let middleware_list = Arc::new(self.middleware_list);

        loop {
            match listener.accept().await {
                Ok((stream, client_addr)) => {
                    let middleware_list = Arc::clone(&middleware_list);
                    let serve = async_h1::accept(stream, move |mut req| {
                        req.set_peer_addr(Some(client_addr));
                        Self::serve(Arc::clone(&middleware_list), req)
                    });
                    smol::Task::spawn(async move {
                        if let Err(e) = serve.await {
                            eprintln!("Error when process request: {}", e);
                        }
                    })
                    .detach();
                }
                Err(e) => {
                    eprintln!("Accept connection error: {}", e);
                }
            }
        }
    }
}

#[async_trait]
impl<Ex: Send + Sync + 'static> Middleware<Ex> for Amiya<Ex> {
    async fn handle(&self, mut ctx: Context<'_, Ex>) -> Result<()> {
        let mut self_ctx = Context {
            req: ctx.req,
            resp: ctx.resp,
            ex: ctx.ex,
            tail: &self.middleware_list[..],
            remain_path: &ctx.remain_path,
        };
        self_ctx.next().await?;
        ctx.next().await
    }
}
