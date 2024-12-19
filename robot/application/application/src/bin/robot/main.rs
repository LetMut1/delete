mod capture;
mod command_processor;
mod environment_configuration;
mod error;
mod grpc_server;
mod http_server;
mod extern_source;
mod robot;
mod spawner;
mod transaction_parser;
mod workflow_data;
use clap::{
    Arg,
    Command,
};
use self::error::{
    Error,
    Backtrace,
    OptionConverter,
    Common,
};
use self::command_processor::{
    CommandProcessor,
    ParseTransaction,
    Trade,
};
fn main() -> Result<(), Error> {
    Processor::process()
}
struct Processor;
impl Processor {
    fn process() -> Result<(), Error> {
        const COMMAND_TRADE: &'static str = "trade";
        const COMMAND_PARSE_TRANSACTION: &'static str = "parse_transaction";
        const ARGUMENT_ENVIRONMENT_FILE_PATH: &'static str = "environment_configuration_file_path";
        let arg_matches = clap::command!()
            .arg_required_else_help(true)
            .arg(Arg::new(ARGUMENT_ENVIRONMENT_FILE_PATH).required(true).long(ARGUMENT_ENVIRONMENT_FILE_PATH))
            .subcommand_required(true)
            .subcommand(Command::new(COMMAND_TRADE))
            .subcommand(Command::new(COMMAND_PARSE_TRANSACTION))
            .get_matches();
        let environment_configuration_file_path = arg_matches.get_one::<String>(ARGUMENT_ENVIRONMENT_FILE_PATH).into_unreachable_state(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?;
        let subcommand_arg_matches = arg_matches.subcommand().into_unreachable_state(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?;
        match subcommand_arg_matches {
            (COMMAND_TRADE, _) => CommandProcessor::<Trade>::process(environment_configuration_file_path.as_str()),
            (COMMAND_PARSE_TRANSACTION, _) => CommandProcessor::<ParseTransaction>::process(environment_configuration_file_path.as_str()),
            _ => {
                Result::Err(
                    Error::new_(
                        Common::UnreachableState,
                        Backtrace::new(
                            line!(),
                            file!(),
                        ),
                    ),
                )
            }
        }
    }
}