#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};

use crate::{msg::{ComponentsResponse, QueryMsg}, state::COMPONENTS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
  match msg {
    QueryMsg::Components {} => to_json_binary(&query_components(deps)?),
  }
}

fn query_components(deps: Deps) -> StdResult<ComponentsResponse> {
  let components = COMPONENTS
    .range(deps.storage, None, None, Order::Ascending)
    .map(|item| item.map(|(_, component)| component).unwrap())
    .collect();
  Ok(ComponentsResponse { components })
}
