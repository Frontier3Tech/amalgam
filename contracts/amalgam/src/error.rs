use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("Unauthorized")]
  Unauthorized,

  #[error("Invalid funds")]
  InvalidFunds,

  #[error("Duplicate component")]
  DuplicateComponent,

  #[error("Unknown asset")]
  UnknownAsset,

  #[error("Invalid fee must be between 0 and 10000")]
  InvalidWithdrawalFee,

  #[error("No taxes")]
  NoTaxes,

  // Add any other custom errors you like here.
  // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
