use super::error::{
    Backtrace,
    Error,
    Common,
    OptionConverter,
};
use uint::construct_uint;
construct_uint! {
    pub struct U128(2);
}
// https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/instruction.rs#L29
#[repr(C)]
#[derive(Debug)]
pub struct RaydiumAmmInitializeInstruction2 {
    pub nonce: u8,
    pub open_time: u64,
    pub init_pc_amount: u64,
    pub init_coin_amount: u64,
}
impl RaydiumAmmInitializeInstruction2 {
    // https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/instruction.rs#L386
    pub fn unpack<'a>(input: &'a [u8]) -> Result<Self, Error> {
        let (tag, rest) = input
            .split_first()
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
        match *tag {
            1 => {
                let (nonce, rest) = Self::unpack_u8(rest)?;
                let (open_time, rest) = Self::unpack_u64(rest)?;
                let (init_pc_amount, rest) = Self::unpack_u64(rest)?;
                let (init_coin_amount, _reset) = Self::unpack_u64(rest)?;
                Ok(
                    RaydiumAmmInitializeInstruction2 {
                        nonce,
                        open_time,
                        init_pc_amount,
                        init_coin_amount,
                    }
                )
            }
            _ => Err(
                Error::new_(
                    Common::ValueDoesNotExist,
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )
            )
        }
    }
    // https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/instruction.rs#L603
    fn unpack_u8<'a>(input: &'a [u8]) -> Result<(u8, &'a [u8]), Error> {
        if input.len() >= 1 {
            let (amount, rest) = input.split_at(1);
            let amount = amount
            .get(..1)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            Ok((amount, rest))
        } else {
            Err(
                Error::new_(
                    Common::ValueDoesNotExist,
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )
            )
        }
    }
    // https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/instruction.rs#L631
    fn unpack_u64<'a>(input: &'a [u8]) -> Result<(u64, &'a [u8]), Error> {
        if input.len() >= 8 {
            let (amount, rest) = input.split_at(8);
            let amount = amount
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .into_value_does_not_exist(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )?;
            Ok((amount, rest))
        } else {
            Err(
                Error::new_(
                    Common::ValueDoesNotExist,
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )
            )
        }
    }
}
pub struct Calcaulator;
impl Calcaulator {
    // https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/state.rs#L518
    const AMM_SWAP_FEE: AmmSwapFee = AmmSwapFee {
        numerator: 25,
        denominator: 10000,
    };
    pub fn get_coin_amount_from_pc_amount(
        pc_amount: u64,
        total_pc_amount_without_take_pnl: u64,
        total_coin_amount_without_take_pnl: u64,
    ) -> Result<U128, Error> {
        // https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/processor.rs#L2393
        let swap_fee_pc_amount = U128::from(pc_amount)
        .checked_mul(Self::AMM_SWAP_FEE.numerator.into())
        .into_out_of_range(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?
        .checked_ceil_div(Self::AMM_SWAP_FEE.denominator.into())
        .into_out_of_range(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?
        .0;
        let pc_amount_after_taking_swap_fee = U128::from(pc_amount)
        .checked_sub(swap_fee_pc_amount)
        .into_out_of_range(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?;
        // https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/math.rs#L401
        let denominator = U128::from(total_pc_amount_without_take_pnl)
        .checked_add(pc_amount_after_taking_swap_fee)
        .into_out_of_range(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?;
        U128::from(total_coin_amount_without_take_pnl)
        .checked_mul(pc_amount_after_taking_swap_fee)
        .into_out_of_range(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?
        .checked_div(denominator)
        .into_out_of_range(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )
    }
}
struct AmmSwapFee {
    numerator: u64,
    denominator: u64,
}
// https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/math.rs#L589
trait CheckedCeilDiv: Sized {
    fn checked_ceil_div<'a>(&'a self, rhs: Self) -> Option<(Self, Self)>;
}
// https://github.com/raydium-io/raydium-amm/blob/d10a8e9fab9f7a3d87b4ae3891e3e4c24b75c041/program/src/math.rs#L594
impl CheckedCeilDiv for U128 {
    fn checked_ceil_div<'a>(&'a self, mut rhs: Self) -> Option<(Self, Self)> {
        let mut quotient = self.checked_div(rhs)?;
        // Avoid dividing a small number by a big one and returning 1, and instead
        // fail.
        let zero = U128::from(0);
        let one = U128::from(1);
        if quotient.is_zero() {
            // return None;
            if self.checked_mul(U128::from(2))? >= rhs {
                return Some((one, zero));
            } else {
                return Some((zero, zero));
            }
        }

        // Ceiling the destination amount if there's any remainder, which will
        // almost always be the case.
        let remainder = self.checked_rem(rhs)?;
        if remainder > zero {
            quotient = quotient.checked_add(one)?;
            // calculate the minimum amount needed to get the dividend amount to
            // avoid truncating too much
            rhs = self.checked_div(quotient)?;
            let remainder = self.checked_rem(quotient)?;
            if remainder > zero {
                rhs = rhs.checked_add(one)?;
            }
        }
        Some((quotient, rhs))
    }
}