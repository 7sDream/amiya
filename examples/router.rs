use {
    amiya::{middleware::Router, new_middleware, Amiya, Context, Result},
    futures::future,
    smol,
};

async fn response(mut ctx: Context<'_, ()>, data: &'static str) -> Result {
    ctx.next().await?;
    ctx.resp.set_body(data);
    Ok(())
}

fn main() {
    for _ in 0..num_cpus::get().max(1) {
        std::thread::spawn(|| smol::run(future::pending::<()>()));
    }

    let amiya = Amiya::default()
        // Amiya support extra data attach in context, just set it's type as second argument
        .uses(Router::default()
            .route("api").route(|api| {
                api.route("v1").route(|v1| {
                    v1.route("hello").uses(new_middleware!(ctx, (), {
                        response(ctx, "from hello").await
                    }))
                })
        }));
    smol::block_on(amiya.listen("[::]:8080")).unwrap();
}
