use {
    amiya::{new_middleware, Amiya},
    futures::future,
    smol,
};

fn main() {
    // Start async worker threads pre cpu core
    for _ in 0..num_cpus::get().max(1) {
        std::thread::spawn(|| smol::run(future::pending::<()>()));
    }

    // Middleware is onion mode, just as NodeJs's koa framework.
    // The executed order is:
    //   - `Logger`'s code before `next()`, which print a log about request in
    //   - `Respond`'s code before `next()`, which do nothing
    //   - `Respond`'s code after `next()`, which set the response body
    //   - `Logger`'s code after `next()`, which read the response body and log it
    let amiya = Amiya::<()>::default()
        // Let's call This middleware `Logger`
        // `ctx.next().await` will return after all inner middleware be executed
        // so the `content` will be "Hello World" , which is set by next middleware.
        .uses(new_middleware!(ctx, {
            println!("new request at");
            ctx.next().await?;
            let content = ctx.resp.take_body().into_string().await.unwrap();
            println!("finish, response is: {}", content);
            ctx.resp.set_body(content);
            Ok(())
        }))
        // Let's call This middleware `Respond`
        // This middleware set tht response
        .uses(new_middleware!(ctx, {
            ctx.next().await?;
            ctx.resp.set_body("Hello World!");
            Ok(())
        }));

    smol::block_on(amiya.listen("[::]:8080")).unwrap();
}
