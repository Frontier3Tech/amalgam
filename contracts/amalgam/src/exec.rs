#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{from_json, Addr, Decimal, DepsMut, Env, Fraction, MessageInfo, Response, Uint128};

use crate::contract::get_tftoken;
use crate::{ContractError, ContractResult};
use crate::msg::{Cw20ReceivePayload, ExecuteMsg, UpdateMetadataMsg};
use crate::state::{Asset, Component, COMPONENTS, STATE, WITHDRAWAL_TAXES};

use amalgam_utils::tokenfactory::{DenomMetadata, DenomUnit, TFToken};

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
    ExecuteMsg::CollectTaxes { asset } =>
      collect_taxes(&mut ctx, asset),
    ExecuteMsg::UpdateAdmin { admin } =>
      update_admin(&mut ctx, admin),
  }
}

fn add_component(ctx: &mut ExecuteContext, component: Component) -> ContractResult<Response> {
  helpers::assert_admin(ctx)?;

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

  if component.withdrawal_tax > 10000 {
    return Err(ContractError::InvalidWithdrawalFee);
  }

  COMPONENTS.save(ctx.deps.storage, key, &component)?;

  Ok(Response::new()
    .add_attribute("action", "add_component")
  )
}

fn update_metadata(ctx: &mut ExecuteContext, metadata: UpdateMetadataMsg) -> ContractResult<Response> {
  helpers::assert_admin(ctx)?;

  let tftoken = get_tftoken(&ctx.env);
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

  let withdrawal_tax_decimal = Decimal::from_ratio(component.withdrawal_tax as u64, 10000u64);

  // invert mint weight
  let amount = fund.amount * Decimal::inv(&component.weight).unwrap();
  // compute withdrawal tax
  let tax = amount * withdrawal_tax_decimal;
  // apply withdrawal tax & subtract one to make sure we don't run out of funds due to precision loss
  let amount = amount - tax - Uint128::one();

  // update collected withdrawal taxes
  WITHDRAWAL_TAXES.update(ctx.deps.storage, asset.key(), |fees| -> Result<Uint128, ContractError> {
    Ok(fees.unwrap_or(Uint128::zero()) + tax)
  })?;

  Ok(Response::new()
    .add_attribute("action", "withdraw")
    // burn the sent tokens
    .add_messages(tftoken.burn(fund.amount, ctx.env.contract.address.to_string()))
    .add_message(asset.send(amount, ctx.info.sender.to_string()))
  )
}

fn collect_taxes(ctx: &mut ExecuteContext, asset: Asset) -> ContractResult<Response> {
  let admin = helpers::assert_admin(ctx)?;

  let taxes = WITHDRAWAL_TAXES.may_load(ctx.deps.storage, asset.key())?;
  if taxes.is_none() {
    return Err(ContractError::NoTaxes);
  }
  let taxes = taxes.unwrap();

  WITHDRAWAL_TAXES.remove(ctx.deps.storage, asset.key());

  Ok(Response::new()
    .add_attribute("action", "collect_taxes")
    .add_message(asset.send(taxes, admin.to_string()))
  )
}

fn update_admin(ctx: &mut ExecuteContext, admin: String) -> ContractResult<Response> {
  STATE.update(ctx.deps.storage, |mut state| {
    if ctx.info.sender != state.admin {
      return Err(ContractError::Unauthorized);
    }
    state.admin = admin.clone();
    Ok(state)
  })?;
  Ok(Response::new()
    .add_attribute("action", "update_admin")
    .add_attribute("new_admin", admin)
  )
}

mod helpers {
  use super::*;

  pub fn assert_admin(ctx: &mut ExecuteContext) -> ContractResult<Addr> {
    let state = STATE.load(ctx.deps.storage)?;
    if ctx.info.sender != state.admin {
      return Err(ContractError::Unauthorized);
    }
    // sender can never be an invalid address, so `.unwrap()` is safe
    Ok(ctx.deps.api.addr_validate(&state.admin).unwrap())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

  use crate::state::State;

  #[test]
  fn test_add_component_non_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user", &[]);

    let msg = ExecuteMsg::AddComponent(Component {
      token: Asset::Native("uosmo".to_string()),
      weight: Decimal::from_ratio(1u64, 100u64),
      withdrawal_tax: 1000,
    });

    STATE.save(deps.as_mut().storage, &State {
      admin: "admin".to_string(),
    }).unwrap();

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(matches!(res, Err(ContractError::Unauthorized)));

    let info = mock_info("admin", &[]);

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(matches!(res, Ok(_)));

    let component = COMPONENTS.load(deps.as_mut().storage, "native:uosmo".to_string()).unwrap();
    assert_eq!(component.weight, Decimal::from_ratio(1u64, 100u64));
    assert_eq!(component.withdrawal_tax, 1000);
  }

  #[test]
  fn test_change_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user", &[]);

    let msg = ExecuteMsg::UpdateAdmin {
      admin: "new_admin".to_string(),
    };

    STATE.save(deps.as_mut().storage, &State {
      admin: "admin".to_string(),
    }).unwrap();

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(matches!(res, Err(ContractError::Unauthorized)));

    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(matches!(res, Ok(_)));

    let state = STATE.load(deps.as_mut().storage).unwrap();
    assert_eq!(state.admin, "new_admin".to_string());
  }

  #[test]
  fn test_update_metadata() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user", &[]);

    let msg = ExecuteMsg::UpdateMetadata(UpdateMetadataMsg {
      name: Some("new_name".to_string()),
      description: Some("new_description".to_string()),
      uri: Some("new_uri".to_string()),
      uri_hash: Some("new_uri_hash".to_string()),
    });

    STATE.save(deps.as_mut().storage, &State {
      admin: "admin".to_string(),
    }).unwrap();

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(matches!(res, Err(ContractError::Unauthorized)));

    // NOTE: cannot update metadata w/o an actual chain, bank module mock is insufficient
  }

  #[test]
  fn test_collect_taxes() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("user", &[]);

    let msg = ExecuteMsg::CollectTaxes {
      asset: Asset::Native("utest".to_string()),
    };

    STATE.save(deps.as_mut().storage, &State {
      admin: "admin".to_string(),
    }).unwrap();

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(matches!(res, Err(ContractError::Unauthorized)));

    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    assert!(matches!(res, Err(ContractError::NoTaxes)));

    WITHDRAWAL_TAXES.save(deps.as_mut().storage, "native:utest".to_string(), &Uint128::from(1000u64)).unwrap();

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    assert!(matches!(res, Ok(_)));

    let taxes = WITHDRAWAL_TAXES.may_load(deps.as_mut().storage, "native:utest".to_string()).unwrap();
    assert_eq!(taxes, None);
  }
}
