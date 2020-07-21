mod common;

use {
    amiya::{
        m,
        middleware::router::{Router, RouterLike},
        Amiya,
    },
    common::response,
};

fn main() {
    let ex = common::global_executor();

    #[rustfmt::skip]
    let router = Router::default()
        .at("api/v1/hello")
            .endpoint().get(m!(ctx => response("Call version 1 hello API\n", ctx).await))
            .done()
        .at("static")
            .endpoint().get(m!(ctx => response("We do not allow list dir\n", ctx).await))
            .fallback().get(m!(ctx => response(format!("Get file {}\n", ctx.remain_path()), ctx).await))
            .done();

    let amiya = Amiya::default().uses(router);

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
