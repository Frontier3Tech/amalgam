#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{from_json, Addr, Decimal, DepsMut, Env, Fraction, MessageInfo, Response, Uint128};

use crate::contract::get_tftoken;
use crate::{ContractError, ContractResult};
use crate::msg::{Cw20ReceivePayload, ExecuteMsg, UpdateMetadataMsg};
use crate::state::{Asset, Component, COMPONENTS};

use amalgam_utils::tokenfactory::{self, DenomMetadata, DenomUnit, TFToken};

struct ExecuteContext<'a> {
  deps: DepsMut<'a>,
  env: Env,
  info: MessageInfo,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: ExecuteMsg,
) -> ContractResult<Response> {
  let mut ctx = ExecuteContext { deps, env, info: info.clone() };

  match msg {
    ExecuteMsg::AddComponent(component) =>
      add_component(&mut ctx, component),
    ExecuteMsg::UpdateMetadata(metadata) =>
      update_metadata(&mut ctx, metadata),
    ExecuteMsg::Receive(msg) => {
      let payload: Cw20ReceivePayload = from_json(&msg.msg)?;
      match payload {
        Cw20ReceivePayload::Deposit {} =>
          deposit_cw20(
            &mut ctx,
            info.sender.clone(),
            msg.amount,
            msg.sender.to_string(),
          )
      }
    }
    ExecuteMsg::Deposit {} =>
      deposit_native(&mut ctx),
    ExecuteMsg::Withdraw { asset } =>
      withdraw(&mut ctx, asset),
  }
}

fn add_component(ctx: &mut ExecuteContext, component: Component) -> ContractResult<Response> {
  let key = match component.token.clone() {
    Asset::Native(denom) => {
      let key = format!("native:{}", denom);
      let existing = COMPONENTS.may_load(ctx.deps.storage, key.clone())?;
      if existing.is_some() {
        return Err(ContractError::DuplicateComponent);
      }
      key
    }
    Asset::Cw20(contract) => {
      let key = format!("cw20:{}", contract);
      let existing = COMPONENTS.may_load(ctx.deps.storage, key.clone())?;
      if existing.is_some() {
        return Err(ContractError::DuplicateComponent);
      }
      key
    }
  };

  if component.withdrawal_fee > 10000 {
    return Err(ContractError::InvalidWithdrawalFee);
  }

  COMPONENTS.save(ctx.deps.storage, key, &component)?;

  Ok(Response::new()
    .add_attribute("action", "add_component")
  )
}

fn update_metadata(ctx: &mut ExecuteContext, metadata: UpdateMetadataMsg) -> ContractResult<Response> {
  let tftoken = tokenfactory::osmosis::TFToken::new(
    ctx.env.contract.address.clone(),
    "amalgam".to_string(),
  );

  let existing = ctx.deps.querier.query_denom_metadata(tftoken.denom())?;

  let new_metadata = DenomMetadata {
    base: existing.base,
    denom_units: existing.denom_units.iter().map(|unit| DenomUnit {
      denom: unit.denom.clone(),
      exponent: unit.exponent,
      aliases: unit.aliases.clone(),
    }).collect(),
    display: existing.display,
    name: metadata.name.unwrap_or(existing.name),
    description: metadata.description.unwrap_or(existing.description),
    symbol: existing.symbol,
    uri: metadata.uri.unwrap_or(existing.uri),
    uri_hash: metadata.uri_hash.unwrap_or(existing.uri_hash),
  };

  Ok(Response::new()
    .add_attribute("action", "update_metadata")
    .add_messages(tftoken.set_metadata(new_metadata))
  )
}

fn deposit_native(ctx: &mut ExecuteContext) -> ContractResult<Response> {
  if ctx.info.funds.len() != 1 {
    return Err(ContractError::InvalidFunds);
  }
  let fund = &ctx.info.funds[0];
  let component = COMPONENTS.may_load(ctx.deps.storage, format!("native:{}", fund.denom))?;
  if component.is_none() {
    return Err(ContractError::UnknownAsset);
  }
  deposit(ctx, component.unwrap(), fund.amount, ctx.info.sender.to_string())
}

fn deposit_cw20(ctx: &mut ExecuteContext, token_contract: Addr, amount: Uint128, recipient: String) -> ContractResult<Response> {
  let component = COMPONENTS.may_load(ctx.deps.storage, format!("cw20:{}", token_contract))?;
  if component.is_none() {
    return Err(ContractError::UnknownAsset);
  }
  deposit(ctx, component.unwrap(), amount, recipient)
}

fn deposit(ctx: &mut ExecuteContext, component: Component, amount: Uint128, recipient: String) -> ContractResult<Response> {
  let tftoken = get_tftoken(&ctx.env);
  Ok(Response::new()
    .add_attribute("action", "deposit")
    .add_messages(tftoken.mint(
      amount * component.weight,
      recipient,
    ))
  )
}

fn withdraw(ctx: &mut ExecuteContext, asset: Asset) -> ContractResult<Response> {
  let tftoken = get_tftoken(&ctx.env);

  if ctx.info.funds.len() != 1 {
    return Err(ContractError::InvalidFunds);
  }

  let fund = &ctx.info.funds[0];
  if fund.denom != tftoken.denom() {
    return Err(ContractError::InvalidFunds);
  }

  let component = COMPONENTS.may_load(ctx.deps.storage, asset.key())?;
  if component.is_none() {
    return Err(ContractError::UnknownAsset);
  }
  let component = component.unwrap();

  let withdrawal_fee_decimal = Decimal::from_ratio(component.withdrawal_fee as u64, 10000u64);

  // invert mint weight
  let amount = fund.amount * Decimal::inv(&component.weight).unwrap();
  // apply withdrawal fee
  let amount = amount * (Decimal::one() - withdrawal_fee_decimal);
  // subtract one to make sure we don't run out of funds due to precision loss
  let amount = amount - Uint128::one();

  Ok(Response::new()
    .add_attribute("action", "withdraw")
    // burn the sent tokens
    .add_messages(tftoken.burn(fund.amount, ctx.env.contract.address.to_string()))
    .add_message(asset.send(amount, ctx.info.sender.to_string()))
  )
}
