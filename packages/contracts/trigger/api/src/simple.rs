use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Empty, Uint64};

pub type InstantiateMsg = Empty;

#[cw_serde]
#[schemaifier(mute_warnings)]
pub enum ExecuteMsg {
    Push { message: String },
}

#[cw_serde]
#[schemaifier(mute_warnings)]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    TriggerMessage { trigger_id: Uint64 },
}

pub struct PushMessageEvent {
    pub trigger_id: Uint64,
}

impl PushMessageEvent {
    pub const EVENT_TYPE: &'static str = "push-message";
    pub const EVENT_ATTR_KEY_TRIGGER_ID: &'static str = "trigger-id";
}

impl From<PushMessageEvent> for cosmwasm_std::Event {
    fn from(src: PushMessageEvent) -> Self {
        cosmwasm_std::Event::new(PushMessageEvent::EVENT_TYPE).add_attribute(
            PushMessageEvent::EVENT_ATTR_KEY_TRIGGER_ID,
            src.trigger_id.to_string(),
        )
    }
}

impl TryFrom<&cosmwasm_std::Event> for PushMessageEvent {
    type Error = anyhow::Error;

    fn try_from(event: &cosmwasm_std::Event) -> Result<Self, Self::Error> {
        if event.ty != Self::EVENT_TYPE && event.ty != format!("wasm-{}", Self::EVENT_TYPE) {
            return Err(anyhow::anyhow!(
                "Expected event type {}, found {}",
                Self::EVENT_TYPE.to_string(),
                event.ty.to_string()
            ));
        }

        let trigger_id = event
            .attributes
            .iter()
            .find(|attr| attr.key == Self::EVENT_ATTR_KEY_TRIGGER_ID)
            .map(|attr| attr.value.to_string())
            .ok_or_else(|| {
                anyhow::anyhow!("Missing attribute {}", Self::EVENT_ATTR_KEY_TRIGGER_ID)
            })?;

        let trigger_id = trigger_id.parse::<u64>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid attribute {}: {}",
                Self::EVENT_ATTR_KEY_TRIGGER_ID,
                trigger_id
            )
        })?;

        Ok(Self {
            trigger_id: trigger_id.into(),
        })
    }
}
