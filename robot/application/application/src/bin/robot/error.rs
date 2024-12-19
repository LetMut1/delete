use std::{
    error::Error as StdError,
    fmt::{
        Debug,
        Display,
        Error as FmtError,
        Formatter,
    },
};
#[derive(Debug)]
pub struct Error(pub Auditor<Box<dyn StdError + Send + Sync + 'static>>);
impl Error {
    pub fn new(error: Box<dyn StdError + Send + Sync + 'static>, backtrace: Backtrace) -> Self {
        Self(
            Auditor {
                subject: error,
                backtrace,
            },
        )
    }
    pub fn new_(common: Common, backtrace: Backtrace) -> Self {
        Self(
            Auditor {
                subject: common.into(),
                backtrace,
            },
        )
    }
}
impl Display for Error {
    fn fmt<'a, 'b>(&'a self, formatter: &'b mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            formatter,
            "{}\n{}:{}",
            self.0.subject,
            self.0.backtrace.file_path,
            self.0.backtrace.line_number,
        )
    }
}
impl StdError for Error {}
#[derive(Debug)]
pub struct Auditor<T> {
    pub subject: T,
    pub backtrace: Backtrace,
}
#[derive(Debug)]
pub struct Backtrace {
    pub line_number: u32,
    pub file_path: &'static str,
}
impl Backtrace {
    pub fn new(line_number: u32, file_path: &'static str) -> Self {
        Self {
            line_number,
            file_path,
        }
    }
}
pub trait ResultConverter<T> {
    fn into_(self, backtrace: Backtrace) -> Result<T, Error>;
}
impl<T, E> ResultConverter<T> for Result<T, E>
where
    E: StdError + Sync + Send + 'static
{
    fn into_(self, backtrace: Backtrace) -> Result<T, Error> {
        self.map_err(
            move |error: _| -> _ {
                return Error::new(
                    error.into(),
                    backtrace,
                );
            },
        )
    }
}
pub trait OptionConverter<T> {
    fn into_unreachable_state(self, backtrace: Backtrace) -> Result<T, Error>;
    fn into_value_does_not_exist(self, backtrace: Backtrace) -> Result<T, Error>;
    fn into_out_of_range(self, backtrace: Backtrace) -> Result<T, Error>;
}
impl<T> OptionConverter<T> for Option<T> {
    fn into_out_of_range(self, backtrace: Backtrace) -> Result<T, Error> {
        self.ok_or(
            Error::new_(
                Common::OutOfRange,
                backtrace,
            ),
        )
    }
    fn into_unreachable_state(self, backtrace: Backtrace) -> Result<T, Error> {
        self.ok_or(
            Error::new_(
                Common::UnreachableState,
                backtrace,
            ),
        )
    }
    fn into_value_does_not_exist(self, backtrace: Backtrace) -> Result<T, Error> {
        self.ok_or(
            Error::new_(
                Common::ValueDoesNotExist,
                backtrace,
            ),
        )
    }
}
#[derive(Debug)]
pub enum Common {
    OutOfRange,
    UnreachableState,
    ValueAlreadyExist,
    ValueDoesNotExist,
}
impl Display for Common {
    fn fmt<'a>(&'a self, formatter: &'a mut Formatter<'_>) -> Result<(), FmtError> {
        let message = match *self {
            Self::OutOfRange => "Out of range.",
            Self::UnreachableState => "Unreachable state.",
            Self::ValueAlreadyExist => "Value already exist.",
            Self::ValueDoesNotExist => "Value does not exist.",
        };
        write!(formatter, "{}", message)
    }
}
impl StdError for Common {}