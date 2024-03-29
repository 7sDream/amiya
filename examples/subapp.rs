mod common;

use {
    amiya::{m, middleware::Router},
    common::response,
};

fn main() {
    #[rustfmt::skip]
    let api_server = Router::new().at("v1")
        // When we do not want to set the exact match handler, you can directly call a new `at`
        // to start sub router table setting.
        // Then a call to `get` means we set the exact match handler for "/v1/login"
        .at("login").get(m!(ctx => response("Login V1 called\n", ctx).await)).done()
        // As the same as login, we set the exact match handler for "/v1/logout"
        .at("logout").get(m!(ctx => response("Logout V1 called\n", ctx).await)).done()
        // Finish "/v1" sub router setting
        .done();

    #[rustfmt::skip]
    let static_files_server = amiya::new()
        .uses(m!(ctx =>
            println!("someone visit static file server");
            ctx.next().await?;
        ))
        .uses(Router::new() 
            // `endpoint` enter exact match handler setting context for new router. For sub router
            // (when use `at`) we do not call it explicit
            .endpoint().get(m!(ctx => response("We do not allow list dir", ctx).await))
            .fallback().get(m!(ctx => response(format!("Get file {}\n", ctx.path()), ctx).await))
            // Do not needs a `done` here, because we are setting router itself, not sub router
        );

    #[rustfmt::skip]
    let app = amiya::new().uses(Router::new()
        // `is` use the middleware you give as the path's handler, no matter exact match or sub match
        .at("api").is(api_server)
        // You can use another Amiya server as a middleware too,
        // so the static files server handler all request under "/static" path
        .at("static").is(static_files_server));

    app.listen("[::]:8080").unwrap();

    std::thread::park();
}
