mod common;

use {
    amiya::{m, middleware::router},
    common::response,
};

fn main() {
    let ex = common::global_executor();

    #[rustfmt::skip]
    let amiya = amiya::new().uses(router()
        // `at` let you set the handler when exact meets the path,
        // `get` let you limit this path only accept get request and set the handler
        // `done` finish router setting for "/api/v1/hello"
        .at("api/v1/hello").get(m!(ctx => response("Call version 1 hello API\n", ctx).await)).done()
        .at("static")
            // As above, we give request to exact "/static" a clear response message that 
            // we do support list dir content
            .get(m!(ctx => response("We do not allow list dir\n", ctx).await))
            // but we not finish this setting here, a call to `fallback` let you set the
            // handler of all other request just except exact match
            // `get` limit we only support get method for static files
            .fallback().get(m!(ctx => response(format!("Get file {}\n", ctx.path()), ctx).await))
            // and we finish "/static" router setting
            .done());

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
