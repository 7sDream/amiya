//! Amiya is a experimental middleware-based minimalism async HTTP server framework,
//! built up on [`smol-rs`] related asynchronous components.
//!
//! It's currently still working in progress and in a very early alpha stage.
//!
//! API design may changes every day, **DO NOT** use it in any condition except for test or study!
//!
//! ## Goal
//!
//! The goal of this project is try to build a (by importance order):
//!
//! - Safe, with `#![forbid(unsafe_code)]`
//! - Async
//! - Middleware-based
//! - Minimalism
//! - Easy to use
//! - Easy to extend
//!
//! HTTP framework for myself to write simple web services.
//!
//! Amiya uses [`async-h1`] to parse and process requests, so only HTTP version 1.1 is supported for
//! now. HTTP 1.0 or 2.0 is not in goal list, at least in the near future.
//!
//! Performance is **NOT** in the list too, after all, Amiya is just a experimental for now, it uses
//! many heap alloc (Box) and dynamic dispatch (Trait Object) so there may be some performance loss
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
//! some property of [`Request`] and edit [`Response`]" or, a request handler, for now.
//!
//! ### [`Context`]
//!
//! But middleware do not works on [`Request`] and [`Response`] directly. [`Context`] wraps the
//! immutable [`Request`] and mutable [`Response`] with some other information and shortcut
//! methods.
//!
//! ### Onion model
//!
//! The execution process of middleware uses the onion model:
//!
//! ![][img-onion-model]
//!
//! *We reuse this famous picture from [Python's Pylons framework][Pylons-concept-middleware].*
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
//! ```
//! use amiya::{Context, Result, m};
//!
//! async fn a(mut ctx: Context<'_, ()>) -> Result {
//!     println!("A - before");
//!     ctx.next().await?;
//!     println!("A - out");
//!     Ok(())
//! }
//!
//! async fn b(mut ctx: Context<'_, ()>) -> Result {
//!     println!("B - before");
//!     ctx.next().await?;
//!     println!("B - out");
//!     Ok(())
//! }
//!
//! async fn c(mut ctx: Context<'_, ()>) -> Result {
//!     println!("C - before");
//!     ctx.next().await?;
//!     println!("C - out");
//!     Ok(())
//! }
//!
//! let amiya = amiya::new().uses(m!(a)).uses(m!(b)).uses(m!(c));
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
//! You can referrer to [`examples/middleware.rs`] for a more meaningful example.
//!
//! ### Middleware, the truth
//!
//! So with the help of [`next`] method, a middleware can not only be a request handler, it can be:
//!
//! - a error handler, by capture inner middleware returned [`Result`]
//! - a [`Router`], by looking the path then delegate [`Context`] to other corresponding middleware
//! - a access logger or [time measurer], by print log before and after the [`next`] call
//! - etc...
//!
//! A middleware even does not have to call [`next`], in that situation no inner middleware will
//! be executed. Middleware like [`Router`] or login state checker can use this mechanism to make
//! invalid requests respond early.
//!
//! You can create you own [`Middleware`] by implement the trait for your type, or using the [`m`]
//! macro, see their document for detail.
//!
//! ## Examples
//!
//! To start a very simple HTTP service that returns `Hello World` to the client in all paths:
//!
//! ```
//! use amiya::m;
//!
//! let app = amiya::new().uses(m!(ctx =>
//!     ctx.resp.set_body(format!("Hello World from: {}", ctx.path()));
//! ));
//!
//! app.listen("[::]:8080").unwrap();
//!
//! // ... do other things ...
//! ```
//!
//! Amiya has a built-in multi-thread async executor powered by `async-executor` and `async-io`,
//! amiya server will run in it. So `Amiya::listen` do not block your thread.
//!
//! See *[Readme - Examples]* section for more examples to check.
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
//! [`smol-rs`]: https://github.com/smol-rs
//! [`async-h1`]: https://github.com/http-rs/async-h1
//! [img-onion-model]: https://rikka.7sdre.am/files/774eff6f-9368-48d6-8bd2-1b547a74bc23.jpeg
//! [Pylons-concept-middleware]: https://docs.pylonsproject.org/projects/pylons-webframework/en/latest/concepts.html#wsgi-middleware
//! [`examples/middleware.rs`]: https://github.com/7sDream/amiya/blob/master/examples/middleware.rs
//! [time measurer]: https://github.com/7sDream/amiya/blob/master/examples/measurer.rs
//! [Readme - Examples]: https://github.com/7sDream/amiya#examples

