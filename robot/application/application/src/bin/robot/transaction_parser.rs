use solana_rpc_client_api::config::RpcTransactionConfig;
use solana_sdk::message::VersionedMessage;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::UiInstruction;
use crate::capture::Capture;
use crate::error::Common;
use crate::error::OptionConverter;
use crate::workflow_data::TransactionDifferentiation;
use crate::workflow_data::WorkflowData;
use super::environment_configuration::EnvironmentConfiguration;
use std::future::Future;
use solana_sdk::commitment_config::CommitmentLevel;
use std::str::FromStr;
use solana_transaction_status::UiTransactionEncoding;
use std::time::Duration;
use solana_transaction_status::TransactionBinaryEncoding;
use solana_transaction_status::EncodedTransaction;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use super::error::{
    Error,
    ResultConverter,
    Backtrace,
};
use super::extern_source::RaydiumAmmInitializeInstruction2;
use super::environment_configuration::ParseTransaction;
pub struct TransactionParser;
impl TransactionParser {
    pub fn parse<'a>(environment_configuration: &'a EnvironmentConfiguration<ParseTransaction>) -> impl Future<Output = Result<(), Error>> + Send + Capture<&'a ()> {
        async move {
            let rpc_client = RpcClient::new_with_timeout(
                "https://api.mainnet-beta.solana.com".to_string(),
                Duration::from_secs(90),
            );
            'a: for solana_transaction_signature in environment_configuration.subject.solana_transaction_signature_registry.iter() {
                let signature = Signature::from_str(solana_transaction_signature.as_str())
                .into_(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?;
                let encoded_confirmed_transaction_with_status_meta = rpc_client.get_transaction_with_config(
                    &signature,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Base58),
                        commitment: Some(
                            CommitmentConfig {
                                commitment: CommitmentLevel::Finalized
                            }
                        ),
                        max_supported_transaction_version: Some(0),
                    }
                )
                .await
                .into_(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?;
                let base58_transaction = match encoded_confirmed_transaction_with_status_meta.transaction.transaction {
                    EncodedTransaction::Binary(base58_transaction_, TransactionBinaryEncoding::Base58) => base58_transaction_,
                    _ => {
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
                };
                let transaction_byte_registry = bs58::decode(base58_transaction.as_str())
                .into_vec()
                .into_(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?;
                let versioned_transaction = bincode::deserialize::<VersionedTransaction>(transaction_byte_registry.as_slice())
                .into_(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?;
                if versioned_transaction.signatures[0] != signature {
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
                let message = match versioned_transaction.message {
                    VersionedMessage::V0(message) => message,
                    _ => {
                        tracing::info!("{} - invalid.", solana_transaction_signature.as_str());
                        continue 'a;
                    }
                };
                let ui_transaction_status_meta = encoded_confirmed_transaction_with_status_meta.transaction.meta
                .into_value_does_not_exist(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )?;
                if ui_transaction_status_meta.err.is_some() {
                    tracing::info!("{} - failed.", solana_transaction_signature.as_str());
                    continue 'a;
                }
                let mut is_right_transaction = true;
                if message.instructions.len() != WorkflowData::<TransactionDifferentiation>::INSTRUCTIONS_QUANTITY {
                    is_right_transaction = false;
                }
                let initialize_2_compiled_instruction = &message.instructions[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INITIALIZE_2_INSTRUCTION_VECTOR_INDEX];
                if is_right_transaction {
                    if message.account_keys[initialize_2_compiled_instruction.program_id_index as usize] != WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_CONTRACT_PUBKEY {
                        is_right_transaction = false;
                    }
                }
                let ui_inner_instruction_registry_stub = vec![];
                let ui_inner_instruction_registry = if is_right_transaction {
                    match ui_transaction_status_meta.inner_instructions {
                        OptionSerializer::Some(ref ui_inner_instruction_registry_) => ui_inner_instruction_registry_,
                        _ => &ui_inner_instruction_registry_stub,
                    }
                } else {
                    &ui_inner_instruction_registry_stub
                };
                is_right_transaction =
                ui_inner_instruction_registry.len() == WorkflowData::<TransactionDifferentiation>::INSTRUCTIONS_WITH_INNER_INSTRUCTIONS_QUANTITY
                && ui_inner_instruction_registry[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTION_VECTOR_INDEX].index as usize == WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INITIALIZE_2_INSTRUCTION_VECTOR_INDEX
                && ui_inner_instruction_registry[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTION_VECTOR_INDEX].instructions.len() == WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTIONS_QUANTITY;
                let log_message_registry_stub = vec![];
                let log_message_registry = if is_right_transaction {
                    match ui_transaction_status_meta.log_messages {
                        OptionSerializer::Some(ref log_message_registry_) => log_message_registry_,
                        _ => &log_message_registry_stub,
                    }
                } else {
                    &log_message_registry_stub
                };
                is_right_transaction = log_message_registry.len() >= WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_VECTOR_INDEX + 1
                && log_message_registry[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_VECTOR_INDEX].as_bytes()[0..=47] == *WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_LOG_PATTERN;
                if is_right_transaction {
                    let create_token_account_compiled_instruction = &ui_inner_instruction_registry[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTION_VECTOR_INDEX
                    ].instructions[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_PC_VAULT_TOKEN_ACCOUNT_INITIALIZING_INSTRUCTION_VECTOR_INDEX
                    ];
                    let ui_compiled_instruction = match create_token_account_compiled_instruction {
                        &UiInstruction::Compiled(ref ui_compiled_instruction_) => ui_compiled_instruction_,
                        &UiInstruction::Parsed(ref ui_parsed_instruction) => {
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
                    };
                    is_right_transaction = message.account_keys[ui_compiled_instruction.program_id_index as usize] == WorkflowData::<TransactionDifferentiation>::TOKEN_PROGRAM_PUBKEY
                    && message.account_keys[ui_compiled_instruction.accounts[1] as usize] == WorkflowData::<TransactionDifferentiation>::WRAPPED_SOL_TOKEN_ACCOUNT_PUBKEY
                    && ui_compiled_instruction.accounts[0] == initialize_2_compiled_instruction.accounts[WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_PC_VAULT_PUBKEY_VECTOR_INDEX];
                }
                if !is_right_transaction {
                    tracing::info!("{} - invalid.", solana_transaction_signature.as_str());
                    continue 'a;
                }
                let amm_market_pubkey = message.account_keys[
                    initialize_2_compiled_instruction.accounts[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_MARKET_PUBKEY_VECTOR_INDEX
                    ] as usize
                ];
                let amm_coin_vault_pubkey = message.account_keys[
                    initialize_2_compiled_instruction.accounts[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_COIN_VAULT_PUBKEY_VECTOR_INDEX
                    ] as usize
                ];
                let amm_pc_vault_pubkey = message.account_keys[
                    initialize_2_compiled_instruction.accounts[
                        WorkflowData::<TransactionDifferentiation>::RAYDIUM_LIQUIDITY_POOL_V4_AMM_PC_VAULT_PUBKEY_VECTOR_INDEX
                    ] as usize
                ];
                let raydium_amm_initialize_instruction_2 = RaydiumAmmInitializeInstruction2::unpack(
                    initialize_2_compiled_instruction.data.as_slice(),
                )?;
                tracing::info!(
                    "\n{} - right.\namm_market_pubkey: {}\namm_coin_vault_pubkey: {}\namm_pc_vault_pubkey: {}\nraydium_amm_initialize_instruction_2: {:?}",
                    solana_transaction_signature.as_str(),
                    &amm_market_pubkey,
                    &amm_coin_vault_pubkey,
                    &amm_pc_vault_pubkey,
                    &raydium_amm_initialize_instruction_2,
                );
            }
            Ok(())
        }
    }
}
/*
let rpc_client = RpcClient::new_with_timeout("https://api.mainnet-beta.solana.com".to_string(), Duration::from_secs(60));
let signature = Signature::from_str("2gMuTdGx6RaQKSrUqGib2kkNQ7XD71eMvA3fm8h5MY8qFSLoALQrnxiWo3YzCdaTSEstGd751HwD3LqVaxjX268t")?;
*/
/*
let encoded_confirmed_transaction_with_status_meta = rpc_client.get_transaction_with_config(
    &signature,
    RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(
            CommitmentConfig {
                commitment: CommitmentLevel::Finalized
            }
        ),
        max_supported_transaction_version: Some(0),
    }
)?;
println!("{:?}", encoded_confirmed_transaction_with_status_meta);

OUTPUT:
EncodedConfirmedTransactionWithStatusMeta { slot: 304155378, transaction: EncodedTransactionWithStatusMeta { transaction:
Json(UiTransaction { signatures: ["2gMuTdGx6RaQKSrUqGib2kkNQ7XD71eMvA3fm8h5MY8qFSLoALQrnxiWo3YzCdaTSEstGd751HwD3LqVaxjX268t"],
message: Raw(UiRawMessage { header: MessageHeader { num_required_signatures: 1, num_readonly_signed_accounts: 0, num_readonly_unsigned_accounts: 11 },
account_keys: ["87nRYXqKArSLrotSCWXRLkCnk5jVfiQVo3HqjLT31VeK", "5jhe4Lf4J51Afa53hjgz4WUmWN6TEnxW15bEKUGFD44M", "6DdbGL4GRr4gMeE4gkHQHLVABuD8vMDNC1QmsgtUKvPZ",
"6ZmtFc7ZfGZCvTGwtKnNCqjDBKd8JFYrMwvXD3kZs9Yc", "JBFZxVNNMrR6prECdMWbSQXMUtGjYRYy61psjgQdm5jU", "93NvHA7Ci7yu6oL4sca1f976AcKpAUSXNUMs1YDQZvZb",
"Dc88MUmS675aV4YDLkyLvofSSBidkAFWcVWsiQKRnpX9", "HXkWvZfyo8gwJZMduYVBbMXSh12X9xWs9h32syfpjoKX", "7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5",
"GPjBbuAQJ5nLCuCfbvsgxwZZLLNqwQ3jUT2Crkx3muFL", "DZAbjSBqm4jerdzJymDp8bphDpoSmMCL7bbFQhAtr6Ei", "11111111111111111111111111111111",
"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "So11111111111111111111111111111111111111112", "SysvarRent111111111111111111111111111111111",
"675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
"q2y4FENF5cFdX95e2hg6e2MRdxkfeiF3PUHtw1ypump", "9DCxsMizn3H1hprZ7xWe6LDzeUeZBksYFpBWBtSf1PQX", "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX",
"BnMFVbommD8RLKjoGTeH5DyuoXAmMjkV8h42sPpYmJ28"], recent_blockhash: "4nPwNadGMsKpsHvUG2WR1vCesejp4o7EfkwgMjgrb3uP", instructions: [UiCompiledInstruction
{ program_id_index: 11, accounts: [0, 1], data: "3ipZWoAAn4YDcVQVHvBfwEQ2246mFzLaU4WzTiFjwbB26yfE6t7trEPrFrKDhPYKrYAqR21Kc3UdwikPj83Ftot5EDi7U4pzcefyf4VHA6DdajWx2PR9czmBVooYwWQ7XHTzixxJTkta5Gxudpy8sv7pxbMwaij8q77LRyrPi",
stack_height: None }, UiCompiledInstruction { program_id_index: 12, accounts: [1, 13, 0, 14], data: "2", stack_height: None }, UiCompiledInstruction {
program_id_index: 15, accounts: [12, 16, 11, 14, 2, 17, 3, 4, 18, 13, 5, 6, 7, 19, 8, 20, 21, 0, 9, 1, 10], data: "4YGRzKFWLGjqCEnZ2ZNMhL4Z2pcEJ8HRH4u",
stack_height: None }, UiCompiledInstruction { program_id_index: 12, accounts: [1, 0, 0], data: "A", stack_height: None }], address_table_lookups: Some([]) }) }),
meta: Some(UiTransactionStatusMeta { err: None, status: Ok(()), fee: 5000, pre_balances: [764655925380, 0, 0, 0, 0, 0, 0, 0, 165490584495438, 2039280, 0, 1, 934087680,
735562554318, 1009200, 1141440, 731913600, 13412421853, 1461600, 4677120, 1141440, 3591360], post_balances: [1202599820, 0, 6124800, 23357760, 1461600, 2039280, 763002039280,
16258560, 165490984495438, 2039280, 2039280, 1, 934087680, 735562554318, 1009200, 1141440, 731913600, 13412421853, 1461600, 4677120, 1141440, 3591360],
inner_instructions: Some([UiInnerInstructions { index: 2, instructions: [Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 8], data:
"3Bxs3zwhE1jnACsh", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [8], data: "J", stack_height: Some(2) }),
Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 7], data: "3Bxs3zsXjYXEcY9D", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 11, accounts: [7], data: "9krTDTC9CyNDTCP9", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11,
accounts: [7], data: "SYXsG5gxn13RGVJBuJ66WMvnpkuC3ZXmxCAkmzi1nLhi459e", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index:
11, accounts: [0, 4], data: "3Bxs4GxuFmUxu9wu", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [4], data:
"9krTDE99A3SWNSHd", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [4], data: "SYXsBSQy3GeifSEQSGvTbrPNposbSAiSoh1YA85wcvGKSnYg",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [4, 14], data: "1D8qpeSmcAZXbhY6jAPqguwXxxrrFAnmcbUaH5dxdLLS3Ub",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 5], data: "3Bxs4h24hBtQy9rw", stack_height: Some(2) }),
Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [5], data: "9krTDU2LzCSUJuVZ", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 11, accounts: [5], data: "SYXsBSQy3GeifSEQSGvTbrPNposbSAiSoh1YA85wcvGKSnYg", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 12, accounts: [5, 18, 17, 14], data: "2", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts:
[0, 6], data: "3Bxs4h24hBtQy9rw", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [6], data: "9krTDU2LzCSUJuVZ",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [6], data: "SYXsBSQy3GeifSEQSGvTbrPNposbSAiSoh1YA85wcvGKSnYg",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [6, 13, 17, 14], data: "2", stack_height: Some(2) }),
Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 2], data: "3Bxs3zw7D1St6MtB", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 11, accounts: [2], data: "9krTDga1qCiqxLs9", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts:
[2], data: "SYXsG5gxn13RGVJBuJ66WMvnpkuC3ZXmxCAkmzi1nLhi459e", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts:
[0, 3], data: "3Bxs4BdXwcxHpZ19", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [3], data: "9krTDSXVJqcrnRvf",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [3], data: "SYXsBrTzDsq3kLD1BhH4w6jQTUs6sbwfa7yN5CyH8syhMbj3",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 20, accounts: [3, 17, 21, 14], data: "1PEpEB", stack_height: Some(2) }),
Compiled(UiCompiledInstruction { program_id_index: 16, accounts: [0, 10, 0, 4, 11, 12], data: "1", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 12, accounts: [4], data: "84eT", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 10],
data: "11119os1e9qSs2u7TsThXqkBSRVFxhmYaFKFZ1waB2X7armDmvK3p5GmLdUxYdg3h7QSrL", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index:
12, accounts: [10], data: "P", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [10, 4], data:
"6UFV6UjN7ThiB8HyxHfikpieBEyKK5TgCVxThDv8rJe5H", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [9, 5, 0], data: "3DVzL1qggAfT",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [1, 6, 0], data: "3DU78ZaeqheF", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 12, accounts: [4, 10, 17], data: "6MYaS7nsq4yd", stack_height: Some(2) })] }]), log_messages: Some(["Program 11111111111111111111111111111111 invoke [1]",
"Program 11111111111111111111111111111111 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]", "Program log: Instruction: InitializeAccount",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3443 of 799850 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
"Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 invoke [1]", "Program log: initialize2: InitializeInstruction2 { nonce: 254, open_time: 1732807457,
init_pc_amount: 763000000000, init_coin_amount: 206900000000000000 }", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: SyncNative", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3045 of
778963 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 11111111111111111111111111111111 invoke [2]", "Program
11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program
11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]",
"Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
invoke [2]", "Program log: Instruction: InitializeMint", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2920 of 748273 compute units", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111
success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111
invoke [2]", "Program 11111111111111111111111111111111 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction:
InitializeAccount", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4475 of 733771 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]",
"Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: InitializeAccount", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
consumed 3445 of 714695 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 11111111111111111111111111111111 invoke [2]",
"Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]",
"Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX invoke [2]",
"Program srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX consumed 2319 of 684694 compute units", "Program srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX success",
"Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]", "Program log: Create", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
"Program log: Instruction: GetAccountDataSize", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1595 of 670831 compute units", "Program return:
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 11111111111111111111111111111111 invoke [3]",
"Program 11111111111111111111111111111111 success", "Program log: Initialize the associated token account", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
"Program log: Instruction: InitializeImmutableOwner", "Program log: Please upgrade to SPL Token 2022 for immutable owner support", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 664218 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]", "Program log: Instruction: InitializeAccount3", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
consumed 4214 of 660336 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL
consumed 20389 of 676228 compute units", "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
invoke [2]", "Program log: Instruction: Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 652851 compute units", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction:
Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4736 of 645248 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: MintTo", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
consumed 4492 of 627855 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program log: ray_log:
ACGLSGcAAAAACQkA4fUFAAAAAADh9QUAAAAAAA5YprEAAAAAQAcsdA7fAqAy5vwufyQb7XVEmbVElVzX4rS5eY47hgAW/v6eAilV", "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8
consumed 185777 of 796407 compute units", "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]",
"Program log: Instruction: CloseAccount", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2915 of 610630 compute units", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success"]), pre_token_balances: Some([UiTransactionTokenBalance { account_index: 8, mint:
"So11111111111111111111111111111111111111112", ui_token_amount: UiTokenAmount { ui_amount: Some(165490.582456158), decimals: 9,
amount: "165490582456158", ui_amount_string: "165490.582456158" }, owner: Some("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ"),
program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }, UiTransactionTokenBalance { account_index: 9, mint:
"q2y4FENF5cFdX95e2hg6e2MRdxkfeiF3PUHtw1ypump", ui_token_amount: UiTokenAmount { ui_amount: Some(206900000.0), decimals: 9,
amount: "206900000000000000", ui_amount_string: "206900000" }, owner: Some("87nRYXqKArSLrotSCWXRLkCnk5jVfiQVo3HqjLT31VeK"), program_id:
Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }]), post_token_balances: Some([UiTransactionTokenBalance { account_index: 5, mint:
"q2y4FENF5cFdX95e2hg6e2MRdxkfeiF3PUHtw1ypump", ui_token_amount: UiTokenAmount { ui_amount: Some(206900000.0), decimals: 9, amount:
"206900000000000000", ui_amount_string: "206900000" }, owner: Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"), program_id:
Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }, UiTransactionTokenBalance { account_index: 6, mint: "So11111111111111111111111111111111111111112",
ui_token_amount: UiTokenAmount { ui_amount: Some(763.0), decimals: 9, amount: "763000000000", ui_amount_string: "763" },
owner: Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"), program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") },
UiTransactionTokenBalance { account_index: 8, mint: "So11111111111111111111111111111111111111112", ui_token_amount: UiTokenAmount
{ ui_amount: Some(165490.982456158), decimals: 9, amount: "165490982456158", ui_amount_string: "165490.982456158" }, owner:
 Some("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ"), program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") },
 UiTransactionTokenBalance { account_index: 9, mint: "q2y4FENF5cFdX95e2hg6e2MRdxkfeiF3PUHtw1ypump", ui_token_amount: UiTokenAmount
 { ui_amount: None, decimals: 9, amount: "0", ui_amount_string: "0" }, owner: Some("87nRYXqKArSLrotSCWXRLkCnk5jVfiQVo3HqjLT31VeK"),
  program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }, UiTransactionTokenBalance { account_index: 10, mint:
  "JBFZxVNNMrR6prECdMWbSQXMUtGjYRYy61psjgQdm5jU", ui_token_amount: UiTokenAmount { ui_amount: Some(397320.90979104), decimals: 9,
  amount: "397320909791040", ui_amount_string: "397320.90979104" }, owner: Some("87nRYXqKArSLrotSCWXRLkCnk5jVfiQVo3HqjLT31VeK"),
  program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }]), rewards: Some([]), loaded_addresses: Some(UiLoadedAddresses
  { writable: [], readonly: [] }), return_data: Skip, compute_units_consumed: Some(192285) }), version: Some(Number(0)) }, block_time: Some(1732807470) }
*/
/*
let encoded_confirmed_transaction_with_status_meta = rpc_client.get_transaction_with_config(
    &signature,
    RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Base58),
        commitment: Some(
            CommitmentConfig {
                commitment: CommitmentLevel::Finalized
            }
        ),
        max_supported_transaction_version: Some(0),
    }
)?;
println!("{:?}", encoded_confirmed_transaction_with_status_meta);

OUTPUT:
EncodedConfirmedTransactionWithStatusMeta { slot: 304155378, transaction: EncodedTransactionWithStatusMeta { transaction:
Binary("2TsTSahjz9GqcSRu8QqWKxtb612Z6SkBHyrrQHxmbRAWwKztaXJUbQDeZuw3Zf86pFxv21t3G4Up8E499shqjComCCBVrALSFGBTzkVRD68kijxDo823pj66dmE8pwzBpA
REJ2ajsV7A1WStf8fk9sq2eZekeBehZE3dDmQApWVBHYJYmBwq6pfV25CgrxaRT3wfZSDau5gPeAGxz7M6Jioezbu7LnB5gkJh5Gf8K9dQnQKxoNkhSthxgEfAVP5HTLhFKGirLea6
MUjbopPSaeErEuBU4vb42ukqMdru4Y6eADQ19X48J53SL7UMJP8yacKn2BWgiB7YDw4amrq3TB3ehL8qPt82a7hm1QgYQwt1QMd52ufrB7oTpTxUQzRWd6iuAQdBMHNBGMK3HA5Lwg
6T4wWbHYoW23j7tHc6SiabwQZsZAyN4n5NSRhHvbK8NdbeuVxHvWmkCyiDm8MXNsKBF5j4sVEgBwjWN4kB7sAgDrDKCJS5LauuTwoB6qwVzccXTZoTWbwVCAT3EYsvV7iyNNqPse53
Mi3T3DSQ8N9GaqrvkfDHNv4c1XB1uxJ9P1p6skpc3uoU95a177r455zzaB9rYdfuztvpuhguCL5VQzPVMbj3VPwzU2Ta2BCNb5MiZjVuYAGVcjakEKwJnwA9htzGNA1JL5fnLnoL2t
rW2KxTXvaNej9wjiGmEw4ykM3FmD8Ri7qtZuVaMBA8MABrhGH8KL4G7TjfbvcFtJQA7kDscJ6MwtPMgK8Ts6LYMZPnoTh9Ep8rFtDBvkWFhALun6fVVJGeeajksMM33ydiJCC9oQAe
cdC6DUik23KQcW2TVQRgnVe5v7Zz6hqvZFc61RoRanYYPxZkNnLFeP8r2U5pQrEmSemkYbMz22JyxiPqMm7Fzar113zAicVbmqNfjygF3SeentNzDSWFkqQjzZ6WoLQ3K1L4BiSrSi
UcUx1acAm1ysh69h7Vn6KM7chPGEDj5NrsPomgQeDNfLG27oAsojQS1LjKbCierrNeEf6b3CG92mFe4xMMAxcpdhweUTKviHEgFLqnHaZ4nC9QFJU7cD49pMcegKoU4JwhDXEyCpDV
f319fccNgjwpAWoPbMs7sdTBgEsLn9Um66zeaiwYoCm5RruGKnbq7xy98PpzhMwHtXgSVzq27BtZS2TFovYBoxfx7N1cs6QMyaiq6AoKAnLRv394kUgry8P222VwxyNbTkyDwTxZk1
SsjSLSCGJUy2frt1Y9LsckDk15shD5jkYPWXbs78vYxz7bVKBKsGFCPk6DcATfAEL2tqGVJCk1dDGVGQDMYBNwwQFWgwPvrAf6vkeEY7Ht9vnrcKXL1gh9Hqarf2MDWbvcoXro", Base58),
meta: Some(UiTransactionStatusMeta { err: None, status: Ok(()), fee: 5000, pre_balances: [764655925380, 0, 0, 0, 0, 0, 0, 0, 165490584495438,
2039280, 0, 1, 934087680, 735562554318, 1009200, 1141440, 731913600, 13412421853, 1461600, 4677120, 1141440, 3591360], post_balances: [1202599820,
0, 6124800, 23357760, 1461600, 2039280, 763002039280, 16258560, 165490984495438, 2039280, 2039280, 1, 934087680, 735562554318, 1009200, 1141440,
731913600, 13412421853, 1461600, 4677120, 1141440, 3591360], inner_instructions: Some([UiInnerInstructions { index: 2, instructions:
[Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 8], data: "3Bxs3zwhE1jnACsh", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 12, accounts: [8], data: "J", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts:
[0, 7], data: "3Bxs3zsXjYXEcY9D", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [7], data:
"9krTDTC9CyNDTCP9", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [7], data: "SYXsG5gxn13RGVJBuJ66WMvnpkuC3ZXmxCAkmzi1nLhi459e",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 4], data: "3Bxs4GxuFmUxu9wu", stack_height: Some(2) }),
Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [4], data: "9krTDE99A3SWNSHd", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 11, accounts: [4], data: "SYXsBSQy3GeifSEQSGvTbrPNposbSAiSoh1YA85wcvGKSnYg", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 12, accounts: [4, 14], data: "1D8qpeSmcAZXbhY6jAPqguwXxxrrFAnmcbUaH5dxdLLS3Ub", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 11, accounts: [0, 5], data: "3Bxs4h24hBtQy9rw", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11,
accounts: [5], data: "9krTDU2LzCSUJuVZ", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [5], data:
"SYXsBSQy3GeifSEQSGvTbrPNposbSAiSoh1YA85wcvGKSnYg", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [5, 18,
17, 14], data: "2", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 6], data: "3Bxs4h24hBtQy9rw",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [6], data: "9krTDU2LzCSUJuVZ", stack_height: Some(2) }),
Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [6], data: "SYXsBSQy3GeifSEQSGvTbrPNposbSAiSoh1YA85wcvGKSnYg", stack_height: Some(2)
}), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [6, 13, 17, 14], data: "2", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 11, accounts: [0, 2], data: "3Bxs3zw7D1St6MtB", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11,
accounts: [2], data: "9krTDga1qCiqxLs9", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [2], data:
"SYXsG5gxn13RGVJBuJ66WMvnpkuC3ZXmxCAkmzi1nLhi459e", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 3],
data: "3Bxs4BdXwcxHpZ19", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [3], data: "9krTDSXVJqcrnRvf",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [3], data: "SYXsBrTzDsq3kLD1BhH4w6jQTUs6sbwfa7yN5CyH8syhMbj3",
stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 20, accounts: [3, 17, 21, 14], data: "1PEpEB", stack_height: Some(2) }),
Compiled(UiCompiledInstruction { program_id_index: 16, accounts: [0, 10, 0, 4, 11, 12], data: "1", stack_height: Some(2) }), Compiled(UiCompiledInstruction
{ program_id_index: 12, accounts: [4], data: "84eT", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index: 11, accounts: [0, 10], data:
"11119os1e9qSs2u7TsThXqkBSRVFxhmYaFKFZ1waB2X7armDmvK3p5GmLdUxYdg3h7QSrL", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index: 12,
accounts: [10], data: "P", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [10, 4], data:
"6UFV6UjN7ThiB8HyxHfikpieBEyKK5TgCVxThDv8rJe5H", stack_height: Some(3) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [9, 5, 0],
data: "3DVzL1qggAfT", stack_height: Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [1, 6, 0], data: "3DU78ZaeqheF", stack_height:
Some(2) }), Compiled(UiCompiledInstruction { program_id_index: 12, accounts: [4, 10, 17], data: "6MYaS7nsq4yd", stack_height: Some(2) })] }]), log_messages:
Some(["Program 11111111111111111111111111111111 invoke [1]", "Program 11111111111111111111111111111111 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
invoke [1]", "Program log: Instruction: InitializeAccount", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3443 of 799850 compute units",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 invoke [1]",
"Program log: initialize2: InitializeInstruction2 { nonce: 254, open_time: 1732807457, init_pc_amount: 763000000000, init_coin_amount: 206900000000000000 }",
"Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
invoke [2]", "Program log: Instruction: SyncNative", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3045 of 778963 compute units", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]",
"Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]",
"Program 11111111111111111111111111111111 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction:
InitializeMint", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2920 of 748273 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111
invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111
success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log: Instruction: InitializeAccount", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
consumed 4475 of 733771 compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 11111111111111111111111111111111 invoke [2]",
"Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success",
"Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
invoke [2]", "Program log: Instruction: InitializeAccount", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 3445 of 714695 compute units",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111
success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111
invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111
success", "Program 11111111111111111111111111111111 invoke [2]", "Program 11111111111111111111111111111111 success", "Program 11111111111111111111111111111111
invoke [2]", "Program 11111111111111111111111111111111 success", "Program srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX invoke [2]", "Program
srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX consumed 2319 of 684694 compute units", "Program srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX success",
"Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]", "Program log: Create", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
"Program log: Instruction: GetAccountDataSize", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1595 of 670831 compute units",
"Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
"Program 11111111111111111111111111111111 invoke [3]", "Program 11111111111111111111111111111111 success", "Program log: Initialize the associated
token account", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]", "Program log: Instruction: InitializeImmutableOwner", "Program
log: Please upgrade to SPL Token 2022 for immutable owner support", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 664218
compute units", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]",
"Program log: Instruction: InitializeAccount3", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4214 of 660336 compute units",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 20389 of 676228
compute units", "Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
"Program log: Instruction: Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 652851 compute units", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log:
Instruction: Transfer", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4736 of 645248 compute units", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]", "Program log:
Instruction: MintTo", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4492 of 627855 compute units", "Program
TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success", "Program log: ray_log: ACGLSGcAAAAACQkA4fUFAAAAAADh9QUAAAAAAA5YprE
AAAAAQAcsdA7fAqAy5vwufyQb7XVEmbVElVzX4rS5eY47hgAW/v6eAilV", "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 consumed 185777
of 796407 compute units", "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 success", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
invoke [1]", "Program log: Instruction: CloseAccount", "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2915 of 610630 compute units",
"Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success"]), pre_token_balances: Some([UiTransactionTokenBalance { account_index: 8, mint:
"So11111111111111111111111111111111111111112", ui_token_amount: UiTokenAmount { ui_amount: Some(165490.582456158), decimals: 9, amount:
"165490582456158", ui_amount_string: "165490.582456158" }, owner: Some("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ"), program_id:
Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }, UiTransactionTokenBalance { account_index: 9, mint: "q2y4FENF5cFdX95e2hg6e2MRdxkfeiF3PUHtw1ypump",
ui_token_amount: UiTokenAmount { ui_amount: Some(206900000.0), decimals: 9, amount: "206900000000000000", ui_amount_string: "206900000" },
owner: Some("87nRYXqKArSLrotSCWXRLkCnk5jVfiQVo3HqjLT31VeK"), program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }]),
post_token_balances: Some([UiTransactionTokenBalance { account_index: 5, mint: "q2y4FENF5cFdX95e2hg6e2MRdxkfeiF3PUHtw1ypump",
ui_token_amount: UiTokenAmount { ui_amount: Some(206900000.0), decimals: 9, amount: "206900000000000000", ui_amount_string: "206900000" },
owner: Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"), program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") },
UiTransactionTokenBalance { account_index: 6, mint: "So11111111111111111111111111111111111111112", ui_token_amount: UiTokenAmount {
ui_amount: Some(763.0), decimals: 9, amount: "763000000000", ui_amount_string: "763" }, owner: Some("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"),
program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }, UiTransactionTokenBalance { account_index: 8, mint:
"So11111111111111111111111111111111111111112", ui_token_amount: UiTokenAmount { ui_amount: Some(165490.982456158), decimals: 9,
amount: "165490982456158", ui_amount_string: "165490.982456158" }, owner: Some("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ"),
program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }, UiTransactionTokenBalance { account_index: 9, mint:
"q2y4FENF5cFdX95e2hg6e2MRdxkfeiF3PUHtw1ypump", ui_token_amount: UiTokenAmount { ui_amount: None, decimals: 9, amount: "0", ui_amount_string:
"0" }, owner: Some("87nRYXqKArSLrotSCWXRLkCnk5jVfiQVo3HqjLT31VeK"), program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") },
UiTransactionTokenBalance { account_index: 10, mint: "JBFZxVNNMrR6prECdMWbSQXMUtGjYRYy61psjgQdm5jU", ui_token_amount: UiTokenAmount {
ui_amount: Some(397320.90979104), decimals: 9, amount: "397320909791040", ui_amount_string: "397320.90979104" }, owner:
Some("87nRYXqKArSLrotSCWXRLkCnk5jVfiQVo3HqjLT31VeK"), program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") }]),
rewards: Some([]), loaded_addresses: Some(UiLoadedAddresses { writable: [], readonly: [] }), return_data: Skip, compute_units_consumed:
Some(192285) }), version: Some(Number(0)) }, block_time: Some(1732807470) }
*/