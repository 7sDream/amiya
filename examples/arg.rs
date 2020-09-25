mod common;

use {
    amiya::{m, middleware::Router, Context, Result, StatusCode},
    std::convert::TryInto,
};

async fn return_status_code(mut ctx: Context<'_, ()>) -> Result {
    ctx.next().await?;

    let code_arg = ctx.arg("status_code").unwrap();

    if let Ok(code_num) = code_arg.parse::<u16>() {
        if let Ok(code) = code_num.try_into() {
            ctx.resp.set_status(code);
            return Ok(());
        }
    }

    ctx.resp.set_status(StatusCode::BadRequest);
    Ok(())
}

fn main() {
    // Any path matches /status/{status_code}
    #[rustfmt::skip]
    let router = Router::new()
        .at("status")
            .at("{status_code}").uses(m!(return_status_code)).done()
        .done();

    let app = amiya::new().uses(router);

    smol::block_on(app.listen("[::]:8080")).unwrap();
}

// visit /status/200 => http status 200
// visit /status/502 => http status 502
// ... etc ...
// visit /status/<can not convert to a valid http status code> => http status 400(Bad Request)
// visit other path => http status 404
