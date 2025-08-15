use crate::{ServiceHandlerExecClient, ServiceHandlerQueryClient, ServiceManagerExecClient, ServiceManagerQueryClient};

// Convenience wrapper, automatically implemented
pub trait ServiceClient {
    type ServiceHandlerClient: ServiceHandlerExecClient + ServiceHandlerQueryClient;
    type ServiceManagerClient: ServiceManagerExecClient + ServiceManagerQueryClient;

    fn handler(&self) -> &Self::ServiceHandlerClient;
    fn manager(&self) -> &Self::ServiceManagerClient;
}

// blanket implementations
impl <T> ServiceClient for T
where
    T: ServiceHandlerExecClient + ServiceHandlerQueryClient + ServiceManagerExecClient + ServiceManagerQueryClient,
{
    type ServiceHandlerClient = T;
    type ServiceManagerClient = T;

    fn handler(&self) -> &Self::ServiceHandlerClient {
        self
    }

    fn manager(&self) -> &Self::ServiceManagerClient {
        self
    }
}