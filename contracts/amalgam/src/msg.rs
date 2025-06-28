use amalgam_utils::tokenfactory::DenomMetadata;
use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{Asset, Component};

#[cw_serde]
pub struct InstantiateMsg {
  /// Admin of the Amalgam contract. The only one who can add new tokens to the Amalgamation.
  pub admin: String,
  /// Metadata of the Amalgam token.
  pub metadata: DenomMetadata,
}

#[cw_serde]
pub enum ExecuteMsg {
  /// Register a new component token to the Amalgamation.
  AddComponent(Component),

  /// Deposit a native token to the Amalgamation.
  Deposit {},

  /// Withdraw a token from the Amalgamation. There is a withdrawal fee configurable for each component.
  Withdraw {
    asset: Asset,
  },

  /// Receive a cw20 token with payload.
  Receive(cw20::Cw20ReceiveMsg),

  /// Collect balances for a given asset. Callable only by the admin.
  CollectTaxes {
    asset: Asset,
  },

  /// Update the metadata of the Amalgam token.
  UpdateMetadata(UpdateMetadataMsg),

  /// Update the admin of the Amalgam contract.
  UpdateAdmin {
    admin: String,
  },
}

#[cw_serde]
pub enum Cw20ReceivePayload {
  /// Deposit a cw20 token to the Amalgamation.
  Deposit {},
}

#[cw_serde]
pub struct UpdateMetadataMsg {
  pub name: Option<String>,
  pub description: Option<String>,
  pub uri: Option<String>,
  pub uri_hash: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
  #[returns(ComponentsResponse)]
  Components {},
}

#[cw_serde]
pub struct ComponentsResponse {
  pub components: Vec<Component>,
}
