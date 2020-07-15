use {
    smol,
    futures,
    http_types::Request,
    amiya::{AmiyaBuilder, BoxedNextFunc, BoxedRespFut},
};

 fn log(req: Request, next: BoxedNextFunc<'_>) -> BoxedRespFut {
     Box::pin(async {
         println!("Request {} from {}", req.url(), req.remote().unwrap_or("unknown address"));
         let resp = next(req).await;
         if let Err(err) = resp.as_ref() {
             eprintln!("Request process error: {}", err);
         }
         resp
     })
}

fn response(req: Request, next: BoxedNextFunc<'_>) -> BoxedRespFut {
    Box::pin(async {
        let mut resp = next(req).await;
        if let Ok(ref mut resp) = resp {
            resp.set_body("Hello from Amiya!");
        }
        resp
    })
}

fn main() {
    for _ in 0 .. num_cpus::get().max(1) {
        std::thread::spawn(|| smol::run(futures::future::pending::<()>()));
    }

    let amiya = AmiyaBuilder::default()
        .uses(log)
        .uses(response)
        .build();

    smol::block_on(amiya.listen("[::]:8080")).unwrap();
}
