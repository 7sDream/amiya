mod context;
pub mod middleware;

use {
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
    http_types::{Request, Response, StatusCode},
};

pub type Result<T = Response> = http_types::Result<T>;

type MiddlewareList = Vec<Arc<dyn Middleware>>;

#[derive(Default)]
pub struct Amiya {
    middleware_list: MiddlewareList,
}

impl Amiya {
    pub fn uses<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware_list.push(Arc::new(middleware));
        self
    }

    async fn serve(tail: Arc<MiddlewareList>, req: Request) -> Result {
        let ctx = Context { req, tail: &tail };
        ctx.next().await
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
}
