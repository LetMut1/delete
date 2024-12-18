use super::error::{
    Backtrace,
    Error,
    Common,
    OptionConverter,
};
#[derive(Debug)]
pub struct RaydiumAmmInitializeInstruction2 {
    pub nonce: u8,
    pub open_time: u64,
    pub init_pc_amount: u64,
    pub init_coin_amount: u64,
}
impl RaydiumAmmInitializeInstruction2 {
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