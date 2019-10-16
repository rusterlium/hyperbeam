use crate::atoms;
use futures::sync::oneshot;
use futures::Future;
use hyper::rt;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use rustler::{Encoder, Env, Error, NifMap as Map, OwnedEnv, ResourceArc, Term};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

struct ResponseChannel(Mutex<Option<oneshot::Sender<String>>>);
struct ShutdownChannel(Mutex<Option<oneshot::Sender<()>>>);
struct Select(Arc<AtomicBool>);

#[derive(Map)]
struct Req {
    path: String,
    host: Option<String>,
    port: Option<u16>,
    method: String,
    headers: Vec<(String, String)>,
    qs: Option<String>,
    resource: ResourceArc<ResponseChannel>,
}

pub fn start<'a>(env: Env<'a>, _terms: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let (shutdown_tx, shutdown_rx) = futures::sync::oneshot::channel::<()>();
    let select = Arc::new(AtomicBool::new(false));

    let pid = env.pid();
    let select_flag = Arc::clone(&select);

    std::thread::spawn(move || {
        let addr = ([127, 0, 0, 1], 3000).into();

        let queue = Arc::new(Mutex::new(VecDeque::new()));

        let server = Server::bind(&addr)
            .serve(move || {
                let pid = pid.clone();
                let queue = queue.clone();
                let select_flag = Arc::clone(&select_flag);

                service_fn(move |req: Request<Body>| {
                    let (tx, rx) = oneshot::channel::<String>();
                    let (parts, _body) = req.into_parts();

                    let mut lock = queue.lock().unwrap();
                    lock.push_back((parts, tx));

                    if select_flag.swap(false, Ordering::Relaxed) == true {
                        let mut env = OwnedEnv::new();

                        env.send_and_clear(&pid, move |env| {
                            let batch: Vec<Req> = lock
                                .drain(..)
                                .map(|(parts, tx)| {
                                    let uri = parts.uri.clone();
                                    let path = uri.path().to_owned();
                                    let host = uri.host().map(|h| h.to_owned());
                                    let port = uri.port_u16().to_owned();
                                    let qs = uri.query().map(|q| q.to_owned());
                                    let method = parts.method.as_str().to_owned();
                                    let resource =
                                        ResourceArc::new(ResponseChannel(Mutex::new(Some(tx))));

                                    let headers: Vec<(String, String)> = parts
                                        .headers
                                        .iter()
                                        .map(|(k, v)| {
                                            let value = String::from_utf8_lossy(v.as_bytes());
                                            (k.as_str().to_owned(), value.into_owned())
                                        })
                                        .collect();

                                    Req {
                                        path,
                                        host,
                                        port,
                                        method,
                                        headers,
                                        qs,
                                        resource,
                                    }
                                })
                                .collect();
                            (atoms::request(), batch).encode(env)
                        });
                    }

                    rx.and_then(|s| futures::future::ok(Response::new(Body::from(s))))
                })
            })
            .with_graceful_shutdown(shutdown_rx)
            .map_err(|e| eprintln!("server error: {}", e));

        rt::run(server)
    });

    let select_ref = ResourceArc::new(Select(Arc::clone(&select)));
    let shutdown_ref = ResourceArc::new(ShutdownChannel(Mutex::new(Some(shutdown_tx))));
    Ok((atoms::ok(), shutdown_ref, select_ref).encode(env))
}

pub fn stop<'a>(env: Env<'a>, terms: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let resource: ResourceArc<ShutdownChannel> = terms[0].decode()?;
    let mut lock = resource.0.lock().unwrap();

    if let Some(tx) = lock.take() {
        let _ = tx.send(());
    }

    Ok(atoms::ok().encode(env))
}

pub fn send_resp<'a>(env: Env<'a>, terms: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let resource: ResourceArc<ResponseChannel> = terms[0].decode()?;
    let mut lock = resource.0.lock().unwrap();

    let body: String = terms[1].decode()?;

    if let Some(tx) = lock.take() {
        let _ = tx.send(body);
    }

    Ok(atoms::ok().encode(env))
}

pub fn batch_read<'a>(env: Env<'a>, terms: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let resource: ResourceArc<Select> = terms[0].decode()?;

    resource.0.swap(true, Ordering::Relaxed);

    Ok((atoms::ok()).encode(env))
}

pub fn load(env: Env) -> bool {
    rustler::resource_struct_init!(ResponseChannel, env);
    rustler::resource_struct_init!(Select, env);
    rustler::resource_struct_init!(ShutdownChannel, env);
    true
}
