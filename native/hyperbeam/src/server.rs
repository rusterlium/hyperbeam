use crate::atoms;
use futures::sync::oneshot;
use futures::Future;
use hyper::rt;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use rustler::{Encoder, Env, Error, OwnedEnv, ResourceArc, Term};
use std::sync::Mutex;

//pub(crate) struct BodyChannel(Mutex<Option<oneshot::Sender<()>>>);
pub(crate) struct ResponseChannel(Mutex<Option<oneshot::Sender<String>>>);
pub(crate) struct ShutdownChannel(Mutex<Option<oneshot::Sender<()>>>);

#[derive(NifMap)]
struct Req<'a> {
    path: &'a str,
    host: Option<&'a str>,
    port: Option<u16>,
    method: &'a str,
    headers: Vec<(&'a str, Vec<u8>)>,
    qs: Option<&'a str>,
    resource: ResourceArc<ResponseChannel>,
}

pub fn start<'a>(env: Env<'a>, _terms: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let (shutdown_tx, shutdown_rx) = futures::sync::oneshot::channel::<()>();

    let pid = env.pid();

    std::thread::spawn(move || {
        let addr = ([127, 0, 0, 1], 3000).into();

        let server = Server::bind(&addr)
            .serve(move || {
                let pid = pid.clone();

                service_fn(move |req: Request<Body>| {
                    let mut env = OwnedEnv::new();
                    let (tx, rx) = oneshot::channel::<String>();

                    let uri = req.uri();

                    let headers: Vec<(&str, Vec<u8>)> = req
                        .headers()
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_bytes().to_vec()))
                        .collect();

                    env.send_and_clear(&pid, |env| {
                        let request = Req {
                            path: uri.path(),
                            host: uri.host(),
                            port: uri.port_u16(),
                            method: req.method().as_str(),
                            headers: headers,
                            qs: uri.query(),
                            resource: ResourceArc::new(ResponseChannel(Mutex::new(Some(tx)))),
                        };

                        (atoms::request(), request).encode(env)
                    });

                    rx.and_then(|s| futures::future::ok(Response::new(Body::from(s))))
                })
            })
            .with_graceful_shutdown(shutdown_rx)
            .map_err(|e| eprintln!("server error: {}", e));

        rt::run(server)
    });

    let resource = ResourceArc::new(ShutdownChannel(Mutex::new(Some(shutdown_tx))));
    Ok((atoms::ok(), resource).encode(env))
}

pub fn stop<'a>(env: Env<'a>, terms: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let resource: ResourceArc<ShutdownChannel> = terms[0].decode()?;
    let mut lock = resource.0.lock().unwrap();

    if let Some(tx) = lock.take() {
        tx.send(()).unwrap()
    }

    Ok(atoms::ok().encode(env))
}

pub fn send_resp<'a>(env: Env<'a>, terms: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let resource: ResourceArc<ResponseChannel> = terms[0].decode()?;
    let mut lock = resource.0.lock().unwrap();

    let body: String = terms[1].decode()?;

    if let Some(tx) = lock.take() {
        tx.send(body).unwrap()
    }

    Ok(atoms::ok().encode(env))
}
