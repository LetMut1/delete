use self::environment_configuration_file::{
    ParseTransaction as ParseTransaction_,
    Run as Run_,
};
use std::path::Path;
use super::error::{
    Backtrace,
    ResultConverter,
    Error,
};
use serde::de::DeserializeOwned;
use self::run::{
    Geyser,
    Logging,
    TokioRuntime,
};
pub use self::run::Run;
pub use self::parse_transaction::ParseTransaction;
use std::marker::PhantomData;
use super::command_processor::{
    ParseTransaction as ParseTransaction__,
    Run as Run__,
};
pub struct EnvironmentConfiguration<T> {
    pub subject: T,
}
mod environment_configuration_file {
    pub use self::run::Run;
    pub use self::parse_transaction::ParseTransaction;
    pub mod run {
        use super::Value;
        #[derive(serde::Deserialize)]
        pub struct Run {
            pub tokio_runtime: TokioRuntime,
            pub logging: Logging,
            pub geyser: Geyser,
        }
        #[derive(serde::Deserialize)]
        pub struct TokioRuntime {
            pub maximum_blocking_threads_quantity: Value<usize>,
            pub worker_threads_quantity: Value<usize>,
            pub worker_thread_stack_size: Value<usize>,
        }
        #[derive(serde::Deserialize)]
        pub struct Logging {
            pub directory_path: Value<String>,
            pub file_name_prefix: Value<String>,
        }
        #[derive(serde::Deserialize)]
        pub struct Geyser {
            pub grpc_url: Value<String>,
        }
    }
    pub mod parse_transaction {
        use super::Value;
        #[derive(serde::Deserialize)]
        pub struct ParseTransaction {
            pub solana_cluster_url: Value<String>,
            pub solana_transaction_signature_registry: Value<Vec<String>>,
        }
    }
    #[derive(serde::Deserialize)]
    pub struct Value<T> {
        pub value: T,
    }
    #[derive(serde::Deserialize)]
    pub struct ValueExist<T> {
        pub value: T,
        pub is_exist: bool,
    }
}
mod run {
    pub struct Run {
        pub tokio_runtime: TokioRuntime,
        pub logging: Logging,
        pub geyser: Geyser,
    }
    pub struct TokioRuntime {
        pub maximum_blocking_threads_quantity: usize,
        pub worker_threads_quantity: usize,
        pub worker_thread_stack_size: usize,
    }
    pub struct Logging {
        pub directory_path: String,
        pub file_name_prefix: String,
    }
    pub struct Geyser {
        pub grpc_url: String,
    }
}
mod parse_transaction {
    pub struct ParseTransaction {
        pub solana_cluster_url: String,
        pub solana_transaction_signature_registry: Vec<String>,
    }

}
pub struct Loader<S> {
    _subject: PhantomData<S>,
}
impl Loader<Run__> {
    pub fn load<'a>(environment_configuration_file_path: &'a str) -> Result<EnvironmentConfiguration<Run>, Error> {
        let environment_configuration_file = load::<Run_>(environment_configuration_file_path)?;
        Result::Ok(
            EnvironmentConfiguration {
                subject: Run {
                    tokio_runtime: TokioRuntime {
                        maximum_blocking_threads_quantity: environment_configuration_file.tokio_runtime.maximum_blocking_threads_quantity.value,
                        worker_threads_quantity: environment_configuration_file.tokio_runtime.worker_threads_quantity.value,
                        worker_thread_stack_size: environment_configuration_file.tokio_runtime.worker_thread_stack_size.value,
                    },
                    logging: Logging {
                        directory_path: environment_configuration_file.logging.directory_path.value,
                        file_name_prefix: environment_configuration_file.logging.file_name_prefix.value,
                    },
                    geyser: Geyser {
                        grpc_url: environment_configuration_file.geyser.grpc_url.value,
                    },
                },
            },
        )
    }
}
impl Loader<ParseTransaction__> {
    pub fn load<'a>(environment_configuration_file_path: &'a str) -> Result<EnvironmentConfiguration<ParseTransaction>, Error> {
        let environment_configuration_file = load::<ParseTransaction_>(environment_configuration_file_path)?;
        Result::Ok(
            EnvironmentConfiguration {
                subject: ParseTransaction {
                    solana_cluster_url: environment_configuration_file.solana_cluster_url.value,
                    solana_transaction_signature_registry: environment_configuration_file.solana_transaction_signature_registry.value,
                },
            },
        )
    }
}
fn load<'a, T>(environment_configuration_file_path: &'a str) -> Result<T, Error>
where
    T: DeserializeOwned
{
    let environment_configuration_file_path_ = Path::new(environment_configuration_file_path);
    let environment_configuration_file_data = if environment_configuration_file_path_.try_exists().into_(
        Backtrace::new(
            line!(),
            file!(),
        ),
    )? {
        std::fs::read_to_string(environment_configuration_file_path_).into_(
            Backtrace::new(
                line!(),
                file!(),
            ),
        )?
    } else {
        return Result::Err(
            Error::new(
                "The environment.toml file does not exist.".into(),
                Backtrace::new(
                    line!(),
                    file!(),
                ),
            ),
        );
    };
    toml::from_str::<T>(environment_configuration_file_data.as_str()).into_(
        Backtrace::new(
            line!(),
            file!(),
        ),
    )
}