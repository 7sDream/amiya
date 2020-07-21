mod common;

use {
    amiya::{m, middleware::router},
    common::response,
};

fn main() {
    let ex = common::global_executor();

    #[rustfmt::skip]
    let api_server = router().at("v1")
        .at("login").get(m!(ctx => response("Login V1 called\n", ctx).await)).done()
        .at("logout").get(m!(ctx => response("Logout V1 called\n", ctx).await)).done()
        .done();

    #[rustfmt::skip]
    let static_files_server = amiya::new()
        .uses(m!(ctx =>
            println!("someone visit static file server");
            ctx.next().await
        ))
        .uses(router()
            .endpoint().get(m!(ctx => response("We do not allow list dir", ctx).await))
            .fallback().get(m!(ctx => response(format!("request file {}\n", ctx.remain_path()), ctx).await))
        );

    #[rustfmt::skip]
    let amiya = amiya::new().uses(router()
        .at("api").uses(api_server)
        // You can use another Amiya server as a middleware
        .at("static").uses(static_files_server));

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
