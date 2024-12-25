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
    pub const RAYDIUM_LIQUIDITY_POOL_V4_LOG_PATTERN: &'static [u8] = b"Program log: initialize2: InitializeInstruction2".as_slice();
    pub const RAYDIUM_LIQUIDITY_POOL_V4_LOG_VECTOR_INDEX: usize = 7;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_INITIALIZE_2_INSTRUCTION_VECTOR_INDEX: usize = 2;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTIONS_QUANTITY: usize = 32;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_AMM_MARKET_PUBKEY_VECTOR_INDEX: usize = 4;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_AMM_COIN_VAULT_PUBKEY_VECTOR_INDEX: usize = 10;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_AMM_PC_VAULT_PUBKEY_VECTOR_INDEX: usize = 11;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_INNER_INSTRUCTION_VECTOR_INDEX: usize = 0;
    pub const RAYDIUM_LIQUIDITY_POOL_V4_PC_VAULT_TOKEN_ACCOUNT_INITIALIZING_INSTRUCTION_VECTOR_INDEX: usize = 16;
    pub const INSTRUCTIONS_QUANTITY: usize = 4;
    pub const INSTRUCTIONS_WITH_INNER_INSTRUCTIONS_QUANTITY: usize = 1;
    // 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' pubkey.
    pub const TOKEN_PROGRAM_PUBKEY: Pubkey = Pubkey::new_from_array(
        [6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169]
    );
    // 'So11111111111111111111111111111111111111112' pubkey.
    pub const WRAPPED_SOL_TOKEN_ACCOUNT_PUBKEY: Pubkey = Pubkey::new_from_array(
        [6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53, 218, 196, 57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1]
    );
}