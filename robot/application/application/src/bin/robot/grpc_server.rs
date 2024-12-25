use tokio::sync::mpsc::Sender;
use crate::robot::ForAccountTracking2;
use super::environment_configuration::EnvironmentConfiguration;
use std::future::Future;
use super::error::{
    Error,
    ResultConverter,
    Backtrace,
};
use std::collections::HashMap;
use super::spawner::Spawner;
use super::robot::{
    Robot,
    ForAccountProcessing,
    ForAccountTracking1,
};
use super::environment_configuration::Trade;
use {
    futures::stream::StreamExt,
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::prelude::{
        subscribe_update::UpdateOneof,
        SubscribeRequest,
        SubscribeRequestFilterAccounts,
        SubscribeRequestFilterTransactions,
    },
};
pub struct GrpcServer;
impl GrpcServer {
    pub fn run(
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
        accumulate_trackable_account_sender: Sender<ForAccountTracking1>,
        remove_trackable_account_sender: Sender<ForAccountTracking2>,
        process_account_sender: Sender<ForAccountProcessing>,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        const CLIENT_NAME: &'static str = "simo_robot";
        async move {
            let mut subscribe_request_filter_transactions_map = HashMap::<String, SubscribeRequestFilterTransactions>::new();
            let _ = subscribe_request_filter_transactions_map.insert(
                CLIENT_NAME.to_string(),
                SubscribeRequestFilterTransactions {
                    vote: None,
                    failed: None,
                    signature: None,
                    account_include: vec![],
                    account_exclude: vec![],
                    account_required: vec![],
                },
            );
            let mut subscribe_request_filter_accounts_map = HashMap::<String, SubscribeRequestFilterAccounts>::new();
            let _ = subscribe_request_filter_accounts_map.insert(
                CLIENT_NAME.to_string(),
                SubscribeRequestFilterAccounts {
                    account: vec![],
                    owner: vec![],
                    filters: vec![],
                    nonempty_txn_signature: None,
                }
            );
            let subscribe_request = SubscribeRequest {
                accounts: subscribe_request_filter_accounts_map,
                slots: HashMap::new(),
                transactions: subscribe_request_filter_transactions_map,
                transactions_status: HashMap::new(),
                blocks: HashMap::new(),
                blocks_meta: HashMap::new(),
                entry: HashMap::new(),
                commitment: None,    // TODO TODO Сразу принимать confirmed?
                accounts_data_slice: vec![],
                ping: None,
            };
            'a: loop {
                let mut client = GeyserGrpcClient::build_from_shared(
                    environment_configuration.subject.geyser.grpc_url.as_str()
                )
                .into_(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?
                .connect()
                .await
                .into_(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?;
                let mut subscribe_update = client.subscribe_once(subscribe_request.clone()).await.into_(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?;
                '_b: loop {
                    match subscribe_update.next().await {
                        Some(subscribe_update_) => {
                            match subscribe_update_ {
                                Ok(subscribe_update__) => {
                                    match subscribe_update__.update_oneof {
                                        Some(update_oneof) => {
                                            match update_oneof {
                                                UpdateOneof::Account(subscribe_update_account) => {
                                                    let process_account_sender_ = process_account_sender.clone();
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async move {
                                                            Robot::process_account(
                                                                &subscribe_update_account,
                                                                &process_account_sender_,
                                                            )
                                                            .await
                                                        },
                                                    );
                                                }
                                                UpdateOneof::Slot(subscribe_update_slot) => {
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async {
                                                            tracing::warn!("Unexpected request: Slot");
                                                            Ok(())
                                                        }
                                                    );
                                                }
                                                UpdateOneof::Transaction(subscribe_update_transaction) => {
                                                    let accumulate_trackable_account_sender_ = accumulate_trackable_account_sender.clone();
                                                    let remove_trackable_account_sender_ = remove_trackable_account_sender.clone();
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async move {
                                                            Robot::process_transaction(
                                                                environment_configuration,
                                                                &subscribe_update_transaction,
                                                                &accumulate_trackable_account_sender_,
                                                                remove_trackable_account_sender_,
                                                            )
                                                            .await
                                                        },
                                                    );
                                                }
                                                UpdateOneof::TransactionStatus(subscribe_update_transaction_status) => {
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async {
                                                            tracing::warn!("Unexpected request: TransactionStatus");
                                                            Ok(())
                                                        }
                                                    );
                                                }
                                                UpdateOneof::Entry(subscribe_update_entry) => {
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async {
                                                            tracing::warn!("Unexpected request: Entry");
                                                            Ok(())
                                                        }
                                                    );
                                                }
                                                UpdateOneof::BlockMeta(subscribe_update_block_meta) => {
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async {
                                                            tracing::warn!("Unexpected request: BlockMeta");
                                                            Ok(())
                                                        }
                                                    );
                                                }
                                                UpdateOneof::Block(subscribe_update_block) => {
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async {
                                                            tracing::warn!("Unexpected request: Block");
                                                            Ok(())
                                                        }
                                                    );
                                                }
                                                UpdateOneof::Ping(subscribe_update_ping) => {
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async {
                                                            tracing::warn!("Unexpected request: Ping");
                                                            Ok(())
                                                        }
                                                    );
                                                }
                                                UpdateOneof::Pong(subscribe_update_pong) => {
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async {
                                                            tracing::warn!("Unexpected request: Pong");
                                                            Ok(())
                                                        }
                                                    );
                                                }
                                            }
                                        }
                                        None => {
                                            continue 'a; // TODO TODO Нужно ли делать реконнект, если получена ошибка на приеме данных? Или же можно продолжать прием на том же коннетке.
                                            todo!()
                                        }
                                    }
                                }
                                Err(status) => {
                                    continue 'a; // TODO TODO Нужно ли делать реконнект, если получена ошибка на приеме данных? Или же можно продолжать прием на том же коннетке.
                                    todo!()
                                }
                            }
                        }
                        None => {
                            continue 'a; // TODO TODO Нужно ли делать реконнект, если получена ошибка на приеме данных? Или же можно продолжать прием на том же коннетке.
                            todo!()
                        }
                    }
                }
            }
            Ok(())
        }
    }
}