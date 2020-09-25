use {
    amiya::{m, middleware::Router, Context, Result, StatusCode},
    serde::{Deserialize, Serialize},
    serde_json::{Map, Value},
};

async fn parse_body_urlencoded(mut ctx: Context<'_, ()>) -> Result {
    if let Some(ct) = ctx.req.header("content-type") {
        if ct.as_str().starts_with("application/x-www-form-urlencoded") {
            let body: Map<String, Value> = ctx.body().unwrap().into_form().await?;
            ctx.resp.set_body(Value::Object(body));
            return Ok(());
        }
    }

    ctx.resp.set_status(StatusCode::BadRequest);
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct SendCommentBody {
    uid: String,
    attitude: Option<String>,
    comment: String,
}

async fn parse_body_struct(mut ctx: Context<'_, ()>) -> Result {
    if let Some(ct) = ctx.req.header("content-type") {
        if ct.as_str().starts_with("application/x-www-form-urlencoded") {
            let body: SendCommentBody = if let Ok(body) = ctx.body().unwrap().into_form().await {
                body
            } else {
                ctx.resp.set_status(StatusCode::BadRequest);
                return Ok(());
            };
            ctx.resp.set_body(serde_json::to_value(body)?);
            return Ok(());
        }
    }

    ctx.resp.set_status(StatusCode::BadRequest);
    Ok(())
}

fn main() {
    #[rustfmt::skip]
    let router = Router::new()
        .at("object").post(m!(parse_body_urlencoded)).done()
        .at("struct").post(m!(parse_body_struct)).done();

    let app = amiya::new().uses(router);

    smol::block_on(app.listen("[::]:8080")).unwrap();
}
