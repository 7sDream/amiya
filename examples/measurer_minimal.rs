mod common;

use {amiya::m, std::time::Instant};

fn main() {
    let ex = common::global_executor();

    let app = amiya::new()
        .uses(m!(ctx =>
            let start = Instant::now();
            ctx.next().await?;
            let measure = format!("req; dur={}us", start.elapsed().as_micros());
            ctx.resp.append_header("server-timing", measure);
        ))
        .uses(m!(ctx => ctx.resp.set_body("Finish");));

    blocking::block_on(ex.spawn(app.listen("[::]:8080"))).unwrap();
}
