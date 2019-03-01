#[macro_use] extern crate rustler_codegen;
use rustler::{Env, Term};

mod atoms;
mod server;

rustler::rustler_export_nifs! {
    "Elixir.Hyper.Native",
    [
        ("start", 1, server::start),
        ("stop", 1, server::stop),
        ("send_resp", 2, server::send_resp),
        ("batch_read", 1, server::batch_read),
    ],
    Some(load)
}

fn load<'a>(env: Env<'a>, _: Term<'a>) -> bool {
    rustler::resource_struct_init!(server::ResponseChannel, env);
    rustler::resource_struct_init!(server::Select, env);
    rustler::resource_struct_init!(server::ShutdownChannel, env);

    true
}
