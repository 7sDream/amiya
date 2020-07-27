use {
    amiya::{async_trait, Context, Middleware, Result},
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
        let measure = format!("req; dur={}us", start.elapsed().as_micros());
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
    let app = amiya::new().uses(TimeMeasurer).uses(RequestHandler);

    smol::run(app.listen("[::]:8080")).unwrap();
}

// < HTTP/1.1 200 OK
// < content-length: 6
// < date: Thu, 23 Jul 2020 15:50:07 GMT
// < content-type: text/plain;charset=utf-8
// < server-timing: req;dur=9us                     <------------- Added by TimeMeasurer
// <
// * Connection #0 to host localhost left intact
// Finish* Closing connection 0                     <------------- Set by RequestHandler

// Referer to `examples/measurer_minimal.rs` to see how to macro `m` to achieve the same result
