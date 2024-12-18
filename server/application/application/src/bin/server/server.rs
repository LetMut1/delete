use super::environment_configuration::EnvironmentConfiguration;
use std::future::Future;
use super::error::{
    Error,
    ResultConverter,
    Backtrace,
};
use super::spawner::Spawner;
use super::environment_configuration::Run;
use {
    futures::stream::StreamExt,
    std::convert::TryFrom,
    solana_sdk::pubkey::Pubkey,
    std::collections::HashMap,
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::prelude::{
        subscribe_request_filter_accounts_filter::Filter as AccountsFilterOneof,
        subscribe_request_filter_accounts_filter_lamports::Cmp as AccountsFilterLamports,
        subscribe_request_filter_accounts_filter_memcmp::Data as AccountsFilterMemcmpOneof,
        subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
        SubscribeRequestAccountsDataSlice, SubscribeRequestFilterAccounts,
        SubscribeRequestFilterAccountsFilter, SubscribeRequestFilterAccountsFilterLamports,
        SubscribeRequestFilterAccountsFilterMemcmp, SubscribeRequestFilterBlocks,
        SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterEntry,
        SubscribeRequestFilterSlots, SubscribeRequestFilterTransactions, SubscribeRequestPing,
        SubscribeUpdateAccountInfo, SubscribeUpdateEntry, SubscribeUpdateTransactionInfo,
    },
};
use crate::capture::Capture;
use crate::raydium_amm_initialize_instruction_2::RaydiumAmmInitializeInstruction2;
use super::error::{
    OptionConverter,
    Common,
};
use super::workflow_data::{
    TransactionDifferentiation,
    WorkflowData,
};
use yellowstone_grpc_proto::geyser::{SubscribeUpdateAccount, SubscribeUpdateTransaction};
use regex::Regex;
use tokio::sync::mpsc::{
    Receiver,
    Sender
};
use std::sync::OnceLock;
static REGULAR_EXPRESSION: OnceLock<Regex> = OnceLock::new();
pub struct Server;
impl Server {
    pub fn run(environment_configuration: &'static EnvironmentConfiguration<Run>) -> impl Future<Output = Result<(), Error>> + Send {
        const CLIENT_NAME: &'static str = "simo_server";
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
            let (
                accumulate_trackable_account_sender,
                mut accumulate_trackable_account_receiver,
            ) = tokio::sync::mpsc::channel::<ForAccountTracking>(100);
            let (
                process_account_sender,
                mut process_account_receiver,
            ) = tokio::sync::mpsc::channel::<ForAccountProcessing>(100000);
            Spawner::spawn_tokio_non_blocking_task_into_background(
                async move {
                    Self::accumulate_trackable_account(
                        &mut accumulate_trackable_account_receiver,
                        &mut process_account_receiver,
                    ).await
                },
            );
            'a: loop {
                let mut client = GeyserGrpcClient::build_from_shared(environment_configuration.subject.geyser.grpc_url.as_str()).into_(
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
                                                            Self::process_account(
                                                                &subscribe_update_account,
                                                                &process_account_sender_,
                                                            ).await
                                                        },
                                                    );
                                                }
                                                UpdateOneof::Slot(subscribe_update_slot) => {
                                                    todo!();
                                                }
                                                UpdateOneof::Transaction(subscribe_update_transaction) => {
                                                    let accumulate_trackable_account_sender_ = accumulate_trackable_account_sender.clone();
                                                    Spawner::spawn_tokio_non_blocking_task_into_background(
                                                        async move {
                                                            Self::process_transaction(
                                                                &subscribe_update_transaction,
                                                                &accumulate_trackable_account_sender_,
                                                            ).await
                                                        },
                                                    );
                                                }
                                                UpdateOneof::TransactionStatus(subscribe_update_transaction_status) => {
                                                    todo!();
                                                }
                                                UpdateOneof::Entry(subscribe_update_entry) => {
                                                    todo!();
                                                }
                                                UpdateOneof::BlockMeta(subscribe_update_block_meta) => {
                                                    todo!();
                                                }
                                                UpdateOneof::Block(subscribe_update_block) => {
                                                    todo!();
                                                }
                                                UpdateOneof::Ping(subscribe_update_ping) => {

                                                }
                                                UpdateOneof::Pong(subscribe_update_pong) => {
                                                    todo!();
                                                }
                                            }
                                        }
                                        None => {
                                            continue 'a; // TODO TODO Нужно ли делать реконнект, если получена ошибка на приеме данных? Или же можно продолжать прием на том же коннетке.
                                        }
                                    }
                                }
                                Err(status) => {
                                    continue 'a; // TODO TODO Нужно ли делать реконнект, если получена ошибка на приеме данных? Или же можно продолжать прием на том же коннетке.
                                }
                            }
                        }
                        None => {
                            continue 'a; // TODO TODO Нужно ли делать реконнект, если получена ошибка на приеме данных? Или же можно продолжать прием на том же коннетке.
                        }
                    }
                }
            }
            Ok(())
        }
    }
    fn process_transaction<'a>(
        subscribe_update_transaction: &'a SubscribeUpdateTransaction,
        accumulate_trackable_account_sender: &'a Sender<ForAccountTracking>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            let subscribe_update_transaction_info = subscribe_update_transaction
            .transaction
            .as_ref()
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let transaction_status_meta = subscribe_update_transaction_info
            .meta
            .as_ref()
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            if transaction_status_meta.err.is_some() {
                return Ok(());
            }
            let message = subscribe_update_transaction_info
            .transaction
            .as_ref()
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?
            .message
            .as_ref()
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            if message.instructions.len() != WorkflowData::<TransactionDifferentiation>::INSTRUCTIONS_QUANTITY {
                return Ok(());
            }
            let compiled_instruction = &message.instructions[
                WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INSTRUCTION_VECTOR_INDEX
            ];
            let pubkey = Pubkey::try_from(
                message.account_keys[compiled_instruction.program_id_index as usize].as_slice()
            )
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            if pubkey != WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_CONTRACT_PUBKEY
            || transaction_status_meta.inner_instructions.len() != WorkflowData::<TransactionDifferentiation>::INSTRUCTIONS_WITH_INNER_INSTRUCTIONS_QUANTITY
            || transaction_status_meta.inner_instructions[0].index as usize != WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INSTRUCTION_VECTOR_INDEX
            || transaction_status_meta.inner_instructions[0].instructions.len() != WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTIONS_QUANTITY {
                return Ok(());
            }
            let regular_expression = match REGULAR_EXPRESSION.get() {
                Option::Some(regular_expression_) => regular_expression_,
                Option::None => {
                    if REGULAR_EXPRESSION
                        .set(
                            Regex::new(WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_PATTERN).into_(
                                Backtrace::new(
                                    line!(),
                                    file!(),
                                ),
                            )?,
                        )
                        .is_err()
                    {
                        return Result::Err(
                            Error::new_(
                                Common::ValueAlreadyExist,
                                Backtrace::new(
                                    line!(),
                                    file!(),
                                ),
                            ),
                        );
                    }
                    REGULAR_EXPRESSION.get().into_value_does_not_exist(
                        Backtrace::new(
                            line!(),
                            file!(),
                        ),
                    )?
                }
            };
            if !regular_expression.is_match(
                transaction_status_meta.log_messages[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_VECTOR_INDEX].as_str()
            ) {
                return Ok(());
            }
            let amm_market_pubkey = Pubkey::try_from(
                message.account_keys[
                    compiled_instruction.accounts[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_MARKET_PUBKEY_VECTOR_INDEX
                    ] as usize
                ].as_slice()
            )
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let amm_coin_vault_pubkey = Pubkey::try_from(
                message.account_keys[
                    compiled_instruction.accounts[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_COIN_VAULT_PUBKEY_VECTOR_INDEX
                    ] as usize
                ].as_slice()
            )
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let amm_pc_vault_pubkey = Pubkey::try_from(
                message.account_keys[
                    compiled_instruction.accounts[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_PC_VAULT_PUBKEY_VECTOR_INDEX
                    ] as usize
                ].as_slice()
            )
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let raydium_amm_initialize_instruction_2 = RaydiumAmmInitializeInstruction2::unpack(
                compiled_instruction.data.as_slice(),
            )?;
            let (
                trade_trackable_account_sender,
                mut trade_trackable_account_receiver,
            ) = tokio::sync::mpsc::channel::<ForAccountProcessing>(10);
            let amm_coin_vault_pubkey_ = amm_coin_vault_pubkey.clone();
            let amm_pc_vault_pubkey_ = amm_pc_vault_pubkey.clone();
            Spawner::spawn_tokio_non_blocking_task_into_background(
                async move {
                    Self::trade(
                        ForTrade {
                            amm_market_pubkey,
                            amm_coin_vault_pubkey: amm_coin_vault_pubkey_,
                            init_coin_amount: raydium_amm_initialize_instruction_2.init_coin_amount,
                            amm_pc_vault_pubkey: amm_pc_vault_pubkey_,
                            init_pc_amount: raydium_amm_initialize_instruction_2.init_pc_amount,
                        },
                        &mut trade_trackable_account_receiver,
                    ).await
                },
            );
            accumulate_trackable_account_sender.send(
                ForAccountTracking {
                    amm_coin_vault_pubkey,
                    amm_pc_vault_pubkey,
                    trade_trackable_account_sender,
                },
            )
            .await
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )
        }
    }
    fn process_account<'a>(
        subscribe_update_account: &'a SubscribeUpdateAccount,
        process_account_sender: &'a Sender<ForAccountProcessing>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            let account_pubkey = Pubkey::try_from(          // TODO TODO subscribe_update_account.is_startup
                subscribe_update_account
                .account
                .as_ref()
                .into_value_does_not_exist(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?
                .pubkey
                .as_slice()
            )
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            process_account_sender.send(
                ForAccountProcessing {
                    account_pubkey,
                },
            )
            .await
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )
        }
    }
    fn trade<'a>(
        for_trade: ForTrade,
        trade_trackable_account_receiver: &'a mut Receiver<ForAccountProcessing>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()>{
        async move {
            // inf loop
            Ok(())
        }
    }
    fn accumulate_trackable_account<'a>(
        accumulate_trackable_account_receiver: &'a mut Receiver<ForAccountTracking>,
        process_account_receiver: &'a mut Receiver<ForAccountProcessing>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            // accumulate_trackable_account_receiver- принимает новые аккаунты с process_transaction

            // inf loop
            Ok(())
        }
    }
}
pub struct ForTrade {
    pub amm_market_pubkey: Pubkey,
    pub amm_coin_vault_pubkey: Pubkey,
    pub init_coin_amount: u64,
    pub amm_pc_vault_pubkey: Pubkey,
    pub init_pc_amount: u64,
}
pub struct ForAccountTracking {
    pub amm_coin_vault_pubkey: Pubkey,
    pub amm_pc_vault_pubkey: Pubkey,
    pub trade_trackable_account_sender: Sender<ForAccountProcessing>
}
pub struct ForAccountProcessing {
    pub account_pubkey: Pubkey,
}
