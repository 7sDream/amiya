#![allow(warnings)]

use {
    std::{
        net::{ToSocketAddrs, TcpListener},
        fmt::Debug,
        io,
        sync::Arc,
        future::Future,
    },
    futures::{
        future::{self, BoxFuture},
    },
    async_trait::async_trait,
    http_types::{self as ht, Request, Response, StatusCode},
    smol::Async,
};

pub type BoxedRespFut<'x> = BoxFuture<'x, ht::Result<Response>>;
pub type BoxedNextFunc<'x> = Box<dyn FnOnce(Request) -> BoxedRespFut<'x> + Send + 'x>;

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, req: Request, next: BoxedNextFunc<'_>) -> ht::Result<Response>;
}

#[async_trait]
impl<F> Middleware for F
    where F: Fn(Request, BoxedNextFunc<'_>) -> BoxedRespFut<'_> + Send + Sync {
    async fn handle(&self, req: Request, next: BoxedNextFunc<'_>) -> ht::Result<Response> {
        (self)(req, next).await
    }
}

pub struct Logger {

}

impl Logger {
    async fn log<'i>(&'i self, req: Request, next: BoxedNextFunc<'i>) -> ht::Result<Response> {
        println!("Req {} from {}", req.url(), req.peer_addr().unwrap_or("unknown address"));
        let resp = next(req).await;
        println!("Req finish");
        resp
    }
}

#[async_trait]
impl Middleware for Logger {
    async fn handle(&self, req: Request, next: BoxedNextFunc<'_>) -> ht::Result<Response> {
        self.log(req, next).await
    }
}

#[derive(Default)]
pub struct AmiyaBuilder {
    middleware_list: Vec<Arc<dyn Middleware>>,
}

impl AmiyaBuilder {
    pub fn uses<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware_list.push(Arc::new(middleware));
        self
    }

    pub fn build(self) -> Amiya {
        Amiya { inner: Arc::new(AmiyaInner { middleware_list: self.middleware_list })}
    }
}

struct AmiyaInner {
    middleware_list: Vec<Arc<dyn Middleware>>,
}

pub struct Amiya {
    inner: Arc<AmiyaInner>,
}

pub struct NextContext<'x> {
    next_middleware: &'x [Arc<dyn Middleware>],
}

impl<'x> NextContext<'x> {
    fn into_next_func(self) -> BoxedNextFunc<'x> {
        Box::new(move |req| {
            Box::pin(Amiya::next(self, req))
        })
    }
}

impl Amiya {
    async fn next(mut ctx: NextContext<'_>, req: Request) -> ht::Result<Response> {
        if let Some((current, next)) = ctx.next_middleware.split_first() {
            ctx.next_middleware = next;
            current.handle(req, ctx.into_next_func()).await
        } else {
            Ok(ht::Response::new(StatusCode::Ok))
        }
    }

    async fn serve(amiya: Arc<AmiyaInner>, req: Request) -> ht::Result<Response> {
        let ctx = NextContext { next_middleware: &amiya.middleware_list };
        ctx.into_next_func()(req).await
    }

    pub async fn listen<A: ToSocketAddrs + Debug>(self, addr: A) -> io::Result<()> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(io::Error::new(io::ErrorKind::Other, format!("Empty socket address: {:?}", addr)))?;

        let listener = Async::<TcpListener>::bind(addr)?;

        loop {
            let (stream, client_addr) = listener.accept().await?;
            let stream = async_dup::Arc::new(stream);
            let amiya = Arc::clone(&self.inner);
            let serve = async_h1::accept(stream, move |mut req| {
                req.set_peer_addr(Some(client_addr));
                Self::serve(Arc::clone(&amiya), req)
            });
            smol::Task::spawn(async move {
                if let Err(e) = serve.await {
                    eprintln!("Error when process request: {}", e);
                }
            }).detach();
        }
    }
}
