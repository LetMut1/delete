use super::error::Error;
use std::future::Future;
use tokio::task::JoinHandle;
pub struct Spawner;
impl Spawner {
    pub fn spawn_tokio_non_blocking_task_into_background<F, T>(future: F) -> ()
    where
        F: Future<Output = Result<T, Error>> + Send + 'static,
    {
        tokio::spawn(
            async move {
                if let Err(error) = future.await {
                    tracing::error!("{}", &error);
                }
            },
        );
    }
    pub fn spawn_tokio_non_blocking_task_processed<F>(future: F) -> JoinHandle<<F as Future>::Output>
    where
        F: Future + Send + 'static,
        <F as Future>::Output: Send + 'static,
    {
        tokio::spawn(future)
    }
}