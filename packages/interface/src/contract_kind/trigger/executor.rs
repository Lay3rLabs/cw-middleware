use async_trait::async_trait;
use cosmwasm_std::{Coin, Uint64};
use trigger_api::simple::{ExecuteMsg, PushMessageEvent};
use super::HasSimpleTriggerAddr;
use crate::{ExecClientExt, TxResponseExt};


#[async_trait(?Send)]
pub trait SimpleTriggerExecClient: ExecClientExt + HasSimpleTriggerAddr
{
    async fn simple_trigger_exec(
        &self,
        msg: &ExecuteMsg,
        funds: &[Coin],
    ) -> Result<Self::TxResponse, cosmwasm_std::StdError> {
        let contract_addr = HasSimpleTriggerAddr::addr(self);
        self.contract_exec(&contract_addr, msg, funds).await
    }

    // returns the trigger ID
    async fn push_message(&self, message: impl ToString) -> Result<Uint64, cosmwasm_std::StdError> {
        let msg = trigger_api::simple::ExecuteMsg::Push {
            message: message.to_string(),
        };
        let resp = self.simple_trigger_exec(&msg, &[]).await?;
        let events = resp.extract_events();

        let event = events
            .event_first_by_attr_key(
                PushMessageEvent::EVENT_TYPE,
                PushMessageEvent::EVENT_ATTR_KEY_TRIGGER_ID,
            )
            .map(cosmwasm_std::Event::from)
            .and_then(|e| PushMessageEvent::try_from(&e))
            .map_err(|err| cosmwasm_std::StdError::msg(err.to_string()))?;

        Ok(event.trigger_id)
    }
}

impl <T> SimpleTriggerExecClient for T
where
    T: ExecClientExt + HasSimpleTriggerAddr { }