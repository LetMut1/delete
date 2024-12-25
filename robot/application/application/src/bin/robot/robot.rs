use super::environment_configuration::EnvironmentConfiguration;
use std::{future::Future, sync::atomic::{AtomicBool, AtomicUsize, Ordering}, time::Duration};
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
use tokio::sync::mpsc::{
    Receiver,
    Sender
};
use tokio::signal::unix::SignalKind;
static IS_GRACEFUL_SHUTDOWN_COMMAND_RECEIVED: AtomicBool = AtomicBool::new(false);
static TRADING_TASKS_QUANTITY: AtomicUsize = AtomicUsize::new(0);
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
            let (
                accumulate_trackable_account_sender,
                mut accumulate_trackable_account_receiver,
            ) = tokio::sync::mpsc::channel::<ForAccountTracking1>(100);
            let (
                remove_trackable_account_sender,
                mut remove_trackable_account_receiver,
            ) = tokio::sync::mpsc::channel::<ForAccountTracking2>(10);
            let (
                process_account_sender,
                mut process_account_receiver,
            ) = tokio::sync::mpsc::channel::<ForAccountProcessing>(100000);
            Spawner::spawn_tokio_non_blocking_task_into_background(
                async move {
                    Robot::accumulate_trackable_account(
                        &mut accumulate_trackable_account_receiver,
                        &mut remove_trackable_account_receiver,
                        &mut process_account_receiver,
                    )
                    .await
                },
            );
            let mpsc_receiver_guard = MpscReceiverGuard {
                accumulate_trackable_account_sender: accumulate_trackable_account_sender.clone(),
                remove_trackable_account_sender: remove_trackable_account_sender.clone(),
                process_account_sender: process_account_sender.clone(),
            };
            let grpc_serving_future = GrpcServer::run(
                environment_configuration,
                accumulate_trackable_account_sender,
                remove_trackable_account_sender,
                process_account_sender,
            );
            let http_serving_future = HttpServer::run(
                environment_configuration,
                &IS_GRACEFUL_SHUTDOWN_COMMAND_RECEIVED,
            );
            let signal_interrupt_receiving_future = create_signal(SignalKind::interrupt())?;
            let signal_terminate_receiving_future = create_signal(SignalKind::terminate())?;
            let graceful_shutdown_signal_receiving_future = async move {
                tokio::select! {
                    _ = signal_interrupt_receiving_future => {},
                    _ = signal_terminate_receiving_future => {},
                }
                Ok::<_, Error>(())
            };
            let graceful_shutdown_process_future = async move {
                'a: loop {
                    if IS_GRACEFUL_SHUTDOWN_COMMAND_RECEIVED.load(Ordering::Relaxed)
                    && TRADING_TASKS_QUANTITY.load(Ordering::Relaxed) == 0 {
                        break 'a;
                    } else {
                        tokio::time::sleep(Duration::from_secs(10)).await
                    }
                }
                Ok::<_, Error>(())
            };
            Spawner::spawn_tokio_non_blocking_task_into_background(grpc_serving_future);
            Spawner::spawn_tokio_non_blocking_task_into_background(http_serving_future);
            let graceful_shutdown_signal_receiving_future_join_handle = Spawner::spawn_tokio_non_blocking_task_processed(
                graceful_shutdown_signal_receiving_future,
            );
            let mut graceful_shutdown_process_future_join_handle = std::pin::pin!(
                Spawner::spawn_tokio_non_blocking_task_processed(
                    graceful_shutdown_process_future,
                )
            );
            let mut graceful_shutdown_process_future_join_handle_ = graceful_shutdown_process_future_join_handle.as_mut();
            let mut is_graceful_shutdown_process_complite = false;
            tokio::select! {
                _ = graceful_shutdown_signal_receiving_future_join_handle => {
                    IS_GRACEFUL_SHUTDOWN_COMMAND_RECEIVED.store(true, Ordering::Relaxed);
                }
                _ = graceful_shutdown_process_future_join_handle_.as_mut() => {
                    is_graceful_shutdown_process_complite = true;
                }
            }
            if !is_graceful_shutdown_process_complite {
                let _ = graceful_shutdown_process_future_join_handle_.await;
            }
            Ok(())
        }
    }
    pub fn process_transaction<'a>(
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
        subscribe_update_transaction: &'a SubscribeUpdateTransaction,
        accumulate_trackable_account_sender: &'a Sender<ForAccountTracking1>,
        remove_trackable_account_sender: Sender<ForAccountTracking2>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            if !IS_GRACEFUL_SHUTDOWN_COMMAND_RECEIVED.load(Ordering::Relaxed) {
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
                let initialize_2_compiled_instruction = &message.instructions[
                    WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INITIALIZE_2_INSTRUCTION_VECTOR_INDEX
                ];
                if message.account_keys[initialize_2_compiled_instruction.program_id_index as usize].as_slice() != WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_CONTRACT_PUBKEY.to_bytes().as_slice()
                || transaction_status_meta.inner_instructions.len() != WorkflowData::<TransactionDifferentiation>::INSTRUCTIONS_WITH_INNER_INSTRUCTIONS_QUANTITY
                || transaction_status_meta.inner_instructions[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTION_VECTOR_INDEX].index as usize != WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INITIALIZE_2_INSTRUCTION_VECTOR_INDEX
                || transaction_status_meta.inner_instructions[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTION_VECTOR_INDEX].instructions.len() != WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTIONS_QUANTITY
                || transaction_status_meta.log_messages.len() < WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_VECTOR_INDEX + 1
                || transaction_status_meta.log_messages[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_VECTOR_INDEX].as_bytes()[0..=47] != *WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_PATTERN
                {
                    return Ok(());
                }
                let create_token_account_compiled_instruction = &transaction_status_meta.inner_instructions[
                    WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTION_VECTOR_INDEX
                ].instructions[
                    WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_PC_VAULT_TOKEN_ACCOUNT_INITIALIZING_INSTRUCTION_VECTOR_INDEX
                ];
                if message.account_keys[create_token_account_compiled_instruction.program_id_index as usize].as_slice() != WorkflowData::<TransactionDifferentiation>::TOKEN_PROGRAM_PUBKEY.to_bytes().as_slice()
                || message.account_keys[create_token_account_compiled_instruction.accounts[1] as usize].as_slice() != WorkflowData::<TransactionDifferentiation>::WRAPPED_SOL_TOKEN_ACCOUNT_PUBKEY.to_bytes().as_slice()
                || create_token_account_compiled_instruction.accounts[0] != initialize_2_compiled_instruction.accounts[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_PC_VAULT_PUBKEY_VECTOR_INDEX] {
                    return Ok(());
                }
                let amm_market_pubkey = Pubkey::try_from(
                    message.account_keys[
                        initialize_2_compiled_instruction.accounts[
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
                        initialize_2_compiled_instruction.accounts[
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
                        initialize_2_compiled_instruction.accounts[
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
                    initialize_2_compiled_instruction.data.as_slice(),
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
                            &remove_trackable_account_sender,
                        ).await
                    },
                );
                accumulate_trackable_account_sender.send(
                    ForAccountTracking1 {
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
                )?;
            }
            Ok(())
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
    fn trade<'a>(
        environment_configuration: &'static EnvironmentConfiguration<Trade>,
        for_trade: ForTrade,
        trade_trackable_account_receiver: &'a mut Receiver<ForAccountProcessing>,
        remove_trackable_account_sender: &'a Sender<ForAccountTracking2>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {

        todo!("инкрементировать счетчик тасков.");



        async move {
            let expected_coin_amount = match Calcaulator::get_coin_amount_from_pc_amount(
                environment_configuration.subject.trading.initial_pc_amount,
                for_trade.init_pc_amount,
                for_trade.init_coin_amount,
            ) {
                Ok(expected_coin_amount_) => expected_coin_amount_,
                Err(error) => {
                    let _ = remove_trackable_account_sender.send(
                        ForAccountTracking2 {
                            amm_coin_vault_pubkey: for_trade.amm_coin_vault_pubkey,
                            amm_pc_vault_pubkey: for_trade.amm_pc_vault_pubkey,
                        }
                    )
                    .await;
                    return Err(error);
                }
            };

            todo!("create transaction for buy, Отслеживаение удачи этой транзакции");


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

                    todo!(" отслеживание аккаунтов для изменения профита. Транзакция на продажу.");





                    // Удаление аккаунтов из отлеживаемых после завершения трейдинга на текущую пару монет
                    let _ = remove_trackable_account_sender.send(
                        ForAccountTracking2 {
                            amm_coin_vault_pubkey: for_trade.amm_coin_vault_pubkey,
                            amm_pc_vault_pubkey: for_trade.amm_pc_vault_pubkey,
                        }
                    )
                    .await;
                }
            }
            Ok(())
        }
    }
    fn accumulate_trackable_account<'a>(
        accumulate_trackable_account_receiver: &'a mut Receiver<ForAccountTracking1>,
        remove_trackable_account_receiver: &'a mut Receiver<ForAccountTracking2>,
        process_account_receiver: &'a mut Receiver<ForAccountProcessing>,
    )-> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            let mut trackable_account_registry = HashMap::<Pubkey, Sender<ForAccountProcessing>, RandomState>::default();
            '_a: loop {
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
                                return Err(
                                    Error::new_(
                                        Common::UnreachableState,
                                        Backtrace::new(
                                            line!(),
                                            file!(),
                                        ),
                                    ),
                                );
                            }
                        }
                    }
                    for_account_tracking = remove_trackable_account_receiver.recv() => {
                        match for_account_tracking {
                            Some(for_account_tracking_) => {
                                let _ = trackable_account_registry.remove(&for_account_tracking_.amm_coin_vault_pubkey);
                                let _ = trackable_account_registry.remove(&for_account_tracking_.amm_pc_vault_pubkey);
                            }
                            None => {
                                return Err(
                                    Error::new_(
                                        Common::UnreachableState,
                                        Backtrace::new(
                                            line!(),
                                            file!(),
                                        ),
                                    ),
                                );
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
                                return Err(
                                    Error::new_(
                                        Common::UnreachableState,
                                        Backtrace::new(
                                            line!(),
                                            file!(),
                                        ),
                                    ),
                                );
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
    amm_market_pubkey: Pubkey,
    amm_coin_vault_pubkey: Pubkey,
    init_coin_amount: u64,
    amm_pc_vault_pubkey: Pubkey,
    init_pc_amount: u64,
}
pub struct ForAccountTracking1 {
    amm_coin_vault_pubkey: Pubkey,
    amm_pc_vault_pubkey: Pubkey,
    trade_trackable_account_sender: Sender<ForAccountProcessing>
}
pub struct ForAccountTracking2 {
    amm_coin_vault_pubkey: Pubkey,
    amm_pc_vault_pubkey: Pubkey,
}
pub struct ForAccountProcessing {
    account_pubkey: Pubkey,
    data: Vec<u8>,
}
struct MpscReceiverGuard {
    accumulate_trackable_account_sender: Sender<ForAccountTracking1>,
    remove_trackable_account_sender: Sender<ForAccountTracking2>,
    process_account_sender: Sender<ForAccountProcessing>,
}