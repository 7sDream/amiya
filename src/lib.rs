//! Amiya is a **experimental** middleware-based minimalism async HTTP server framework built up on
//! the [`smol`] async runtime.
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
//! ## Concepts
//!
//! To understand how this framework works, there are some concept need to be described first.
//!
//! ### Request, Response and the process pipeline
//!
//! For every HTTP request comes to a Amiya server, the framework will create a [`Request`] struct
//! to represent it. It's immutable in the whole request process pipeline.
//!
//! And a [`Response`] is created at the same time. It's a normal `200 OK` empty header empty body
//! response at first, but it's mutable and can be edit by middleware.
//!
//! After all middleware has been executed, the [`Response`] maybe edited by many middleware, and
//! as the final result we will send to the client.
//!
//! ### [`Middleware`]
//!
//! For ease of understanding, you can think this word is a abbreviation of "A function read
//! some propety of [`Request`] and edit [`Response`]" or, a request handler, for now.
//!
//! ### [`Context`]
//!
//! But middleware do not works on [`Request`] and [`Response`] directly. [`Context`] wraps the
//! immutable [`Request`] and editable [`Response`] with some other information and shortcut
//! methods.
//!
//! ### Onion model
//!
//! The execution process of middleware uses the onion model:
//!
//! ![][img-onion-model]
//!
//! *We reuse this famous picture from Nodejs' [Koa] framework.*
//!
//! If we add middleware A, B and C to Amiya server, the running order(if not interrupted in the
//! middle) will be: A -> B -> C -> C -> B -> A
//!
//! So every middleware will be executed twice, but this does not mean same code is executed twice.
//!
//! That's why [`next`] method exists.
//!
//! ### [`next`]
//!
//! The most important method [`Context`] gives us is [`next`].
//!
//! When a middleware calls `ctx.next().await`, the method will return after all inner middleware
//! finish, or, some of them returns a Error.
//!
//! there is a simplest example:
//!
//! ```rust
//! use amiya::{Context, Result, m};
//!
//! async fn A(mut ctx: Context<'_, ()>) -> Result {
//!     println!("A - before");
//!     ctx.next().await?;
//!     println!("A - out");
//!     Ok(())
//! }
//!
//! async fn B(mut ctx: Context<'_, ()>) -> Result {
//!     println!("B - before");
//!     ctx.next().await?;
//!     println!("B - out");
//!     Ok(())
//! }
//!
//! async fn C(mut ctx: Context<'_, ()>) -> Result {
//!     println!("C - before");
//!     ctx.next().await?;
//!     println!("C - out");
//!     Ok(())
//! }
//!
//! let amiya = amiya::new().uses(m!(A)).uses(m!(B)).uses(m!(C));
//! ```
//!
//! When a request in, the output will be:
//!
//! ```console
//! A - before
//! B - before
//! C - before
//! C - after
//! B - after
//! A - after
//! ```
//!
//! You can referer to [`examples/middleware.rs`] for a more meaningful example.
//!
//! ### Middleware, the truth
//!
//! So with the help of [`next`] method, a middleware can not only be a request handler, it can be:
//!
//! - a error handler, by catpure inner middleware's return [`Result`])
//! - a [`Router`], by looking the path then delegate the ctx to corresponding other middleware)
//! - a Logger or time measurer, by print log before and after the [`next`] call)
//! - etc...
//!
//! A middleware even does not have to call [`next`], in that statution no inner middlewares will
//! be executed. Middleware like [`Router`] or login state checker can use this mechanism to make
//! unprocessable requests responsed early.
//!
//! You can create you own [`Middleware`] by implement the trait for your type, or using the [`m`]
//! macro, see their document for detail.
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
//! [`examples`] folder has more examples if you want to check.
//!
//! [`Request`]: struct.Request.html
//! [`Response`]: struct.Response.html
//! [`Middleware`]: middleware/trait.Middleware.html
//! [`Context`]: struct.Context.html
//! [`next`]: struct.Context.html#method.next
//! [`Result`]: type.Result.html
//! [`Router`]: middleware/struct.Router.html
//! [`m`]: macro.m.html
//!
//! [`smol`]: https://github.com/stjepang/smol
//! [`async-h1`]: https://github.com/http-rs/async-h1
//! [img-onion-model]: https://rikka.7sdre.am/files/b1743b18-ba9f-4dc1-8c1d-15945978d0b1.png
//! [Koa]: https://github.com/koajs/koa
//! [`examples`]: https://github.com/7sDream/amiya/blob/master/examples

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

/// Create a [`Amiya`] instance with extra data type `()`.
///
/// [`Amiya`]: struct.Amiya.html
pub fn new() -> Amiya<()> {
    Amiya::default()
}

/// Create a [`Amiya`] instance with user defined extra data.
///
/// [`Amiya`]: struct.Amiya.html
pub fn with_ex<Ex>() -> Amiya<Ex> {
    Amiya::default()
}

/// Amiya HTTP Server.
///
/// Amiya itself also implement the [`Middleware`] trait and can be added to another Amiya
/// instance, see [`examples/subapp.rs`] for a example.
///
/// [`Middleware`]: middleware/trait.Middleware.html
/// [`examples/subapp.rs`]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
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
    /// Create a [`Amiya`] instance.
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
    /// Add a middleware to the end, middleware will be executed as the order of be added.  
    ///
    /// You can create middleware by implement the [`Middleware`] trait
    /// for your custom type or use the [`m`] macro to convert a async func or closure.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use amiya::m;
    ///
    /// amiya::new().uses(m!(ctx => ctx.next().await));
    /// ```
    ///
    /// ```rust
    /// use amiya::{m, middleware::router};
    ///
    /// let router = router().endpoint().get(m!(
    ///     ctx => ctx.resp.set_body("Hello world!");
    /// ));
    ///
    /// amiya::new().uses(router);
    /// ```
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
            remain_path: req.url().path(),
        };
        ctx.next().await?;
        Ok(resp)
    }

    /// Run Amiya server on given `addr`
    ///
    /// ## Examples
    ///
    /// ```rust
    /// amiya::new().listen("127.0.0.1:8080");
    /// ```
    ///
    /// ```rust
    /// amiya::new().listen(("127.0.0.1", 8080));
    /// ```
    ///
    /// ```rust
    /// use std::net::Ipv4Addr;
    ///
    /// amiya::new().listen((Ipv4Addr::new(127, 0, 0, 1), 8080));
    /// ```
    ///
    /// ```rust
    /// use std::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// amiya::new().listen(socket);
    /// ```
    ///
    /// ```rust
    /// amiya::new().listen("[::]:8080");
    /// ```
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
            remain_path: ctx.remain_path,
        };
        self_ctx.next().await?;
        ctx.next().await
    }
}
