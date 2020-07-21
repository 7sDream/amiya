mod common;

use {
    amiya::{m, middleware::router},
    common::response,
};

fn main() {
    let ex = common::global_executor();

    #[rustfmt::skip]
    let api_server = router().at("v1")
        // when we do not want to set the exact match handler, call a new `at` let you do router table setting.
        // then a call to get means we set the exact match handler for "/v1/login"
        .at("login").get(m!(ctx => response("Login V1 called\n", ctx).await)).done()
        // the same as login, we set the exact match handler for "/v1/logout"
        .at("logout").get(m!(ctx => response("Logout V1 called\n", ctx).await)).done()
        // we finish "/v1" sub router setting
        .done();

    #[rustfmt::skip]
    let static_files_server = amiya::new()
        .uses(m!(ctx =>
            println!("someone visit static file server");
            ctx.next().await
        ))
        .uses(router()
            // `endpoint` enter exact match handler setting context for new router
            // for sub router (use `at`) we do not call it explicit
            .endpoint().get(m!(ctx => response("We do not allow list dir", ctx).await))
            .fallback().get(m!(ctx => response(format!("request file {}\n", ctx.remain_path()), ctx).await))
            // we do not needs a `done` here, because we are not setting sub router, just the router itself
        );

    #[rustfmt::skip]
    let amiya = amiya::new().uses(router()
        .at("api").is(api_server)
        // You can use another Amiya server as a middleware
        .at("static").is(static_files_server));

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
