use super::environment_configuration::EnvironmentConfiguration;
use std::future::Future;
use super::error::{
    Error,
    ResultConverter,
    Backtrace,
};
use super::spawner::Spawner;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{
    body::Incoming,
    Request,
    Response,
};
use super::environment_configuration::Trade;
use super::error::{
    OptionConverter,
    Common,
};
use super::workflow_data::{
    TransactionDifferentiation,
    WorkflowData,
};
use yellowstone_grpc_proto::geyser::{SubscribeUpdateAccount, SubscribeUpdateTransaction};

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
pub struct HttpServer;
impl HttpServer {
    pub fn run(environment_configuration: &'static EnvironmentConfiguration<Trade>) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            let tcp_listener = TcpListener::bind(&environment_configuration.subject.http_server.tcp_socket_address)
            .await
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let builder = Builder::new(TokioExecutor::new());
            let service_fn = hyper::service::service_fn(
                move |request: Request<Incoming>| -> _ {
                    return async move {
                        let response = Self::process_request(
                            request,
                            environment_configuration,
                        )
                        .await;
                        return Result::<_, Error>::Ok(response);
                    };
                },
            );
            'a: loop {
                let tcp_stream = match tcp_listener.accept().await {
                    Ok((tcp_stream_, _)) => tcp_stream_,
                    Err(_) => {
                        continue 'a;
                    }
                };
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
    fn process_request(
        request: Request<Incoming>,
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
    ) -> impl Future<Output = Response<Full<Bytes>>> + Send {
        return async move {
            todo!()
            // let (parts, mut incoming) = request.into_parts();
            // let mut action_inner = ActionInner {
            //     incoming: &mut incoming,
            //     parts: &parts,
            // };
            // let r#match = match cloned.router.at(parts.uri.path()) {
            //     Result::Ok(r#match_) => r#match_,
            //     Result::Err(_) => {
            //         return Action::<RouteNotFound>::run(&action_inner);
            //     }
            // };
            // return Action::<RouteNotFound>::run(&action_inner);
        };
    }
}