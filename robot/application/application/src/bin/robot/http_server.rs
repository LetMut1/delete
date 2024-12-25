use super::environment_configuration::EnvironmentConfiguration;
use std::future::Future;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::Arc;
use super::error::{
    Error,
    ResultConverter,
    Backtrace,
};
use super::spawner::Spawner;
use bytes::{
    Buf,
    Bytes,
};
use http_body_util::Full;
use hyper::{
    body::Incoming,
    Request,
    Response,
};
use super::environment_configuration::Trade;
use hyper::{
    server::conn::http2::Builder,
    Method,
};
use hyper_util::rt::{
    tokio::TokioExecutor,
    TokioIo,
};
use matchit::Router;
use tokio::net::TcpListener;
use http::{
    header,
    HeaderMap,
    HeaderValue,
    StatusCode,
    Version,
};
use std::convert::From;
use http_body_util::BodyExt;
pub struct HttpServer;
impl HttpServer {
    pub fn run(
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
        is_graceful_shutdown_command_received: &'static AtomicBool,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let router = Arc::new(Self::create_router()?);
            let tcp_listener = TcpListener::bind(&environment_configuration.subject.http_server.tcp_socket_address)
            .await
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let builder = Builder::new(TokioExecutor::new());
            'a: loop {
                let tcp_stream = match tcp_listener.accept().await {
                    Ok((tcp_stream_, _)) => tcp_stream_,
                    Err(_) => {
                        continue 'a;
                    }
                };
                let router_ = router.clone();
                let service_fn = hyper::service::service_fn(
                    move |request: Request<Incoming>| -> _ {
                        let router__ = router_.clone();
                        return async move {
                            let response = Self::process_request(
                                environment_configuration,
                                is_graceful_shutdown_command_received,
                                request,
                                router__,
                            )
                            .await;
                            return Result::<_, Error>::Ok(response);
                        };
                    },
                );
                let serve_connection_future = builder.serve_connection(
                    TokioIo::new(tcp_stream),
                    service_fn,
                );
                Spawner::spawn_tokio_non_blocking_task_into_background(
                    async move {
                        serve_connection_future.await.into_(
                            Backtrace::new(
                                line!(),
                                file!(),
                            ),
                        )
                    }
                );
            }
            Ok(())
        }
    }
    fn create_router() -> Result<Router<()>, Error> {
        let mut router = Router::<()>::new();
        router.insert("/robot", ())
        .into_(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?;
        Ok(router)
    }
    fn process_request(
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
        is_graceful_shutdown_command_received: &'static AtomicBool,
        request: Request<Incoming>,
        router: Arc<Router<()>>,
    ) -> impl Future<Output = Response<Full<Bytes>>> + Send {
        return async move {
            let (parts, mut incoming) = request.into_parts();
            if let Err(_) = router.at(parts.uri.path()) {
                return ResponseCreator::create_not_found();
            }
            if let Method::POST = parts.method {
                let collected = match incoming.collect().await {
                    Ok(collected_) => collected_,
                    Err(_) => {
                        return ResponseCreator::create_internal_server_error();
                    }
                };
                let command = match serde_json::de::from_slice::<'_, Command>(collected.aggregate().chunk()) {
                    Ok(command_) => command_,
                    Err(_) => {
                        return ResponseCreator::create_bad_request();
                    }
                };
                let data = match command {
                    Command::GracefulShutdown => {
                        if !is_graceful_shutdown_command_received.load(Ordering::Relaxed) {
                            is_graceful_shutdown_command_received.store(true, Ordering::Relaxed);
                            b"The process will not create new trading tasks and will end after all previous tasks have been completed.".to_vec()
                        } else {
                            b"The command has already been received. The process is waiting for previous traiding tasks to complete.".to_vec()
                        }
                    }
                };
                return ResponseCreator::create_ok(data);
            }
            return ResponseCreator::create_not_found();
        };
    }
}
struct ResponseCreator;
impl ResponseCreator {
    const HEADER_VALUE_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/octet-stream");
    fn create(status_code: StatusCode, data: Option<Vec<u8>>) -> Response<Full<Bytes>> {
        let mut header_map = HeaderMap::new();
        header_map.append(
            header::CONTENT_TYPE,
            Self::HEADER_VALUE_CONTENT_TYPE,
        );
        let mut parts = Response::new(()).into_parts().0;
        parts.status = status_code;
        parts.version = Version::HTTP_2;
        parts.headers = header_map;
        let bytes = match data {
            Option::Some(data_) => Bytes::from(data_),
            Option::None => Bytes::new(),
        };
        return Response::from_parts(
            parts,
            Full::new(bytes),
        );
    }
    fn create_bad_request() -> Response<Full<Bytes>> {
        return Self::create(
            StatusCode::BAD_REQUEST,
            Option::None,
        );
    }
    fn create_not_found() -> Response<Full<Bytes>> {
        return Self::create(
            StatusCode::NOT_FOUND,
            Option::None,
        );
    }
    fn create_internal_server_error() -> Response<Full<Bytes>> {
        return Self::create(
            StatusCode::INTERNAL_SERVER_ERROR,
            Option::None,
        );
    }
    fn create_ok(data: Vec<u8>) -> Response<Full<Bytes>> {
        return Self::create(
            StatusCode::OK,
            Option::Some(data),
        );
    }
}
#[derive(serde::Deserialize)]
enum Command {
    GracefulShutdown,
}