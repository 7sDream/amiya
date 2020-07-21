mod common;

use {
    amiya::{m, middleware::router},
    common::response,
};

fn main() {
    let ex = common::global_executor();

    #[rustfmt::skip]
    let router = router()
        .at("api/v1/hello")
            .get(m!(ctx => response("Call version 1 hello API\n", ctx).await))
            .done()
        .at("static")
            .get(m!(ctx => response("We do not allow list dir\n", ctx).await))
            .fallback().get(m!(ctx => response(format!("Get file {}\n", ctx.remain_path()), ctx).await))
            .done();

    let amiya = amiya::new().uses(router);

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
