use {
    amiya::{m, middleware::Router, Context, Result, StatusCode},
    serde::{Deserialize, Serialize},
    serde_json::{Map, Value},
};

async fn parse_query_object(mut ctx: Context<'_, ()>) -> Result {
    let qm: Map<String, Value> = ctx.req.query()?;

    ctx.next().await?;

    ctx.resp.set_body(Value::Object(qm));

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchQuery {
    keyword: String,
    city: Option<String>,
    offset: Option<usize>,
}

async fn parse_query_struct(mut ctx: Context<'_, ()>) -> Result {
    let query: SearchQuery = if let Ok(query) = ctx.req.query() {
        query
    } else {
        ctx.resp.set_status(StatusCode::BadRequest);
        return Ok(());
    };

    ctx.next().await?;

    ctx.resp.set_body(serde_json::to_value(query)?);

    Ok(())
}

fn main() {
    #[rustfmt::skip]
    let router = Router::new()
        .at("object").get(m!(parse_query_object)).done()
        .at("struct").get(m!(parse_query_struct)).done();

    let app = amiya::new().uses(router);

    app.listen("[::]:8080").unwrap();

    std::thread::park();
}

// $ curl 'http://127.0.0.1:8080/object?key=value&arr[]=a1&c=d&arr[]=a2&object[one]=1&object[two]=2'
// {"arr":["a1","a2"],"c":"d","key":"value","object":{"one":"1","two":"2"}}

// $ curl 'http://127.0.0.1:8080/struct?keyword=hello&city=beijing&notexist=haha'
// {"keyword":"hello","city":"beijing","offset":null}

// $ curl 'http://127.0.0.1:8080/struct?keyword=hello&city=beijing&offset=20'
// {"keyword":"hello","city":"beijing","offset":20}