#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_debug_implementations, rust_2018_idioms)]
#![forbid(unsafe_code, missing_docs)]
#![allow(clippy::module_name_repetitions)]

mod context;
mod executor;
pub mod middleware;

use {
    async_channel::{Receiver, Sender},
    async_net::TcpListener,
    std::{collections::HashMap, io, net::ToSocketAddrs, sync::Arc},
};

pub use {
    async_trait::async_trait,
    context::Context,
    executor::{BuiltInExecutor, Executor},
    http_types::{Method, Mime, Request, Response, StatusCode},
    middleware::Middleware,
};

/// The Result type all middleware should returns.
pub type Result<T = ()> = http_types::Result<T>;

/// The Error type of middleware result type.
pub type Error = http_types::Error;

type MiddlewareList<Ex> = Vec<Arc<dyn Middleware<Ex>>>;

/// Create a [`Amiya`] instance with extra data type `()`.
///
/// [`Amiya`]: struct.Amiya.html
#[must_use]
pub fn new() -> Amiya<BuiltInExecutor, ()> {
    Amiya::default()
}

/// Create a [`Amiya`] instance with user defined extra data.
///
/// [`Amiya`]: struct.Amiya.html
#[must_use]
pub fn with_ex<Ex>() -> Amiya<BuiltInExecutor, Ex> {
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
pub struct Amiya<Exec, Ex = ()> {
    executor: Exec,
    middleware_list: MiddlewareList<Ex>,
}

impl<Ex> Default for Amiya<BuiltInExecutor, Ex> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Ex> Amiya<BuiltInExecutor, Ex> {
    /// Create a [`Amiya`] instance.
    ///
    /// [`Amiya`]: struct.Amiya
    #[must_use]
    pub fn new() -> Self {
        Self { executor: BuiltInExecutor, middleware_list: MiddlewareList::default() }
    }
}

// `executor`'s return type muse use type name `Amiya`, there are some false
// positive in `clippy:use_self` lint.
// See: https://rust-lang.github.io/rust-clippy/master/index.html#use_self
// TODO: remove after this false positive is fixed
#[allow(clippy::use_self)]
impl<Exec, Ex> Amiya<Exec, Ex>
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
    /// ```
    /// use amiya::m;
    ///
    /// amiya::new().uses(m!(ctx => ctx.next().await));
    /// ```
    ///
    /// ```
    /// use amiya::{m, middleware::Router};
    ///
    /// let router = Router::new().endpoint().get(m!(
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

    /// Set the executor.
    ///
    /// Normal users do not need to call this method because Amiya has a built-in multi-thread
    /// executor [`BuiltInExecutor`]. This method let you change it to your custom one.
    ///
    /// Your executor needs to implement the [`Executor`] trait.
    ///
    /// See [`examples/tokio_executor.rs`] for an example of use tokio async runtime.
    ///
    /// [`BuiltInExecutor`]: struct.BuiltInExecutor.html
    /// [`Executor`]: trait.Executor.html
    /// [`examples/tokio_executor.rs`]: https://github.com/7sDream/amiya/blob/master/examples/tokio_executor.rs
    pub fn executor<NewExec>(self, executor: NewExec) -> Amiya<NewExec, Ex> {
        Amiya { executor, middleware_list: self.middleware_list }
    }
}

