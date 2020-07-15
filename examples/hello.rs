use {
    amiya::{middleware::BoxedResultFut, Amiya, Context},
    futures::future,
    smol,
};

fn log(ctx: Context<'_>) -> BoxedResultFut<'_> {
    Box::pin(async {
        println!(
            "Request {} from {}",
            ctx.req.url(),
            ctx.req.remote().unwrap_or("unknown address")
        );
        let resp = ctx.next().await;
        if let Err(err) = resp.as_ref() {
            eprintln!("Request process error: {}", err);
        }
        resp
    })
}

fn response(ctx: Context<'_>) -> BoxedResultFut<'_> {
    Box::pin(async {
        let mut resp = ctx.next().await;
        if let Ok(ref mut resp) = resp {
            resp.set_body("Hello from Amiya!");
        }
        resp
    })
}

fn main() {
    for _ in 0..num_cpus::get().max(1) {
        std::thread::spawn(|| smol::run(future::pending::<()>()));
    }

    let amiya = Amiya::default().uses(log).uses(response);

    smol::block_on(amiya.listen("[::]:8080")).unwrap();
}
