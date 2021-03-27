use {
    amiya::{m, Error},
    std::time::Duration,
};

fn main() {
    env_logger::init();

    // Only this stmt is Amiya related code, it sets response to some hello world texts
    let app = amiya::new().uses(m!(ctx =>
        Err(Error::from_str(500, "o_O"))
    ));

    let stop = app.listen("[::]:8080").unwrap();

    std::thread::sleep(Duration::from_secs(10));

    let _ = stop.try_send(());

    std::thread::sleep(Duration::from_secs(1));
}

/*
[2021-03-27T14:45:34Z INFO  amiya] Amiya server start listening Ok([::]:8080)
[2021-03-27T14:45:37Z ERROR amiya] Request handle error: code = 500, type = Unknown, detail = o_O
[2021-03-27T14:45:44Z INFO  amiya] Amiya server stop listening Ok([::]:8080)
*/
