use {
    crate::{
        impl_all_http_method, impl_method, impl_router_like_pub_fn,
        middleware::router::{
            set_which::{SetEndpoint, SetFallback, SetTableItem, SetWhich},
            MethodRouter, Router, RouterLike,
        },
        Method, Middleware,
    },
    std::borrow::Cow,
};

/// Router editing environment.
///
/// You don’t need to understand this type, it's here just for list it's function.
/// See [`Router`] document for api design and usage.
///
/// [`Router`]: struct.Router.html
#[allow(missing_debug_implementations)]
pub struct RouterSetter<R, Sw, Ex> {
    router: R,
    sub_router: Router<Ex>,
    method_router: MethodRouter<Ex>,
    setter: Sw,
}

impl<R, Ex> RouterSetter<R, SetEndpoint, Ex>
where
    R: RouterLike<Ex>,
{
    #[doc(hidden)]
    pub fn new_endpoint_setter(router: R) -> Self {
        Self {
            router,
            method_router: MethodRouter::default(),
            sub_router: Router::default(),
            setter: SetEndpoint {},
        }
    }
}

impl<R, Ex> RouterSetter<R, SetFallback, Ex>
where
    R: RouterLike<Ex>,
{
    #[doc(hidden)]
    pub fn new_fallback_setter(router: R) -> Self {
        Self {
            router,
            method_router: MethodRouter::default(),
            sub_router: Router::default(),
            setter: SetFallback {},
        }
    }
}

#[allow(clippy::use_self)]
impl<R, Ex> RouterSetter<R, SetTableItem, Ex>
where
    R: RouterLike<Ex>,
{
    #[doc(hidden)]
    pub fn new_router_table_setter<P: Into<Cow<'static, str>>>(router: R, path: P) -> Self {
        Self {
            router,
            method_router: MethodRouter::default(),
            sub_router: Router::default(),
            setter: SetTableItem { path: path.into() },
        }
    }

    impl_router_like_pub_fn! { Ex }

    /// Finish this router table editing.
    pub fn done(self) -> R
    where
        Ex: Send + Sync + 'static,
    {
        self.setter.set_to_target(self.router, self.sub_router)
    }
}

impl<R, Ex> RouterLike<Ex> for RouterSetter<R, SetTableItem, Ex>
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

#[allow(clippy::use_self)]
impl<R, Ex> RouterSetter<RouterSetter<R, SetTableItem, Ex>, SetEndpoint, Ex>
where
    R: RouterLike<Ex>,
    Ex: Send + Sync + 'static,
{
    /// Change to fallback editing environment.
    pub fn fallback(self) -> RouterSetter<RouterSetter<R, SetTableItem, Ex>, SetFallback, Ex> {
        self.router.fallback()
    }

    /// Change to inner router's router table table editing environment.
    #[allow(clippy::type_complexity)] // it's api design, not use this type directly
    pub fn at<P: Into<Cow<'static, str>>>(
        self, path: P,
    ) -> RouterSetter<
        RouterSetter<RouterSetter<R, SetTableItem, Ex>, SetTableItem, Ex>,
        SetEndpoint,
        Ex,
    > {
        self.router.at(path)
    }

    /// Finish setting uses `middleware`.
    pub fn is<M: Middleware<Ex> + 'static>(self, middleware: M) -> R {
        self.router.setter.set_to_target(self.router.router, middleware)
    }
}

impl<R, Sw, Ex> RouterSetter<R, Sw, Ex>
where
    R: RouterLike<Ex>,
    Sw: SetWhich<Ex>,
    Ex: Send + Sync + 'static,
{
    /// Finish editing use a method router which accept `method` and uses `middleware`.
    pub fn method<M: Middleware<Ex> + 'static>(self, method: Method, middleware: M) -> R {
        self.setter.set_to_target(self.router, self.method_router.method(method, middleware))
    }

    impl_all_http_method! { R }

    /// Finish editing use a middleware.
    pub fn uses<M: Middleware<Ex> + 'static>(self, middleware: M) -> R {
        self.setter.set_to_target(self.router, middleware)
    }
}
