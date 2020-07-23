mod common;

use amiya::Context;
use {
    amiya::{middleware::Middleware, Result},
    async_trait::async_trait,
    std::time::Instant,
};

struct TimeMeasurer;

#[async_trait]
impl<Ex> Middleware<Ex> for TimeMeasurer
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, mut ctx: Context<'_, Ex>) -> Result {
        let start = Instant::now();
        ctx.next().await?;
        let measure = format!("req;dur={}us", start.elapsed().as_micros());
        ctx.resp.append_header("server-timing", measure);
        Ok(())
    }
}

struct RequestHandler;

#[async_trait]
impl<Ex> Middleware<Ex> for RequestHandler
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, mut ctx: Context<'_, Ex>) -> Result {
        ctx.next().await?;
        ctx.resp.set_body("Finish");
        Ok(())
    }
}

fn main() {
    let ex = common::global_executor();

    let app = amiya::new().uses(TimeMeasurer).uses(RequestHandler);

    blocking::block_on(ex.spawn(app.listen("[::]:8080"))).unwrap();

    // $ curl http://localhost:8080/ -v
    // *   Trying ::1...
    // * TCP_NODELAY set
    // * Connected to localhost (::1) port 8080 (#0)
    // > GET /api/v1/hello/ HTTP/1.1
    // > Host: localhost:8080
    // > User-Agent: curl/7.64.1
    // > Accept: */*
    // >
    // < HTTP/1.1 200 OK
    // < content-length: 6
    // < date: Thu, 23 Jul 2020 15:50:07 GMT
    // < content-type: text/plain;charset=utf-8
    // < server-timing: req;dur=9us                     <------------- Added by TimeMeasurer
    // <
    // * Connection #0 to host localhost left intact
    // Finish* Closing connection 0                     <------------- Set by RequestHandler
}
