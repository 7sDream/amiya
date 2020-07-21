mod common;

use {
    amiya::{
        m,
        middleware::{MethodRouter, Router},
        Amiya,
    },
    common::response,
};

fn main() {
    let ex = common::global_executor();

    #[rustfmt::skip]
    let api_server = Router::default()
        .at("v1")
            .at("login").endpoint().get(m!(ctx => response("Login V1 called\n", ctx).await)).done()
            .at("logout").endpoint().get(m!(ctx => response("Logout V1 called\n", ctx).await)).done()
            .done();

    let static_files_server = Amiya::default()
        .uses(m!(ctx => {
            println!("someone visit static file server");
            ctx.next().await
        }))
        .uses(MethodRouter::default().get(
            m!(ctx => {
                response(format!("Let's pretend this is content of file {}\n", ctx.remain_path()), ctx).await
            })
        ));

    #[rustfmt::skip]
    let amiya = Amiya::default().uses(Router::default()
        .at("api").is(api_server)
        // You can use another Amiya server as a middleware
        .at("static").is(static_files_server));

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
