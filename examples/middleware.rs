use amiya::m;

fn main() {
    // The middleware system of Amiya uses onion model, just as NodeJs's koa framework.
    // The executed order is:
    //   - `Logger`'s code before `next()`, which print a log about request in
    //   - `Respond`'s code before `next()`, which do nothing
    //   - `Respond`'s code after `next()`, which set the response body
    //   - `Logger`'s code after `next()`, which read the response body and log it
    let app = amiya::new()
        // Let's call This middleware `Logger`
        // `ctx.next().await` will return after all inner middleware be executed
        // so the `content` will be "Hello World" , which is set by next middleware.
        .uses(m!(ctx =>
            println!("new request at");
            ctx.next().await?;
            let content = ctx.resp.take_body().into_string().await.unwrap();
            println!("finish, response is: {}", content);
            ctx.resp.set_body(content);
        ))
        // Let's call This middleware `Respond`
        // This middleware set tht response
        .uses(m!(ctx =>
            ctx.next().await?;
            ctx.resp.set_body("Hello World!");
        ));

    smol::run(app.listen("[::]:8080")).unwrap();
}
