use std::sync::OnceLock;
use tokio::runtime::{
    Builder as RuntimeBuilder,
    Runtime,
};
use super::robot::Robot;
use tracing::Level;
use tracing_appender::non_blocking::{
    NonBlocking,
    NonBlockingBuilder,
    WorkerGuard,
};
use super::environment_configuration::Trade as Trade_;
use super::transaction_parser::TransactionParser;
use super::error::{
    Error,
    Backtrace,
    OptionConverter,
    ResultConverter,
    Common,
};
use super::environment_configuration::{
    EnvironmentConfiguration,
    Loader,
};
#[cfg(feature = "logging_to_file")]
use tracing_appender::rolling::{
    RollingFileAppender,
    Rotation,
};
use tracing_subscriber::FmtSubscriber;
use std::marker::PhantomData;
static ENVIRONMENT_CONFIGURATION: OnceLock<EnvironmentConfiguration<Trade_>> = OnceLock::new();
pub struct CommandProcessor<S> {
    _subject: PhantomData<S>,
}
pub struct Trade;
impl CommandProcessor<Trade> {
    pub fn process<'a>(environment_configuration_file_path: &'a str) -> Result<(), Error> {
        let _worker_guard;
        let environment_configuration = Self::initialize_environment(environment_configuration_file_path)?;
        #[cfg(feature = "logging_to_file")]
        {
            _worker_guard = Self::initialize_logging_to_fileger(environment_configuration)?;
        }
        #[cfg(not(feature = "logging_to_file"))]
        {
            _worker_guard = initialize_stdout_logger();
        }
        let runtime = Self::initialize_runtime(environment_configuration)?;
        runtime.block_on(Robot::start(environment_configuration))?;
        Ok(())
    }
    fn initialize_environment<'a>(environment_configuration_file_path: &'a str) -> Result<&'static EnvironmentConfiguration<Trade_>, Error> {
        let environment_configuration = Loader::<Trade>::load(environment_configuration_file_path)?;
        match ENVIRONMENT_CONFIGURATION.get() {
            Some(environment_configuration__) => Result::Ok(environment_configuration__),
            None => {
                if ENVIRONMENT_CONFIGURATION.set(environment_configuration).is_err() {
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
                ENVIRONMENT_CONFIGURATION.get().into_value_does_not_exist(
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                )
            }
        }
    }
    #[cfg(feature = "logging_to_file")]
    fn initialize_logging_to_fileger<'a>(environment_configuration: &'a EnvironmentConfiguration<Trade_>) -> Result<WorkerGuard, Error> {
        let rolling_file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            environment_configuration.subject.logging.directory_path.as_str(),
            environment_configuration.subject.logging.file_name_prefix.as_str(),
        );
        let (non_blocking, worker_guard) = NonBlockingBuilder::default().finish(rolling_file_appender);
        initialize_tracing_subscriber(non_blocking)?;
        Ok(worker_guard)
    }
    fn initialize_runtime<'a>(environment_configuration: &'a EnvironmentConfiguration<Trade_>) -> Result<Runtime, Error> {
        if environment_configuration.subject.tokio_runtime.maximum_blocking_threads_quantity == 0
            || environment_configuration.subject.tokio_runtime.worker_threads_quantity == 0
            || environment_configuration.subject.tokio_runtime.worker_thread_stack_size < (1024 * 1024)
        {
            return Err(
                Error::new(
                    "Invalid Tokio runtime configuration.".into(),
                    Backtrace::new(
                        line!(),
                        file!(),
                    ),
                ),
            );
        }
        RuntimeBuilder::new_multi_thread()
            .max_blocking_threads(environment_configuration.subject.tokio_runtime.maximum_blocking_threads_quantity)
            .worker_threads(environment_configuration.subject.tokio_runtime.worker_threads_quantity)
            .thread_stack_size(environment_configuration.subject.tokio_runtime.worker_thread_stack_size)
            .enable_all()
            .build()
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )
    }
}
pub struct ParseTransaction;
impl CommandProcessor<ParseTransaction> {
    pub fn process<'a>(environment_configuration_file_path: &'a str) -> Result<(), Error> {
        let environment_configuration = Loader::<ParseTransaction>::load(environment_configuration_file_path)?;
        let _worker_guard = initialize_stdout_logger();
        let runtime = Self::initialize_runtime()?;
        runtime.block_on(TransactionParser::parse(&environment_configuration))?;
        Ok(())
    }
    fn initialize_runtime() -> Result<Runtime, Error> {
        RuntimeBuilder::new_multi_thread()
            .max_blocking_threads(2)
            .worker_threads(2)
            .thread_stack_size(2 * 1024 * 1024)
            .enable_all()
            .build()
            .into_(
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            )
    }
}
fn initialize_stdout_logger() -> Result<WorkerGuard, Error> {
    let (non_blocking, worker_guard) = NonBlockingBuilder::default().finish(std::io::stdout());
    initialize_tracing_subscriber(non_blocking)?;
    Ok(worker_guard)
}
fn initialize_tracing_subscriber(non_blocking: NonBlocking) -> Result<(), Error> {
    let fmt_subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .with_file(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(fmt_subscriber).into_(
        Backtrace::new(
            line!(),
            file!(),
        ),
    )?;
    Ok(())
}