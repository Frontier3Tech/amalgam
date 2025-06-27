use prost::Message;
use cosmwasm_std::{Addr, CosmosMsg, Uint128};

use amalgam_macros::typeurl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub trait TFToken {
  /// Address of the owner of the token, usually the contract address
  fn owner(&self) -> Addr;

  /// Subdenom of the token, e.g. "SouLP"
  fn subdenom(&self) -> String;

  /// Full denom of the token, based on `owner` and `subdenom`
  fn denom(&self) -> String;

  /// Create the token, e.g. register the denom on the chain
  fn create(&self) -> Vec<CosmosMsg>;

  /// Set the metadata of the token
  fn set_metadata(&self, metadata: DenomMetadata) -> Vec<CosmosMsg>;

  /// Mint tokens to a recipient
  fn mint(&self, amount: Uint128, recipient: String) -> Vec<CosmosMsg>;

  /// Burn tokens from a recipient
  fn burn(&self, amount: Uint128, sender: String) -> Vec<CosmosMsg>;
}

#[derive(Clone, PartialEq, Message)]
#[typeurl("/cosmos.bank.v1beta1.Coin")]
pub struct Coin {
  #[prost(string, tag = "1")]
  pub denom: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub amount: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, Message, Serialize, Deserialize, JsonSchema)]
#[typeurl("/cosmos.bank.v1beta1.Metadata")]
pub struct DenomMetadata {
  #[prost(string, tag = "1")]
  pub description: String,
  #[prost(message, repeated, tag = "2")]
  pub denom_units: Vec<DenomUnit>,
  #[prost(string, tag = "3")]
  pub base: String,
  #[prost(string, tag = "4")]
  pub display: String,
  #[prost(string, tag = "5")]
  pub name: String,
  #[prost(string, tag = "6")]
  pub symbol: String,
  #[prost(string, tag = "7")]
  pub uri: String,
  #[prost(string, tag = "8")]
  pub uri_hash: String,
}

#[derive(Clone, PartialEq, Message, Serialize, Deserialize, JsonSchema)]
pub struct DenomUnit {
  #[prost(string, tag = "1")]
  pub denom: String,
  #[prost(uint32, tag = "2")]
  pub exponent: u32,
  #[prost(string, repeated, tag = "3")]
  pub aliases: Vec<String>,
}

pub mod osmosis {
  use super::*;

  #[derive(Clone, PartialEq, Message)]
  #[typeurl("/osmosis.tokenfactory.v1beta1.MsgCreateDenom")]
  pub struct MsgCreateDenom {
    #[prost(string, tag = "1")]
    pub sender: String,
    #[prost(string, tag = "2")]
    pub subdenom: String,
  }

  #[derive(Clone, PartialEq, Message)]
  #[typeurl("/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata")]
  pub struct MsgSetDenomMetadata {
    #[prost(string, required, tag = "1")]
    pub sender: String,
    #[prost(message, required, tag = "2")]
    pub metadata: DenomMetadata,
  }

  #[derive(Clone, PartialEq, Message)]
  #[typeurl("/osmosis.tokenfactory.v1beta1.MsgMint")]
  pub struct MsgMint {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: Option<Coin>,
    #[prost(string, tag = "3")]
    pub mint_to_address: String,
  }

  #[derive(Clone, PartialEq, Message)]
  #[typeurl("/osmosis.tokenfactory.v1beta1.MsgBurn")]
  pub struct MsgBurn {
    #[prost(string, tag = "1")]
    pub sender: String,
    #[prost(message, required, tag = "2")]
    pub amount: Coin,
    #[prost(string, tag = "3")]
    pub burn_from_address: String,
  }

  pub struct TFToken {
    pub owner: Addr,
    pub subdenom: String,
  }

  impl TFToken {
    pub fn new(owner: Addr, subdenom: String) -> Self {
      Self { owner, subdenom }
    }
  }

  impl super::TFToken for TFToken {
    fn owner(&self) -> Addr {
      self.owner.clone()
    }

    fn subdenom(&self) -> String {
      self.subdenom.clone()
    }

    fn denom(&self) -> String {
      format!("factory/{}/{}", self.owner, self.subdenom)
    }

    fn create(&self) -> Vec<CosmosMsg> {
      vec![MsgCreateDenom { sender: self.owner.to_string(), subdenom: self.subdenom.clone() }.into()]
    }

    fn set_metadata(&self, metadata: DenomMetadata) -> Vec<CosmosMsg> {
      vec![MsgSetDenomMetadata { sender: self.owner.to_string(), metadata }.into()]
    }

    fn mint(&self, amount: Uint128, recipient: String) -> Vec<CosmosMsg> {
      vec![MsgMint {
        sender: self.owner.to_string(),
        amount: Some(Coin {
          denom: self.denom(),
          amount: amount.to_string(),
        }),
        mint_to_address: recipient,
      }.into()]
    }

    fn burn(&self, amount: Uint128, sender: String) -> Vec<CosmosMsg> {
      vec![MsgBurn {
        sender: self.owner.to_string(),
        amount: Coin { denom: self.denom(), amount: amount.to_string() },
        burn_from_address: sender,
      }.into()]
    }
  }
}
