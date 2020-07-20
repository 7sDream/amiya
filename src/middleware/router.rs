use {
    super::Middleware,
    crate::{Context, Result, StatusCode},
    async_trait::async_trait,
    http_types::Method,
    std::{
        borrow::Cow,
        collections::HashMap,
        fmt::{self, Debug, Formatter},
        sync::Arc,
    },
};

pub struct MethodRouter<Ex> {
    table: HashMap<Method, Arc<dyn Middleware<Ex>>>,
}

impl<Ex> Default for MethodRouter<Ex> {
    fn default() -> Self {
        Self { table: HashMap::new() }
    }
}

impl<Ex> Debug for MethodRouter<Ex> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Method Router { ")?;
        for method in self.table.keys() {
            <Method as Debug>::fmt(method, f)?;
            f.write_str(" ")?;
        }
        f.write_str("}")
    }
}

impl<Ex> MethodRouter<Ex>
where
    Ex: Send + Sync + 'static,
{
    pub fn methods<H: AsRef<[Method]>, M: Middleware<Ex> + 'static>(
        &mut self, methods: H, middleware: M,
    ) -> &mut Self {
        let middleware: Arc<dyn Middleware<Ex>> = Arc::new(middleware);
        methods.as_ref().iter().for_each(|method| {
            self.table.insert(*method, Arc::clone(&middleware));
        });
        self
    }

    pub fn get<M: Middleware<Ex> + 'static>(&mut self, middleware: M) -> &mut Self {
        self.methods([Method::Get], middleware)
    }

    // TODO: other HTTP methods
}

#[async_trait]
impl<Ex> Middleware<Ex> for MethodRouter<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result<()> {
        if let Some(middleware) = self.table.get(&ctx.req.method()) {
            middleware.handle(ctx).await
        } else {
            ctx.resp.set_status(StatusCode::MethodNotAllowed);
            ctx.resp.take_body();
            Ok(())
        }
    }
}

struct RouteMatchStartPos(usize);

#[allow(missing_debug_implementations)]
pub struct Router<Ex> {
    table: HashMap<Cow<'static, str>, Box<dyn Middleware<Ex>>>,
}

impl<Ex> Default for Router<Ex> {
    fn default() -> Self {
        Self { table: HashMap::new() }
    }
}

impl<Ex> Router<Ex> {
    pub fn route<P: Into<Cow<'static, str>>>(self, path: P) -> PathRouterAdder<Ex> {
        PathRouterAdder { path: path.into(), router: self, method_router: MethodRouter::default() }
    }
}

#[async_trait]
impl<Ex> Middleware<Ex> for Router<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result<()> {
        let mut match_pos = ctx.resp.ext().get::<RouteMatchStartPos>().map(|x| x.0).unwrap_or(1);
        let path = ctx.req.url().path();

        if match_pos < path.len() {
            let remain_path = &path[match_pos..];

            for (target_path, endpoint) in &self.table {
                if remain_path.starts_with(target_path.as_ref()) {
                    if remain_path.len() == target_path.len() {
                        match_pos = path.len();
                    } else if remain_path[path.len()..].starts_with('/') {
                        match_pos += target_path.len() + 1;
                    } else {
                        continue;
                    }
                    ctx.resp.ext_mut().insert(RouteMatchStartPos(match_pos));
                    return endpoint.handle(ctx).await;
                }
            }
        }

        ctx.resp.set_status(StatusCode::NotFound);
        Ok(())
    }
}

#[allow(missing_debug_implementations)]
pub struct PathRouterAdder<Ex> {
    path: Cow<'static, str>,
    router: Router<Ex>,
    method_router: MethodRouter<Ex>,
}

impl<Ex> AsMut<MethodRouter<Ex>> for PathRouterAdder<Ex> {
    fn as_mut(&mut self) -> &mut MethodRouter<Ex> {
        &mut self.method_router
    }
}

impl<Ex> PathRouterAdder<Ex>
where
    Ex: Send + Sync,
{
    pub fn uses<M: Middleware<Ex> + 'static>(mut self, middleware: M) -> Router<Ex> {
        self.router.table.insert(self.path, Box::new(middleware));
        self.router
    }

    pub fn route<F: FnOnce(Router<Ex>) -> Router<Ex>>(self, f: F) -> Router<Ex>
    where
        Ex: 'static,
    {
        let router = Router::default();
        f(router)
    }

    pub fn done(mut self) -> Router<Ex>
    where
        Ex: 'static,
    {
        self.router.table.insert(self.path, Box::new(self.method_router));
        self.router
    }
}
