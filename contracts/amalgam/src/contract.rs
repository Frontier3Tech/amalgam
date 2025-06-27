#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use amalgam_utils::tokenfactory::{self, TFToken};

use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> Result<Response, ContractError> {
  set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

  STATE.save(deps.storage, &State {
    admin: info.sender.to_string(),
  })?;

  let tftoken = get_tftoken(&env);

  Ok(Response::new()
    .add_attribute("method", "instantiate")
    .add_messages(tftoken.create())
    .add_messages(tftoken.set_metadata(msg.metadata))
  )
}

pub fn get_tftoken(env: &Env) -> impl TFToken {
  tokenfactory::osmosis::TFToken::new(
    env.contract.address.clone(),
    "amalgam".to_string(),
  )
}
