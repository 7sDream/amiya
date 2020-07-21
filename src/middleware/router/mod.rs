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
    $func_name: ident : $method: expr => $ret: ty) => {
        $(#[$outer])*
        pub fn $func_name<M: Middleware<Ex> + 'static>(self, middleware: M) -> $ret {
            self.method($method, middleware)
        }
    };

    ($($(#[$outer:meta])*
    $func_name: ident : $method: expr),* $(,)? => $ret: ty) => {
        $(impl_method!{$(#[$outer])* $func_name: $method => $ret})+
    };
}

macro_rules! impl_all_http_method {
    ($ret: ty) => {
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
            => $ret
        }
    };
}

macro_rules! impl_methods {
    ($(#[$outer:meta])* $func_name: ident : $methods: expr) => {
        $(#[$outer])*
        pub fn $func_name<M: Middleware<Ex> + 'static>(self, middleware: M) -> Self {
            self.methods($methods, middleware)
        }
    };

    ($($(#[$outer:meta])* $func_name: ident : $methods: expr),* $(,)?) => {
        $(impl_methods!{$(#[$outer])* $func_name: $methods})+
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

impl<Ex> MethodRouter<Ex> {
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

    impl_all_http_method! { Self }

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

pub trait RouterLike<Ex>: Sized {
    fn set_endpoint<M: Middleware<Ex> + 'static>(&mut self, middleware: M);
    fn set_fallback<M: Middleware<Ex> + 'static>(&mut self, middleware: M);
    fn insert_to_router_table<P: Into<Cow<'static, str>>, M: Middleware<Ex> + 'static>(
        &mut self, path: P, middleware: M,
    );
}

macro_rules! impl_router_like_pub_fn {
    ($ex: ty) => {
        pub fn endpoint(self) -> RouterSetter<Self, SetToEndpoint, $ex> {
            RouterSetter::new_endpoint_setter(self)
        }

        pub fn at<P: Into<Cow<'static, str>>>(
            self, path: P,
        ) -> RouterSetter<RouterSetter<Self, SetToRouterTable, $ex>, SetToEndpoint, $ex> {
            RouterSetter::new_router_table_setter(self, path).endpoint()
        }

        pub fn fallback(self) -> RouterSetter<Self, SetToFallback, $ex> {
            RouterSetter::new_fallback_setter(self)
        }
    };
}

#[allow(missing_debug_implementations)]
pub struct Router<Ex> {
    endpoint: Option<Box<dyn Middleware<Ex>>>,
    fallback: Option<Box<dyn Middleware<Ex>>>,
    table: HashMap<Cow<'static, str>, Box<dyn Middleware<Ex>>>,
}

impl<Ex> Default for Router<Ex> {
    fn default() -> Self {
        Self { endpoint: None, fallback: None, table: HashMap::new() }
    }
}

impl<Ex> RouterLike<Ex> for Router<Ex> {
    fn set_endpoint<M: Middleware<Ex> + 'static>(&mut self, middleware: M) {
        self.endpoint.replace(Box::new(middleware));
    }

    fn set_fallback<M: Middleware<Ex> + 'static>(&mut self, middleware: M) {
        self.fallback.replace(Box::new(middleware));
    }

    fn insert_to_router_table<P: Into<Cow<'static, str>>, M: Middleware<Ex> + 'static>(
        &mut self, path: P, middleware: M,
    ) {
        self.table.insert(path.into(), Box::new(middleware));
    }
}

impl<Ex> Router<Ex> {
    pub fn new() -> Self {
        Self::default()
    }

    impl_router_like_pub_fn! { Ex }
}

pub fn router<Ex>() -> Router<Ex> {
    Router::default()
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

pub trait SetToWitch<Ex> {
    fn set_to_target<R, M>(self, router: R, middleware: M) -> R
    where
        R: RouterLike<Ex>,
        M: Middleware<Ex> + 'static;
}

#[derive(Debug)]
pub struct SetToEndpoint {}

impl<Ex> SetToWitch<Ex> for SetToEndpoint {
    fn set_to_target<R, M>(self, mut router: R, middleware: M) -> R
    where
        R: RouterLike<Ex>,
        M: Middleware<Ex> + 'static,
    {
        router.set_endpoint(middleware);
        router
    }
}

#[derive(Debug)]
pub struct SetToFallback {}

impl<Ex> SetToWitch<Ex> for SetToFallback {
    fn set_to_target<R, M>(self, mut router: R, middleware: M) -> R
    where
        R: RouterLike<Ex>,
        M: Middleware<Ex> + 'static,
    {
        router.set_fallback(middleware);
        router
    }
}

#[derive(Debug)]
pub struct SetToRouterTable {
    path: Cow<'static, str>,
}

impl<Ex> SetToWitch<Ex> for SetToRouterTable {
    fn set_to_target<R, M>(self, mut router: R, middleware: M) -> R
    where
        R: RouterLike<Ex>,
        M: Middleware<Ex> + 'static,
    {
        router.insert_to_router_table(self.path, middleware);
        router
    }
}

#[allow(missing_debug_implementations)]
pub struct RouterSetter<R, Sw, Ex> {
    router: R,
    sub_router: Router<Ex>,
    method_router: MethodRouter<Ex>,
    setter: Sw,
}

impl<R, Ex> RouterSetter<R, SetToEndpoint, Ex>
where
    R: RouterLike<Ex>,
{
    fn new_endpoint_setter(router: R) -> Self {
        Self {
            router,
            method_router: MethodRouter::default(),
            sub_router: Router::default(),
            setter: SetToEndpoint {},
        }
    }
}

impl<R, Ex> RouterSetter<R, SetToFallback, Ex>
where
    R: RouterLike<Ex>,
{
    fn new_fallback_setter(router: R) -> Self {
        Self {
            router,
            method_router: MethodRouter::default(),
            sub_router: Router::default(),
            setter: SetToFallback {},
        }
    }
}

impl<R, Ex> RouterSetter<R, SetToRouterTable, Ex>
where
    R: RouterLike<Ex>,
{
    fn new_router_table_setter<P: Into<Cow<'static, str>>>(router: R, path: P) -> Self {
        Self {
            router,
            method_router: MethodRouter::default(),
            sub_router: Router::default(),
            setter: SetToRouterTable { path: path.into() },
        }
    }

    impl_router_like_pub_fn! { Ex }

    pub fn done(self) -> R
    where
        Ex: Send + Sync + 'static,
    {
        self.setter.set_to_target(self.router, self.sub_router)
    }
}

impl<R, Ex> RouterLike<Ex> for RouterSetter<R, SetToRouterTable, Ex>
where
    R: RouterLike<Ex>,
{
    fn set_endpoint<M: Middleware<Ex> + 'static>(&mut self, middleware: M) {
        self.sub_router.set_endpoint(middleware);
    }

    fn set_fallback<M: Middleware<Ex> + 'static>(&mut self, middleware: M) {
        self.sub_router.set_fallback(middleware);
    }

    fn insert_to_router_table<P: Into<Cow<'static, str>>, M: Middleware<Ex> + 'static>(
        &mut self, path: P, middleware: M,
    ) {
        self.sub_router.insert_to_router_table(path, middleware);
    }
}

impl<R, Ex> RouterSetter<RouterSetter<R, SetToRouterTable, Ex>, SetToEndpoint, Ex>
where
    R: RouterLike<Ex>,
    Ex: Send + Sync + 'static,
{
    pub fn fallback(
        self,
    ) -> RouterSetter<RouterSetter<R, SetToRouterTable, Ex>, SetToFallback, Ex> {
        self.router.fallback()
    }

    pub fn at<P: Into<Cow<'static, str>>>(
        self, path: P,
    ) -> RouterSetter<
        RouterSetter<RouterSetter<R, SetToRouterTable, Ex>, SetToRouterTable, Ex>,
        SetToEndpoint,
        Ex,
    > {
        self.router.at(path)
    }

    pub fn is<M: Middleware<Ex> + 'static>(self, middleware: M) -> R {
        self.router.setter.set_to_target(self.router.router, middleware)
    }
}

impl<R, Sw, Ex> RouterSetter<R, Sw, Ex>
where
    R: RouterLike<Ex>,
    Sw: SetToWitch<Ex>,
    Ex: Send + Sync + 'static,
{
    pub fn method<M: Middleware<Ex> + 'static>(self, method: Method, middleware: M) -> R {
        self.setter.set_to_target(self.router, self.method_router.method(method, middleware))
    }

    impl_all_http_method! { R }

    pub fn uses<M: Middleware<Ex> + 'static>(self, middleware: M) -> R {
        self.setter.set_to_target(self.router, middleware)
    }
}