impl<Exec, Ex> Amiya<Exec, Ex>
where
    Exec: Executor + 'static,
    Ex: Default + Send + Sync + 'static,
{
    async fn serve(tail: Arc<MiddlewareList<Ex>>, mut req: Request) -> Result<Response> {
        let mut ex = Ex::default();
        let mut resp = Response::new(StatusCode::Ok);
        let mut router_matches = HashMap::new();
        let mut body = Some(req.take_body());
        let mut ctx = Context {
            req: &req,
            body: &mut body,
            resp: &mut resp,
            ex: &mut ex,
            tail: &tail,
            remain_path: req.url().path(),
            router_matches: &mut router_matches,
        };
        ctx.next().await?;
        Ok(resp)
    }

    async fn accepter(
        listener: TcpListener, executor: Arc<Exec>, middleware_list: MiddlewareList<Ex>,
        stop: Receiver<()>,
    ) {
        let middleware_list = Arc::new(middleware_list);
        let mut forever = false;
        loop {
            let check_stop = if forever {
                Ok(listener.accept().await)
            } else {
                let stop_fut = async { Err(stop.recv().await) };
                let accept_fut = async { Ok(listener.accept().await) };
                futures_lite::future::or(stop_fut, accept_fut).await
            };
            match check_stop {
                // accept wins
                Ok(listener_result) => match listener_result {
                    Ok((stream, client_addr)) => {
                        let middleware_list = Arc::clone(&middleware_list);
                        let serve = async_h1::accept(stream, move |mut req| {
                            req.set_peer_addr(Some(client_addr));
                            Self::serve(Arc::clone(&middleware_list), req)
                        });
                        executor.spawn(async move {
                            if let Err(e) = serve.await {
                                log::error!(
                                    "Request handle error: code = {}, type = {}, detail = {}",
                                    e.status(),
                                    e.type_name().unwrap_or("Unknown"),
                                    e,
                                );
                            }
                        });
                    }
                    Err(e) => {
                        log::warn!("Accept connection failed: {:?}", e);
                    }
                },
                // stop signal wins
                Err(signal) => {
                    if signal.is_err() {
                        // channel closed, user want the server runs forever
                        forever = true;
                    } else {
                        // received stop signal
                        log::info!("Amiya server stop listening {:?}", listener.local_addr());
                        return;
                    }
                }
            }
        }
    }

    /// start Amiya server on given `addr`.
    ///
    /// ## Return
    ///
    /// A bounded 1 capacity channel for stop the server.
    ///
    /// Amiya server will stop listening the `addr` when receive message from this channel.
    ///
    /// ## Examples
    ///
    /// ```
    /// amiya::new().listen("127.0.0.1:8080");
    /// ```
    ///
    /// ```
    /// amiya::new().listen(("127.0.0.1", 8080));
    /// ```
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// amiya::new().listen((Ipv4Addr::new(127, 0, 0, 1), 8080));
    /// ```
    ///
    /// ```
    /// use std::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// amiya::new().listen(socket);
    /// ```
    ///
    /// ```
    /// amiya::new().listen("[::]:8080");
    /// ```
    ///
    /// ```
    /// let stop = amiya::new().listen("[::]:8080").unwrap();
    /// // do other things
    /// stop.try_send(()); // amiya http server will stop
    /// ```
    ///
    /// # Errors
    ///
    /// When listen provided address and port failed.
    pub fn listen<A: ToSocketAddrs>(self, addr: A) -> io::Result<Sender<()>> {
        let addr = addr.to_socket_addrs()?.next().unwrap();
        let listener = self.executor.block_on(TcpListener::bind(addr))?;

        log::info!("Amiya server start listening {:?}", listener.local_addr());

        let executor = Arc::new(self.executor);

        let (tx, rx) = async_channel::bounded::<()>(1);
        executor.spawn(Self::accepter(listener, Arc::clone(&executor), self.middleware_list, rx));
        Ok(tx)
    }
}

#[async_trait]
impl<Exec, Ex> Middleware<Ex> for Amiya<Exec, Ex>
where
    Exec: Send + Sync,
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, mut ctx: Context<'_, Ex>) -> Result {
        let mut self_ctx = Context {
            req: ctx.req,
            body: ctx.body,
            resp: ctx.resp,
            ex: ctx.ex,
            tail: &self.middleware_list[..],
            remain_path: ctx.remain_path,
            router_matches: ctx.router_matches,
        };
        self_ctx.next().await?;
        ctx.next().await
    }
}
