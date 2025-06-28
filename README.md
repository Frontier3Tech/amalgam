# Amalgam Tokens
An amalgamation is *the action or process of uniting or merging two or more things* ([Merriam-Webster](https://www.merriam-webster.com/dictionary/amalgamation)).

*Amalgam Tokens* are similar to [Osmosis' Alloy Tokens](https://medium.com/osmosis/alloyed-assets-on-osmosis-unifying-ux-and-solving-liquidity-fragmentation-168831ce8862). *Amalgam* tokens are tokens created from other tokens. Unlike *Alloy* tokens, *Amalgam* converts many different tokens into one *Amalgam* token. These are not considered to be equivalent, but converted linearly by an admin-configurable factor, or potentially by other functions in the future.

*Amalgam* was born out of a need to use liquidity tokens as governance tokens. However, DAODAO only supports a single staking token (without a customized membership contract). *Amalgam* allows fusing different LP tokens into one unified, canonical token that can be used for governance.

*Amalgam* tokens can also be converted back into their original tokens. The admin may configure a withdrawal tax, and can subsequently withdraw collected taxes. You don't have to convert back to the same token you originally deposited.

The admin CANNOT:

- Remove a component token that was added
- Change the conversion rate of a component
- Change essential token metadata like symbol or denom units

## Queries

The contract supports the following query messages:

### `Components`
- **Public**: Get all registered component tokens
- **Returns**: `ComponentsResponse` containing a list of all components with their weights and withdrawal taxes

**Example:**

```json
{
  "components": {}
}
```

## Public Execute Messages

The contract supports the following public execute messages:

### `Deposit`
- **Public**: Deposit native tokens to the Amalgamation
- **Usage**: Must be called with native tokens in the transaction funds
- **Result**: Mints Amalgam tokens based on the component's weight

**Example:**

```json
{
  "deposit": {}
}
```

### `Withdraw`
- **Public**: Withdraw tokens from the Amalgamation
- **Parameters**:
  - `asset`: The asset to withdraw (Native denom or CW20 contract address)
- **Result**: Burns Amalgam tokens and returns the specified asset (minus withdrawal tax)

**Example:**

```json
{
  "withdraw": {
    "asset": {
      "native": "uluna"
    }
  }
}
```

### `Receive`
- **Public**: Handle incoming CW20 token transfers
- **Usage**: Called automatically when CW20 tokens are sent to the contract using its `Send` message
- **Payload**: Expects `Cw20ReceivePayload::Deposit {}`

**Example:**

```json
{
  "receive": {
    "sender": "cosmos1...",
    "amount": "1000000",
    "msg": "eyJkZXBvc2l0Ijp7fX0="
  }
}
```

Note that currently the only supported submessage is `{"deposit":{}}` (as in the example).

## Admin Execute Messages

The contract supports the following admin-only execute messages:

### `AddComponent`
- **Admin only**: Register a new component token to the Amalgamation
- **Parameters**:
  - `token`: The asset to add (Native denom or CW20 contract address)
  - `weight`: Conversion rate as a Decimal
  - `withdrawal_tax`: Tax rate in basis points (e.g., 100 = 1%)

**Example:**

```json
{
  "add_component": {
    "token": {
      "native": "uluna"
    },
    "weight": "0.5",
    "withdrawal_tax": "500"
  }
}
```

### `CollectTaxes`
- **Admin only**: Collect accumulated withdrawal taxes for a specific asset
- **Parameters**:
  - `asset`: The asset whose taxes to collect
- **Result**: Transfers collected taxes to the admin

**Example:**

```json
{
  "collect_taxes": {
    "asset": {
      "cw20": "cosmos1..."
    }
  }
}
```

### `UpdateMetadata`
- **Admin only**: Update the metadata of the Amalgam token
- **Parameters**:
  - `name`: Optional new token name
  - `description`: Optional new description
  - `uri`: Optional new URI
  - `uri_hash`: Optional new URI hash

**Example:**

```json
{
  "update_metadata": {
    "name": "Updated Amalgam Token",
    "description": "Updated description",
    "uri": "https://example.com/metadata.json",
    "uri_hash": "sha256hash"
  }
}
```

### `UpdateAdmin`
- **Admin only**: Transfer admin privileges to a new address
- **Parameters**:
  - `admin`: New admin address

**Example:**

```json
{
  "update_admin": {
    "admin": "cosmos1..."
  }
}
```
