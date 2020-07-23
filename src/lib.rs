//! # Amiya
//!
//! [![][doc-badge-img]][doc-gh-pages]
//!
//! Amiya is a experimental middleware-based minimalism async HTTP server framework built up on the
//! [`smol`] async runtime.
//!
//! I, a newbie to Rust's async world, start to write Amiya as a personal study project to learn
//! async related concept and practice.
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
//! Amiya uses [`async-h1`] to parse and process requests, so only HTTP version 1.1 is supported for
//! now. HTTP 2.0 is not in goal list, at least in the near future.
//!
//! Performance is **NOT** in the list too, after all, Amiya is just a experimental for now, it uses
//! many heap alloc (Box) and Dynamic Dispatch (Trait Object) so there may be some performance loss
//! compare to use `async-h1` directly.
//!
//! ## Examples
//!
//! To start a very simple HTTP service that returns `Hello World` to the client in all paths:
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
//! Notice any future need a async runtime to run, and that's not amiya's goal too. But you can
//! refer to [`examples/hello.rs`] for a minimal example of how to start [`smol`] runtime.
//!
//! To run those examples, run
//!
//! ```bash
//! $ cargo run --example # show example list
//! $ cargo run --example hello # run hello
//! ```
//!
//! [Document of Amiya struct][doc-struct-Amiya] has some brief description of concept you need to
//! understand before use it, you can check/run other examples after read it:
//!
//! - Understand onion model of Amiya's middleware system: [`examples/middleware.rs`]
//! - How to store extra data in context: [`examples/extra.rs`]
//! - Use `Router` middleware for request diversion: [`examples/router.rs`]
//! - Use another Amiya service as a middleware: [`examples/subapp.rs`]
//!
//! ## License
//!
//! BSD 3-Clause Clear License, See [`LICENSE`].
//!
//! [doc-badge-img]: https://img.shields.io/badge/docs-on_github_pages-success?style=flat-square&logo=read-the-docs
//! [doc-gh-pages]: https://7sdream.github.io/amiya/master/amiya
//! [`smol`]: https://github.com/stjepang/smol
//! [`async-h1`]: https://github.com/http-rs/async-h1
//! [doc-struct-Amiya]: struct.Amiya.html
//! [`examples/hello.rs`]: https://github.com/7sDream/amiya/blob/master/examples/hello.rs
//! [`examples/middleware.rs`]: https://github.com/7sDream/amiya/blob/master/examples/middleware.rs
//! [`examples/extra.rs`]: https://github.com/7sDream/amiya/blob/master/examples/extra.rs
//! [`examples/router.rs`]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
//! [`examples/subapp.rs`]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
//! [`LICENSE`]: https://github.com/7sDream/amiya/blob/master/LICENSE

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

/// Amiya Http Server.
///
/// For understand how this framework works, there are some concept need to be described first.
///
/// ## Concepts
///
/// ### Request, Response and the process pipeline
///
/// For every Http Request comes to a Amiya server, the framework will create a Struct [`Request`] to
/// represent it, It's **immutable** in the whole request process pipeline.
///
/// And a [`Response`] is created at the same time. It's a normal `200 OK` empty header empty body
/// response at first, but it's mutable and can be edit by middleware.
///
/// After all middleware has been executed, the [`Response`] maybe edit by many middleware, and it's
/// the final result we will send to the client.
///
/// ### [`Middleware`]
///
/// For ease of understanding, you can think this word is a abbreviation of "A function read
/// some propety of [`Request`] and edit [`Response`]" or, a Request Handler, for now.
///
/// ### [`Context`]
///
/// But middleware do not works on [`Request`] and [`Response`] directly. [`Context`] wraps the
/// immutable [`Request`] and editable [`Response`] with some other information and shortcuts
/// method.
///
/// ### Onion model
///
/// Middleware is controlled by a system using the onion model.
///
/// ![][img-onion-model]
///
/// *We reuse this famous picture from Nodejs' [Koa] framework.*
///
/// If we add middleware A, B and C to Amiya server, the running order will be(in normal
/// condition): A -> B -> C -> C -> B -> A
///
/// So every middleware will be executed twice, but this not mean same code executed twice.
///
/// That's why [`next`] method exists.
///
/// ### [`next`]
///
/// The most important method [`Context`] given is [`next`].
///
/// When a middleware calls `ctx.next().await`, the method will return after all inner middleware
/// finish.
///
/// there is a simplest example:
///
/// ```rust
/// use amiya::{Context, Result, m};
///
/// async fn A(mut ctx: Context<'_, ()>) -> Result {
///     println!("A - before");
///     ctx.next().await?;
///     println!("A - out");
///     Ok(())
/// }
///
/// async fn B(mut ctx: Context<'_, ()>) -> Result {
///     println!("B - before");
///     ctx.next().await?;
///     println!("B - out");
///     Ok(())
/// }
///
/// async fn C(mut ctx: Context<'_, ()>) -> Result {
///     println!("C - before");
///     ctx.next().await?;
///     println!("C - out");
///     Ok(())
/// }
///
/// let amiya = amiya::new().uses(m!(A)).uses(m!(B)).uses(m!(C));
/// ```
///
/// When a request in, the output will be:
///
/// ```console
/// A - before
/// B - before
/// C - before
/// C - after
/// B - after
/// A - after
/// ```
///
/// You can referer to [`examples/middleware.rs`] for a more meaningful example.
///
/// ### Middleware 2
///
/// So with the help of [`next`] method, a middleware can not noly be a request handler, it can be a
/// error handler(by catpure inner middleware's return [`Result`]), a [`Router`] (by looking the
/// path then delegate the ctx to corresponding other middleware), a Logger or Time measurer (by
/// print log before and after the [`next`] call), etc...
///
/// You can create you own [`Middleware`] by implement the trait for your type, or using the [`m`]
/// macro, see their document for detail.
///
/// [`Request`]: struct.Request.html
/// [`Response`]: struct.Response.html
/// [`Middleware`]: middleware/trait.Middleware.html
/// [`Context`]: struct.Context.html
/// [Koa]: https://github.com/koajs/koa
/// [`next`]: struct.Context.html#method.next
/// [`Result`]: type.Result.html
/// [`m`]: macro.m.html
/// [img-onion-model]: https://rikka.7sdre.am/files/b1743b18-ba9f-4dc1-8c1d-15945978d0b1.png
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
