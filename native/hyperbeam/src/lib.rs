mod atoms;
mod server;

rustler::init! {
    "Elixir.Hyperbeam.Native",
    [
        server::start,
        server::stop,
        server::send_resp,
        server::batch_read,
    ],
    load = server::load
}
