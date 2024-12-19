use super::environment_configuration::EnvironmentConfiguration;
use std::future::Future;
use super::error::{
    Error,
    ResultConverter,
    Backtrace,
};
use ahash::RandomState;
use std::collections::HashMap;
use super::spawner::Spawner;
use super::environment_configuration::Trade;
use spl_token::{solana_program::program_pack::Pack, state::Account};
use {
    std::convert::TryFrom,
    solana_sdk::pubkey::Pubkey,
};
use super::http_server::HttpServer;
use super::capture::Capture;
use super::grpc_server::GrpcServer;
use crate::extern_source::{
    RaydiumAmmInitializeInstruction2,
    Calcaulator,
};
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
use tokio::signal::unix::SignalKind;
static REGULAR_EXPRESSION: OnceLock<Regex> = OnceLock::new();
pub struct Robot;
impl Robot {
    pub fn start(environment_configuration: &'static EnvironmentConfiguration<Trade>) -> impl Future<Output = Result<(), Error>> + Send {
        fn create_signal(signal_kind: SignalKind) -> Result<impl Future<Output = ()> + Send, Error> {
            let mut signal = tokio::signal::unix::signal(signal_kind).into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let signal_future = async move {
                let _ = signal.recv().await;
            };
            Ok(signal_future)
        }
        async move {
            let grpc_serving_future = GrpcServer::run(environment_configuration);
            let http_serving_future = HttpServer::run(environment_configuration);
            let signal_interrupt_receiving_future = create_signal(SignalKind::interrupt())?;
            let signal_terminate_receiving_future = create_signal(SignalKind::terminate())?;
            let graceful_shutdown_signal_receiving_future = async move {
                tokio::select! {
                    _ = signal_interrupt_receiving_future => {},
                    _ = signal_terminate_receiving_future => {},
                }
                Ok::<_, Error>(())
            };
            let grpc_serving_future_join_handle = Spawner::spawn_tokio_non_blocking_task_processed(grpc_serving_future);
            let http_serving_future_join_handle = Spawner::spawn_tokio_non_blocking_task_processed(http_serving_future);
            let graceful_shutdown_signal_receiving_future_join_handle = Spawner::spawn_tokio_non_blocking_task_processed(
                graceful_shutdown_signal_receiving_future
            );
            tokio::select! {
                _ = grpc_serving_future_join_handle => {}
                _ = http_serving_future_join_handle => {}
                _ = graceful_shutdown_signal_receiving_future_join_handle => {}
            }
            // TODO TODO Здесь получен грейсфулл сигнал. Нужно продолжать получать данные с гейзера,
            // пока все торговые таски не Завершат выполнение, но не создавать новые таски
            // Продолжить эвэитить grpc_serving_future_join_handle через Pin?
            //
            // Либо же не оказываться здесь, и везде, где получаетгрейсфул шатдаун, отправлять по Mpsq сигнал к завершению.
            Ok(())
        }
    }
    pub fn process_transaction<'a>(
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
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
                        environment_configuration,
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
    pub fn process_account<'a>(
        subscribe_update_account: &'a SubscribeUpdateAccount,
        process_account_sender: &'a Sender<ForAccountProcessing>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            let subscribe_update_account_info = subscribe_update_account
            .account
            .as_ref()
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            let account_pubkey = Pubkey::try_from(          // TODO TODO subscribe_update_account.is_startup
                subscribe_update_account_info.pubkey.as_slice()
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
                    data: subscribe_update_account_info.data.clone(),
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
    pub fn trade<'a>(
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
        for_trade: ForTrade,
        trade_trackable_account_receiver: &'a mut Receiver<ForAccountProcessing>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            let expected_coin_amount = match Calcaulator::get_coin_amount_from_pc_amount(
                environment_configuration.subject.trading.initial_pc_amount,
                for_trade.init_pc_amount,
                for_trade.init_coin_amount,
            ) {
                Ok(expected_coin_amount_) => expected_coin_amount_,
                Err(error) => {
                    // TODO TODO отменитьь отслеживагние аккаунтов в accumulate_trackable_account
                    todo!();
                }
            };

            todo!("create transaction");


            'a: loop {
                if let Some(for_account_processing) = trade_trackable_account_receiver.recv().await {
                    let token_account = Account::unpack(for_account_processing.data.as_slice()).map_err(
                        |_: _| -> _ {
                            Error::new_(
                                Common::UnreachableState,
                                Backtrace::new(
                                    line!(),
                                    file!(),
                                ),
                            )
                        }
                    )?;

                    todo!()
                }
            }
            Ok(())
        }
    }
    pub fn accumulate_trackable_account<'a>(
        accumulate_trackable_account_receiver: &'a mut Receiver<ForAccountTracking>,
        process_account_receiver: &'a mut Receiver<ForAccountProcessing>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            let mut trackable_account_registry = HashMap::<Pubkey, Sender<ForAccountProcessing>, RandomState>::default();
            'a: loop {
                tokio::select! {
                    biased;
                    for_account_tracking = accumulate_trackable_account_receiver.recv() => {
                        match for_account_tracking {
                            Some(for_account_tracking_) => {
                                let _ = trackable_account_registry.insert(
                                    for_account_tracking_.amm_coin_vault_pubkey,
                                    for_account_tracking_.trade_trackable_account_sender.clone(),
                                );
                                let _ = trackable_account_registry.insert(
                                    for_account_tracking_.amm_pc_vault_pubkey,
                                    for_account_tracking_.trade_trackable_account_sender,
                                );
                            }
                            None => {
                                continue 'a;
                            }
                        }
                    }
                    for_account_processing = process_account_receiver.recv() => {
                        match for_account_processing {
                            Some(for_account_processing_) => {
                                if let Some(trade_trackable_account_sender) = trackable_account_registry.get(
                                    &for_account_processing_.account_pubkey,
                                ) {
                                    trade_trackable_account_sender.send(for_account_processing_).await.map_err(
                                        |_: _| -> _ {
                                            Error::new_(
                                                Common::UnreachableState,
                                                Backtrace::new(
                                                    line!(),
                                                    file!(),
                                                ),
                                            )
                                        }
                                    )?
                                }
                            }
                            None => {
                                continue 'a;
                            }
                        }
                    }
                }
            }
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
    pub data: Vec<u8>,
}
