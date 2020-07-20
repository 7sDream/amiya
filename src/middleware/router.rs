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

static ALL_METHODS: &'static [Method] = &[
    Method::Get,
    Method::Head,
    Method::Post,
    Method::Put,
    Method::Delete,
    Method::Connect,
    Method::Options,
    Method::Trace,
    Method::Patch,
];

macro_rules! impl_method {
    ($(#[$outer:meta])*
    $funcname: ident : $method: expr) => {
        $(#[$outer])*
        pub fn $funcname<M: Middleware<Ex> + 'static>(self, middleware: M) -> Self {
            self.method($method, middleware)
        }
    };

    ($($(#[$outer:meta])* $funcname: ident : $method: expr),* $(,)?) => {
        $(impl_method!{$(#[$outer])* $funcname: $method})+
    };
}

macro_rules! impl_methods {
    ($(#[$outer:meta])* $funcname: ident : $methods: expr) => {
        $(#[$outer])*
        pub fn $funcname<M: Middleware<Ex> + 'static>(self, middleware: M) -> Self {
            self.methods($methods, middleware)
        }
    };

    ($($(#[$outer:meta])* $funcname: ident : $methods: expr),* $(,)?) => {
        $(impl_methods!{$(#[$outer])* $funcname: $methods})+
    };
}

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
    pub fn method<M: Middleware<Ex> + 'static>(mut self, method: Method, middleware: M) -> Self {
        let middleware: Arc<dyn Middleware<Ex>> = Arc::new(middleware);
        self.table.insert(method, Arc::clone(&middleware));
        self
    }

    pub fn methods<H: AsRef<[Method]>, M: Middleware<Ex> + 'static>(
        mut self, methods: H, middleware: M,
    ) -> Self {
        let middleware: Arc<dyn Middleware<Ex>> = Arc::new(middleware);
        methods.as_ref().iter().for_each(|method| {
            self.table.insert(*method, Arc::clone(&middleware));
        });
        self
    }

    impl_method! {
        get: Method::Get,
        head: Method::Head,
        post: Method::Post,
        put: Method::Put,
        delete: Method::Delete,
        connect: Method::Connect,
        options: Method::Options,
        trace: Method::Trace,
        patch: Method::Patch,
    }

    impl_methods! {
        all: ALL_METHODS,
    }
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

#[allow(missing_debug_implementations)]
pub struct Router<Ex> {
    endpoint: Option<Box<dyn Middleware<Ex>>>,
    fallback: Option<Box<dyn Middleware<Ex>>>,
    table: HashMap<Cow<'static, str>, Box<dyn Middleware<Ex>>>,
}

impl<Ex> AsMut<Self> for Router<Ex> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<Ex> Default for Router<Ex> {
    fn default() -> Self {
        Self { endpoint: None, fallback: None, table: HashMap::new() }
    }
}

impl<Ex> Router<Ex>
where
    Ex: Send + Sync + 'static,
{
    pub fn sub_middleware<P, M>(mut self, path: P, middleware: M) -> Self
    where
        P: Into<Cow<'static, str>>,
        M: Middleware<Ex> + 'static,
    {
        let path = path.into();
        let middleware = Box::new(middleware);
        if path.is_empty() {
            self.fallback.replace(middleware);
        } else {
            self.table.insert(path, middleware);
        }
        self
    }

    pub fn sub_router<P, F>(self, path: P, f: F) -> Self
    where
        P: Into<Cow<'static, str>>,
        F: FnOnce(Router<Ex>) -> Self,
    {
        let sub_router = Router::default();
        let sub_router = f(sub_router);
        self.sub_middleware(path, sub_router)
    }

    pub fn sub_by_method<P, F>(self, path: P, f: F) -> Self
    where
        P: Into<Cow<'static, str>>,
        F: FnOnce(MethodRouter<Ex>) -> MethodRouter<Ex>,
    {
        let method_router = MethodRouter::default();
        let method_router = f(method_router);
        self.sub_middleware(path, method_router)
    }

    pub fn sub_endpoint<P, M>(self, path: P, middleware: M) -> Self
    where
        P: Into<Cow<'static, str>>,
        M: Middleware<Ex> + 'static,
    {
        self.sub_router(path, |router| router.endpoint(middleware))
    }

    pub fn sub_endpoint_by_method_router<P, F>(self, path: P, f: F) -> Self
    where
        P: Into<Cow<'static, str>>,
        F: FnOnce(MethodRouter<Ex>) -> MethodRouter<Ex>,
    {
        let method_router = MethodRouter::default();
        let method_router = f(method_router);
        self.sub_endpoint(path, method_router)
    }

    pub fn sub_endpoint_by_method<P, M>(self, path: P, method: Method, middleware: M) -> Self
    where
        P: Into<Cow<'static, str>>,
        M: Middleware<Ex> + 'static,
    {
        self.sub_endpoint_by_method_router(path, move |mrouter| mrouter.method(method, middleware))
    }

    pub fn fallback<M: Middleware<Ex> + 'static>(mut self, middleware: M) -> Self {
        self.fallback.replace(Box::new(middleware));
        self
    }

    pub fn fallback_by_method_router<F>(self, f: F) -> Self
    where
        F: FnOnce(MethodRouter<Ex>) -> MethodRouter<Ex>,
    {
        let method_router = MethodRouter::default();
        let method_router = f(method_router);
        self.fallback(method_router)
    }

    pub fn fallback_by_method<M>(self, method: Method, middleware: M) -> Self
    where
        M: Middleware<Ex> + 'static,
    {
        self.fallback_by_method_router(move |mrouter| mrouter.method(method, middleware))
    }

    pub fn endpoint_by_method<F>(self, f: F) -> Self
    where
        F: FnOnce(MethodRouter<Ex>) -> MethodRouter<Ex>,
    {
        let method_router = MethodRouter::default();
        let method_router = f(method_router);
        self.endpoint(method_router)
    }

    pub fn endpoint<M>(mut self, middleware: M) -> Self
    where
        M: Middleware<Ex> + 'static,
    {
        self.endpoint.replace(Box::new(middleware));
        self
    }
}

#[async_trait]
impl<Ex> Middleware<Ex> for Router<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, mut ctx: Context<'_, Ex>) -> Result<()> {
        if ctx.remain_path.is_empty() {
            if let Some(ref endpoint) = self.endpoint {
                return endpoint.handle(ctx).await;
            }
        } else {
            for (target_path, sub_router) in &self.table {
                if ctx.remain_path.starts_with(target_path.as_ref()) {
                    if ctx.remain_path.len() == target_path.len() {
                        ctx.remain_path = "";
                    } else if ctx.remain_path[target_path.len()..].starts_with('/') {
                        ctx.remain_path = &ctx.remain_path[(target_path.len() + 1)..];
                    } else {
                        continue;
                    }
                    return sub_router.handle(ctx).await;
                }
            }

            if let Some(ref fallback) = self.fallback {
                return fallback.handle(ctx).await;
            }
        }

        ctx.resp.set_status(StatusCode::NotFound);
        Ok(())
    }
}
