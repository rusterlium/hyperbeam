use rustler::{Env, Term};

mod atoms;
mod server;

rustler::rustler_export_nifs! {
    "Elixir.Hyperbeam.Native",
    [
        ("start", 1, server::start),
        ("stop", 1, server::stop),
        ("send_resp", 2, server::send_resp),
        ("batch_read", 1, server::batch_read),
    ],
    Some(load)
}

fn load(env: Env, _: Term) -> bool {
    server::load(env);
    true
}
