use {amiya::m, std::time::Instant};

fn main() {
    let app = amiya::new()
        .uses(m!(ctx =>
            let start = Instant::now();
            ctx.next().await?;
            let measure = format!("req; dur={}us", start.elapsed().as_micros());
            ctx.resp.append_header("server-timing", measure);
        ))
        .uses(m!(ctx => ctx.resp.set_body("Finish");));

    smol::block_on(app.listen("[::]:8080")).unwrap();
}
