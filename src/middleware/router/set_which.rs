use {
    crate::{middleware::router::RouterLike, Middleware},
    std::borrow::Cow,
};

pub trait SetWhich<Ex> {
    fn set_to_target<R, M>(self, router: R, middleware: M) -> R
    where
        R: RouterLike<Ex>,
        M: Middleware<Ex> + 'static;
}

#[derive(Debug)]
pub struct SetEndpoint {}

impl<Ex> SetWhich<Ex> for SetEndpoint {
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
pub struct SetFallback {}

impl<Ex> SetWhich<Ex> for SetFallback {
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
pub struct SetTableItem {
    pub path: Cow<'static, str>,
}

impl<Ex> SetWhich<Ex> for SetTableItem {
    fn set_to_target<R, M>(self, mut router: R, middleware: M) -> R
    where
        R: RouterLike<Ex>,
        M: Middleware<Ex> + 'static,
    {
        router.insert_to_router_table(self.path, middleware);
        router
    }
}
