use std::marker::PhantomData;
use solana_sdk::pubkey::Pubkey;
pub struct WorkflowData<S> {
    _subject: PhantomData<S>
}
pub struct TransactionDifferentiation;
impl WorkflowData<TransactionDifferentiation> {
    // '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8' pubkey.
    pub const RAYDIUM_LIQUIDITY_POOL_V4_CONTRACT_PUBKEY: Pubkey = Pubkey::new_from_array(
        [75, 217, 73, 196, 54, 2, 195, 63, 32, 119, 144, 237, 22, 163, 82, 76, 161, 185, 151, 92, 241, 33, 162, 169, 12, 255, 236, 125, 248, 182, 138, 205]
    );
    pub const RAYDIUM_LIQUIDITY_POOL_V4_LOG_PATTERN: &'static str = r#"\binitialize2: InitializeInstruction2\b"#;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_LOG_VECTOR_INDEX: usize = 7;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_INSTRUCTION_VECTOR_INDEX: usize = 2;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTIONS_QUANTITY: usize = 32;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_AMM_MARKET_PUBKEY_VECTOR_INDEX: usize = 4;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_AMM_COIN_VAULT_PUBKEY_VECTOR_INDEX: usize = 10;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_AMM_PC_VAULT_PUBKEY_VECTOR_INDEX: usize = 11;
    pub const INSTRUCTIONS_QUANTITY: usize = 4;
    pub const INSTRUCTIONS_WITH_INNER_INSTRUCTIONS_QUANTITY: usize = 1;
}