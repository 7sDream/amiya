#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]

mod context;
pub mod middleware;

use {
    async_trait::async_trait,
    middleware::Middleware,
    smol::Async,
    std::{
        fmt::Debug,
        io,
        net::{TcpListener, ToSocketAddrs},
        sync::Arc,
    },
};

pub use {
    context::Context,
    http_types::{Method, Request, Response, StatusCode},
};

pub type Result<T = ()> = http_types::Result<T>;

type MiddlewareList<Ex> = Vec<Arc<dyn Middleware<Ex>>>;

#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct Amiya<Ex = ()> {
    middleware_list: MiddlewareList<Ex>,
}

impl<Ex> Amiya<Ex>
where
    Ex: Default + Send + Sync + 'static,
{
    pub fn uses<M: Middleware<Ex> + 'static>(mut self, middleware: M) -> Self {
        self.middleware_list.push(Arc::new(middleware));
        self
    }

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

    pub async fn listen<A: ToSocketAddrs + Debug>(self, addr: A) -> io::Result<()> {
        let addr = addr.to_socket_addrs()?.next().ok_or(io::Error::new(
            io::ErrorKind::Other,
            format!("Empty socket address: {:?}", addr),
        ))?;

        let listener = Async::<TcpListener>::bind(addr)?;
        let middleware_list = Arc::new(self.middleware_list);

        loop {
            match listener.accept().await {
                Ok((stream, client_addr)) => {
                    let stream = async_dup::Arc::new(stream);
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

    pub fn listen_block<A: ToSocketAddrs + Debug>(self, addr: A) -> io::Result<()> {
        smol::block_on(self.listen(addr))
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
