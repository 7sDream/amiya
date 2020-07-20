mod common;

use {
    amiya::{m, middleware::Router, Amiya},
    common::response,
};

fn main() {
    common::start_smol_workers();

    let router = Router::default()
        // sub endpoint will be executed if and only if the remain path is exactly  
        .sub_endpoint("api/v1/hello", m!(ctx => response("Call version 1 hello API\n", ctx).await))
        .sub_router("statics", |statics| {
            statics
                .endpoint(m!(ctx => response("We do not allow list dir\n", ctx).await))
                // fallback will execute if and only if not not exactly meets and no item in router table matches
                // That is, all sub path do not exist in router table
                .fallback_by_method_router(|files| {
                    files.get(m!(ctx => response(format!("Get file {}\n", ctx.remain_path()), ctx).await))
                })
        });

    let amiya = Amiya::default().uses(router);

    amiya.listen_block("[::]:8080").unwrap();
}
