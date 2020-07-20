mod common;

use {
    amiya::{m, middleware::Router, Amiya, Method},
    common::response,
};

fn main() {
    let ex = common::global_executor();

    let api_server = Router::default().sub_router("v1", |v1| {
        v1.sub_endpoint_by_method(
            "login",
            Method::Post,
            m!(ctx => response("Login V1 called\n", ctx).await),
        )
        .sub_endpoint_by_method(
            "logout",
            Method::Post,
            m!(ctx => response("Logout V1 called\n", ctx).await),
        )
    });

    let static_files_server = Amiya::default()
        .uses(m!(ctx => {
            println!("someone visit static file server");
            ctx.next().await
        }))
        .uses(Router::default().fallback_by_method(Method::Get,
            m!(ctx => {
                response(format!("Let's pretend this is content of file {}\n", ctx.remain_path()), ctx).await
            })
        ));

    let amiya = Amiya::default().uses(
        Router::default()
            .sub_middleware("api", api_server)
            // You can use another Amiya server as a middleware
            .sub_middleware("static", static_files_server),
    );

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
