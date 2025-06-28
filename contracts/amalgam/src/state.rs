use cosmwasm_std::{coin, to_json_binary, Addr, BankMsg, CosmosMsg, Decimal, Deps, QuerierWrapper, Uint128, WasmMsg};
use cosmwasm_schema::cw_serde;
use cw20::{Cw20Contract, Cw20ExecuteMsg};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct State {
  /// Admin of the Amalgam contract. The only one who can add new tokens to the Amalgamation.
  pub admin: String,
}

#[cw_serde]
pub struct Component {
  pub token: Asset,
  pub weight: Decimal,
  /// In basis points.
  pub withdrawal_tax: u16,
}

#[cw_serde]
pub enum Asset {
  Native(String),
  Cw20(String),
}

impl Asset {
  pub fn key(&self) -> String {
    match self {
      Asset::Native(denom) => format!("native:{}", denom),
      Asset::Cw20(contract) => format!("cw20:{}", contract),
    }
  }

  pub fn send(&self, amount: Uint128, recipient: String) -> CosmosMsg {
    match self {
      Asset::Native(denom) =>
        BankMsg::Send { to_address: recipient, amount: vec![coin(amount.into(), denom.clone())] }.into(),
      Asset::Cw20(contract) =>
        WasmMsg::Execute {
          contract_addr: contract.clone(),
          msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.clone(),
            amount: amount.into(),
          }).unwrap(),
          funds: vec![],
        }.into(),
    }
  }

  pub fn balance(&self, deps: &Deps, querier: &QuerierWrapper, address: Addr) -> Uint128 {
    match self {
      Asset::Native(denom) => {
        let balance = querier.query_balance(address, denom);
        balance.map_or(Uint128::zero(), |balance| balance.amount)
      },
      Asset::Cw20(contract) => {
        let contract = deps.api.addr_validate(contract).unwrap();
        let contract = Cw20Contract(contract);
        contract.balance(querier, address).unwrap_or(Uint128::zero())
      },
    }
  }
}

pub const STATE: Item<State> = Item::new("state");
pub const COMPONENTS: Map<String, Component> = Map::new("components");
/// Map of asset keys to balances.
pub const BALANCES: Map<String, Uint128> = Map::new("balances");
