use crate::{MockServiceHandlerExecClient, MockServiceHandlerQueryClient, MockServiceManagerExecClient, MockServiceManagerQueryClient};

// Convenience wrapper, automatically implemented
pub trait MockServiceClient {
    type MockServiceHandlerClient: MockServiceHandlerExecClient + MockServiceHandlerQueryClient;
    type MockServiceManagerClient: MockServiceManagerExecClient + MockServiceManagerQueryClient;

    fn mock_handler(&self) -> &Self::MockServiceHandlerClient;
    fn mock_manager(&self) -> &Self::MockServiceManagerClient;
}

// blanket implementations
impl <T> MockServiceClient for T
where
    T: MockServiceHandlerExecClient + MockServiceHandlerQueryClient + MockServiceManagerExecClient + MockServiceManagerQueryClient,
{
    type MockServiceHandlerClient = T;
    type MockServiceManagerClient = T;

    fn mock_handler(&self) -> &Self::MockServiceHandlerClient {
        self
    }

    fn mock_manager(&self) -> &Self::MockServiceManagerClient {
        self
    }
}