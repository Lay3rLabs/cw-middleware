// Just a bunch of useful convenience wrappers, automatically implemented

use crate::{ServiceHandlerExecClient, ServiceHandlerQueryClient, ServiceManagerExecClient, ServiceManagerQueryClient, SimpleTriggerExecClient, SimpleTriggerQueryClient};

pub trait ServiceTriggerClient {
    type ServiceHandlerClient: ServiceHandlerExecClient + ServiceHandlerQueryClient;
    type ServiceManagerClient: ServiceManagerExecClient + ServiceManagerQueryClient;
    type TriggerClient: SimpleTriggerExecClient + SimpleTriggerQueryClient;

    fn handler(&self) -> &Self::ServiceHandlerClient;
    fn manager(&self) -> &Self::ServiceManagerClient;
    fn trigger(&self) -> &Self::TriggerClient;
}

impl <T> ServiceTriggerClient for T
where
T: ServiceHandlerExecClient + ServiceHandlerQueryClient + ServiceManagerExecClient + ServiceManagerQueryClient + SimpleTriggerExecClient + SimpleTriggerQueryClient
{
    type ServiceHandlerClient = T;
    type ServiceManagerClient = T;
    type TriggerClient = T;

    fn handler(&self) -> &Self::ServiceHandlerClient {
        self
    }

    fn manager(&self) -> &Self::ServiceManagerClient {
        self
    }

    fn trigger(&self) -> &Self::TriggerClient {
        self
    }
}