use serde_json::{Map, Value};

use crate::asset::Asset;
use crate::authority::Authority;
use crate::codec::Encoder;
use crate::crypto::PublicKey;
use crate::error::{Error, Result};

/// Chain-wide witness parameters used by `witness_update`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainProperties {
    pub account_creation_fee: Asset,
    pub maximum_block_size: u32,
    pub sbd_interest_rate: u16,
}

/// A Steem operation. The enum covers the operations a wallet commonly signs.
///
/// New operation variants should add an `op_id`, a JSON payload and a `serialize`
/// arm; the wire order must match the Steem serializer exactly.
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Vote {
        voter: String,
        author: String,
        permlink: String,
        weight: i16,
    },
    Comment {
        parent_author: String,
        parent_permlink: String,
        author: String,
        permlink: String,
        title: String,
        body: String,
        json_metadata: String,
    },
    Transfer {
        from: String,
        to: String,
        amount: crate::asset::Asset,
        memo: String,
    },
    TransferToVesting {
        from: String,
        to: String,
        amount: crate::asset::Asset,
    },
    WithdrawVesting {
        account: String,
        vesting_shares: crate::asset::Asset,
    },
    AccountUpdate {
        account: String,
        owner: Option<Authority>,
        active: Option<Authority>,
        posting: Option<Authority>,
        memo_key: PublicKey,
        json_metadata: String,
    },
    AccountWitnessVote {
        account: String,
        witness: String,
        approve: bool,
    },
    CustomJson {
        required_auths: Vec<String>,
        required_posting_auths: Vec<String>,
        id: String,
        json: String,
    },
    ClaimRewardBalance {
        account: String,
        reward_steem: crate::asset::Asset,
        reward_sbd: crate::asset::Asset,
        reward_vests: crate::asset::Asset,
    },
    TransferToSavings {
        from: String,
        to: String,
        amount: crate::asset::Asset,
        memo: String,
    },
    TransferFromSavings {
        from: String,
        request_id: u32,
        to: String,
        amount: crate::asset::Asset,
        memo: String,
    },
    DelegateVestingShares {
        delegator: String,
        delegatee: String,
        vesting_shares: crate::asset::Asset,
    },
    RecurrentTransfer {
        from: String,
        to: String,
        amount: crate::asset::Asset,
        memo: String,
        recurrence: u16,
        executions: u16,
    },
    SetWithdrawVestingRoute {
        from_account: String,
        to_account: String,
        percent: u16,
        auto_vest: bool,
    },
    WitnessUpdate {
        owner: String,
        url: String,
        block_signing_key: PublicKey,
        props: ChainProperties,
        fee: Asset,
    },
    UpdateProposalVotes {
        voter: String,
        proposal_ids: Vec<i64>,
        approve: bool,
    },
    ChangeRecoveryAccount {
        account_to_recover: String,
        new_recovery_account: String,
    },
    RequestAccountRecovery {
        recovery_account: String,
        account_to_recover: String,
        new_owner_authority: Authority,
    },
    CancelTransferFromSavings {
        from: String,
        request_id: u32,
    },
    AccountWitnessProxy {
        account: String,
        proxy: String,
    },
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match self {
            Operation::Vote { .. } => "vote",
            Operation::Comment { .. } => "comment",
            Operation::Transfer { .. } => "transfer",
            Operation::TransferToVesting { .. } => "transfer_to_vesting",
            Operation::WithdrawVesting { .. } => "withdraw_vesting",
            Operation::AccountUpdate { .. } => "account_update",
            Operation::AccountWitnessVote { .. } => "account_witness_vote",
            Operation::CustomJson { .. } => "custom_json",
            Operation::ClaimRewardBalance { .. } => "claim_reward_balance",
            Operation::TransferToSavings { .. } => "transfer_to_savings",
            Operation::TransferFromSavings { .. } => "transfer_from_savings",
            Operation::DelegateVestingShares { .. } => "delegate_vesting_shares",
            Operation::RecurrentTransfer { .. } => "recurrent_transfer",
            Operation::SetWithdrawVestingRoute { .. } => "set_withdraw_vesting_route",
            Operation::WitnessUpdate { .. } => "witness_update",
            Operation::UpdateProposalVotes { .. } => "update_proposal_votes",
            Operation::ChangeRecoveryAccount { .. } => "change_recovery_account",
            Operation::RequestAccountRecovery { .. } => "request_account_recovery",
            Operation::CancelTransferFromSavings { .. } => "cancel_transfer_from_savings",
            Operation::AccountWitnessProxy { .. } => "account_witness_proxy",
        }
    }

    pub fn op_id(&self) -> u32 {
        match self {
            Operation::Vote { .. } => 0,
            Operation::Comment { .. } => 1,
            Operation::Transfer { .. } => 2,
            Operation::TransferToVesting { .. } => 3,
            Operation::WithdrawVesting { .. } => 4,
            Operation::AccountUpdate { .. } => 10,
            Operation::AccountWitnessVote { .. } => 12,
            Operation::CustomJson { .. } => 18,
            Operation::TransferToSavings { .. } => 32,
            Operation::TransferFromSavings { .. } => 33,
            Operation::ClaimRewardBalance { .. } => 39,
            Operation::DelegateVestingShares { .. } => 40,
            Operation::RecurrentTransfer { .. } => 49,
            Operation::SetWithdrawVestingRoute { .. } => 20,
            Operation::WitnessUpdate { .. } => 11,
            Operation::UpdateProposalVotes { .. } => 45,
            Operation::ChangeRecoveryAccount { .. } => 26,
            Operation::RequestAccountRecovery { .. } => 24,
            Operation::CancelTransferFromSavings { .. } => 34,
            Operation::AccountWitnessProxy { .. } => 13,
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            Operation::Vote {
                voter,
                author,
                permlink,
                weight,
            } => json!({
                "voter": voter,
                "author": author,
                "permlink": permlink,
                "weight": weight,
            }),
            Operation::Comment {
                parent_author,
                parent_permlink,
                author,
                permlink,
                title,
                body,
                json_metadata,
            } => json!({
                "parent_author": parent_author,
                "parent_permlink": parent_permlink,
                "author": author,
                "permlink": permlink,
                "title": title,
                "body": body,
                "json_metadata": json_metadata,
            }),
            Operation::Transfer {
                from,
                to,
                amount,
                memo,
            } => json!({
                "from": from,
                "to": to,
                "amount": amount,
                "memo": memo,
            }),
            Operation::TransferToVesting { from, to, amount } => json!({
                "from": from,
                "to": to,
                "amount": amount,
            }),
            Operation::WithdrawVesting {
                account,
                vesting_shares,
            } => json!({
                "account": account,
                "vesting_shares": vesting_shares,
            }),
            Operation::AccountUpdate {
                account,
                owner,
                active,
                posting,
                memo_key,
                json_metadata,
            } => json!({
                "account": account,
                "owner": owner,
                "active": active,
                "posting": posting,
                "memo_key": memo_key,
                "json_metadata": json_metadata,
            }),
            Operation::AccountWitnessVote {
                account,
                witness,
                approve,
            } => json!({
                "account": account,
                "witness": witness,
                "approve": approve,
            }),
            Operation::CustomJson {
                required_auths,
                required_posting_auths,
                id,
                json,
            } => json!({
                "required_auths": required_auths,
                "required_posting_auths": required_posting_auths,
                "id": id,
                "json": json,
            }),
            Operation::ClaimRewardBalance {
                account,
                reward_steem,
                reward_sbd,
                reward_vests,
            } => json!({
                "account": account,
                "reward_steem": reward_steem,
                "reward_sbd": reward_sbd,
                "reward_vests": reward_vests,
            }),
            Operation::TransferToSavings {
                from,
                to,
                amount,
                memo,
            } => json!({
                "from": from,
                "to": to,
                "amount": amount,
                "memo": memo,
            }),
            Operation::TransferFromSavings {
                from,
                request_id,
                to,
                amount,
                memo,
            } => json!({
                "from": from,
                "request_id": request_id,
                "to": to,
                "amount": amount,
                "memo": memo,
            }),
            Operation::DelegateVestingShares {
                delegator,
                delegatee,
                vesting_shares,
            } => json!({
                "delegator": delegator,
                "delegatee": delegatee,
                "vesting_shares": vesting_shares,
            }),
            Operation::RecurrentTransfer {
                from,
                to,
                amount,
                memo,
                recurrence,
                executions,
            } => json!({
                "from": from,
                "to": to,
                "amount": amount,
                "memo": memo,
                "recurrence": recurrence,
                "executions": executions,
                "extensions": [],
            }),
            Operation::SetWithdrawVestingRoute {
                from_account,
                to_account,
                percent,
                auto_vest,
            } => json!({
                "from_account": from_account,
                "to_account": to_account,
                "percent": percent,
                "auto_vest": auto_vest,
            }),
            Operation::WitnessUpdate {
                owner,
                url,
                block_signing_key,
                props,
                fee,
            } => json!({
                "owner": owner,
                "url": url,
                "block_signing_key": block_signing_key,
                "props": {
                    "account_creation_fee": props.account_creation_fee,
                    "maximum_block_size": props.maximum_block_size,
                    "sbd_interest_rate": props.sbd_interest_rate,
                },
                "fee": fee,
            }),
            Operation::UpdateProposalVotes {
                voter,
                proposal_ids,
                approve,
            } => json!({
                "voter": voter,
                "proposal_ids": proposal_ids,
                "approve": approve,
                "extensions": [],
            }),
            Operation::ChangeRecoveryAccount {
                account_to_recover,
                new_recovery_account,
            } => json!({
                "account_to_recover": account_to_recover,
                "new_recovery_account": new_recovery_account,
                "extensions": [],
            }),
            Operation::RequestAccountRecovery {
                recovery_account,
                account_to_recover,
                new_owner_authority,
            } => json!({
                "recovery_account": recovery_account,
                "account_to_recover": account_to_recover,
                "new_owner_authority": new_owner_authority,
                "extensions": [],
            }),
            Operation::CancelTransferFromSavings { from, request_id } => json!({
                "from": from,
                "request_id": request_id,
            }),
            Operation::AccountWitnessProxy { account, proxy } => json!({
                "account": account,
                "proxy": proxy,
            }),
        }
    }

    /// `[name, payload]`, matching Steem's JSON operation tuple.
    pub fn to_tuple(&self) -> serde_json::Value {
        serde_json::json!([self.name(), self.to_json_value()])
    }

    pub(crate) fn serialize(&self, encoder: &mut Encoder) {
        encoder.write_varint32(self.op_id());
        match self {
            Operation::Vote {
                voter,
                author,
                permlink,
                weight,
            } => {
                encoder.write_vstring(voter);
                encoder.write_vstring(author);
                encoder.write_vstring(permlink);
                encoder.write_i16_le(*weight);
            }
            Operation::Comment {
                parent_author,
                parent_permlink,
                author,
                permlink,
                title,
                body,
                json_metadata,
            } => {
                encoder.write_vstring(parent_author);
                encoder.write_vstring(parent_permlink);
                encoder.write_vstring(author);
                encoder.write_vstring(permlink);
                encoder.write_vstring(title);
                encoder.write_vstring(body);
                encoder.write_vstring(json_metadata);
            }
            Operation::Transfer {
                from,
                to,
                amount,
                memo,
            } => {
                encoder.write_vstring(from);
                encoder.write_vstring(to);
                write_asset(encoder, *amount);
                encoder.write_vstring(memo);
            }
            Operation::TransferToVesting { from, to, amount } => {
                encoder.write_vstring(from);
                encoder.write_vstring(to);
                write_asset(encoder, *amount);
            }
            Operation::WithdrawVesting {
                account,
                vesting_shares,
            } => {
                encoder.write_vstring(account);
                write_asset(encoder, *vesting_shares);
            }
            Operation::AccountUpdate {
                account,
                owner,
                active,
                posting,
                memo_key,
                json_metadata,
            } => {
                encoder.write_vstring(account);
                write_optional_authority(encoder, owner);
                write_optional_authority(encoder, active);
                write_optional_authority(encoder, posting);
                encoder.write_bytes(memo_key.as_bytes());
                encoder.write_vstring(json_metadata);
            }
            Operation::AccountWitnessVote {
                account,
                witness,
                approve,
            } => {
                encoder.write_vstring(account);
                encoder.write_vstring(witness);
                encoder.write_bool(*approve);
            }
            Operation::CustomJson {
                required_auths,
                required_posting_auths,
                id,
                json,
            } => {
                write_string_array(encoder, required_auths);
                write_string_array(encoder, required_posting_auths);
                encoder.write_vstring(id);
                encoder.write_vstring(json);
            }
            Operation::ClaimRewardBalance {
                account,
                reward_steem,
                reward_sbd,
                reward_vests,
            } => {
                encoder.write_vstring(account);
                write_asset(encoder, *reward_steem);
                write_asset(encoder, *reward_sbd);
                write_asset(encoder, *reward_vests);
            }
            Operation::TransferToSavings {
                from,
                to,
                amount,
                memo,
            } => {
                encoder.write_vstring(from);
                encoder.write_vstring(to);
                write_asset(encoder, *amount);
                encoder.write_vstring(memo);
            }
            Operation::TransferFromSavings {
                from,
                request_id,
                to,
                amount,
                memo,
            } => {
                encoder.write_vstring(from);
                encoder.write_u32_le(*request_id);
                encoder.write_vstring(to);
                write_asset(encoder, *amount);
                encoder.write_vstring(memo);
            }
            Operation::DelegateVestingShares {
                delegator,
                delegatee,
                vesting_shares,
            } => {
                encoder.write_vstring(delegator);
                encoder.write_vstring(delegatee);
                write_asset(encoder, *vesting_shares);
            }
            Operation::RecurrentTransfer {
                from,
                to,
                amount,
                memo,
                recurrence,
                executions,
            } => {
                encoder.write_vstring(from);
                encoder.write_vstring(to);
                write_asset(encoder, *amount);
                encoder.write_vstring(memo);
                encoder.write_u16_le(*recurrence);
                encoder.write_u16_le(*executions);
                encoder.write_varint32(0); // extensions
            }
            Operation::SetWithdrawVestingRoute {
                from_account,
                to_account,
                percent,
                auto_vest,
            } => {
                encoder.write_vstring(from_account);
                encoder.write_vstring(to_account);
                encoder.write_u16_le(*percent);
                encoder.write_bool(*auto_vest);
            }
            Operation::WitnessUpdate {
                owner,
                url,
                block_signing_key,
                props,
                fee,
            } => {
                encoder.write_vstring(owner);
                encoder.write_vstring(url);
                encoder.write_bytes(block_signing_key.as_bytes());
                write_asset(encoder, props.account_creation_fee);
                encoder.write_u32_le(props.maximum_block_size);
                encoder.write_u16_le(props.sbd_interest_rate);
                write_asset(encoder, *fee);
            }
            Operation::UpdateProposalVotes {
                voter,
                proposal_ids,
                approve,
            } => {
                encoder.write_vstring(voter);
                encoder.write_varint32(proposal_ids.len() as u32);
                for proposal_id in proposal_ids {
                    encoder.write_i64_le(*proposal_id);
                }
                encoder.write_bool(*approve);
                encoder.write_varint32(0); // extensions
            }
            Operation::ChangeRecoveryAccount {
                account_to_recover,
                new_recovery_account,
            } => {
                encoder.write_vstring(account_to_recover);
                encoder.write_vstring(new_recovery_account);
                encoder.write_varint32(0); // extensions
            }
            Operation::RequestAccountRecovery {
                recovery_account,
                account_to_recover,
                new_owner_authority,
            } => {
                encoder.write_vstring(recovery_account);
                encoder.write_vstring(account_to_recover);
                new_owner_authority.serialize(encoder);
                encoder.write_varint32(0); // extensions
            }
            Operation::CancelTransferFromSavings { from, request_id } => {
                encoder.write_vstring(from);
                encoder.write_u32_le(*request_id);
            }
            Operation::AccountWitnessProxy { account, proxy } => {
                encoder.write_vstring(account);
                encoder.write_vstring(proxy);
            }
        }
    }
}

impl Operation {
    /// Parse a Steem JSON operation tuple: `["transfer", { ... }]`.
    pub fn from_tuple(value: &Value) -> Result<Self> {
        let array = value
            .as_array()
            .ok_or_else(|| Error::Serialization("operation must be a JSON array".to_string()))?;
        if array.len() != 2 {
            return Err(Error::Serialization(
                "operation must have exactly two elements".to_string(),
            ));
        }
        let name = array[0]
            .as_str()
            .ok_or_else(|| Error::Serialization("operation name must be a string".to_string()))?;
        let payload = array[1].as_object().ok_or_else(|| {
            Error::Serialization("operation payload must be an object".to_string())
        })?;

        match name {
            "vote" => Ok(Operation::Vote {
                voter: req_str(payload, "voter")?,
                author: req_str(payload, "author")?,
                permlink: req_str(payload, "permlink")?,
                weight: req_i16(payload, "weight")?,
            }),
            "comment" => Ok(Operation::Comment {
                parent_author: req_str(payload, "parent_author")?,
                parent_permlink: req_str(payload, "parent_permlink")?,
                author: req_str(payload, "author")?,
                permlink: req_str(payload, "permlink")?,
                title: req_str(payload, "title")?,
                body: req_str(payload, "body")?,
                json_metadata: opt_str(payload, "json_metadata"),
            }),
            "transfer" => Ok(Operation::Transfer {
                from: req_str(payload, "from")?,
                to: req_str(payload, "to")?,
                amount: req_asset(payload, "amount")?,
                memo: opt_str(payload, "memo"),
            }),
            "transfer_to_vesting" => Ok(Operation::TransferToVesting {
                from: req_str(payload, "from")?,
                to: req_str(payload, "to")?,
                amount: req_asset(payload, "amount")?,
            }),
            "withdraw_vesting" => Ok(Operation::WithdrawVesting {
                account: req_str(payload, "account")?,
                vesting_shares: req_asset(payload, "vesting_shares")?,
            }),
            "account_update" => Ok(Operation::AccountUpdate {
                account: req_str(payload, "account")?,
                owner: opt_authority(payload, "owner")?,
                active: opt_authority(payload, "active")?,
                posting: opt_authority(payload, "posting")?,
                memo_key: req_public_key(payload, "memo_key")?,
                json_metadata: opt_str(payload, "json_metadata"),
            }),
            "account_witness_vote" => Ok(Operation::AccountWitnessVote {
                account: req_str(payload, "account")?,
                witness: req_str(payload, "witness")?,
                approve: req_bool(payload, "approve")?,
            }),
            "custom_json" => Ok(Operation::CustomJson {
                required_auths: opt_string_array(payload, "required_auths")?,
                required_posting_auths: opt_string_array(payload, "required_posting_auths")?,
                id: req_str(payload, "id")?,
                json: req_str(payload, "json")?,
            }),
            "claim_reward_balance" => Ok(Operation::ClaimRewardBalance {
                account: req_str(payload, "account")?,
                reward_steem: req_asset(payload, "reward_steem")?,
                reward_sbd: req_asset(payload, "reward_sbd")?,
                reward_vests: req_asset(payload, "reward_vests")?,
            }),
            "transfer_to_savings" => Ok(Operation::TransferToSavings {
                from: req_str(payload, "from")?,
                to: req_str(payload, "to")?,
                amount: req_asset(payload, "amount")?,
                memo: opt_str(payload, "memo"),
            }),
            "transfer_from_savings" => Ok(Operation::TransferFromSavings {
                from: req_str(payload, "from")?,
                request_id: req_u32(payload, "request_id")?,
                to: req_str(payload, "to")?,
                amount: req_asset(payload, "amount")?,
                memo: opt_str(payload, "memo"),
            }),
            "delegate_vesting_shares" => Ok(Operation::DelegateVestingShares {
                delegator: req_str(payload, "delegator")?,
                delegatee: req_str(payload, "delegatee")?,
                vesting_shares: req_asset(payload, "vesting_shares")?,
            }),
            "recurrent_transfer" => Ok(Operation::RecurrentTransfer {
                from: req_str(payload, "from")?,
                to: req_str(payload, "to")?,
                amount: req_asset(payload, "amount")?,
                memo: opt_str(payload, "memo"),
                recurrence: req_u16(payload, "recurrence")?,
                executions: req_u16(payload, "executions")?,
            }),
            "set_withdraw_vesting_route" => Ok(Operation::SetWithdrawVestingRoute {
                from_account: req_str(payload, "from_account")?,
                to_account: req_str(payload, "to_account")?,
                percent: req_u16(payload, "percent")?,
                auto_vest: req_bool(payload, "auto_vest")?,
            }),
            "witness_update" => {
                let props = field(payload, "props")?.as_object().ok_or_else(|| {
                    Error::Serialization("witness props must be an object".to_string())
                })?;
                Ok(Operation::WitnessUpdate {
                    owner: req_str(payload, "owner")?,
                    url: req_str(payload, "url")?,
                    block_signing_key: req_public_key(payload, "block_signing_key")?,
                    props: ChainProperties {
                        account_creation_fee: req_asset(props, "account_creation_fee")?,
                        maximum_block_size: req_u32(props, "maximum_block_size")?,
                        sbd_interest_rate: req_u16(props, "sbd_interest_rate")?,
                    },
                    fee: req_asset(payload, "fee")?,
                })
            }
            "update_proposal_votes" => Ok(Operation::UpdateProposalVotes {
                voter: req_str(payload, "voter")?,
                proposal_ids: req_i64_array(payload, "proposal_ids")?,
                approve: req_bool(payload, "approve")?,
            }),
            "change_recovery_account" => Ok(Operation::ChangeRecoveryAccount {
                account_to_recover: req_str(payload, "account_to_recover")?,
                new_recovery_account: req_str(payload, "new_recovery_account")?,
            }),
            "request_account_recovery" => {
                let recovery_account = payload
                    .get("recovery_account")
                    .or_else(|| payload.get("recovery_authority"))
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        Error::Serialization(
                            "request_account_recovery is missing recovery_account".to_string(),
                        )
                    })?
                    .to_string();
                Ok(Operation::RequestAccountRecovery {
                    recovery_account,
                    account_to_recover: req_str(payload, "account_to_recover")?,
                    new_owner_authority: req_authority(payload, "new_owner_authority")?,
                })
            }
            "cancel_transfer_from_savings" => Ok(Operation::CancelTransferFromSavings {
                from: req_str(payload, "from")?,
                request_id: req_u32(payload, "request_id")?,
            }),
            "account_witness_proxy" => Ok(Operation::AccountWitnessProxy {
                account: req_str(payload, "account")?,
                proxy: req_str(payload, "proxy")?,
            }),
            other => Err(Error::UnsupportedOperation(other.to_string())),
        }
    }
}

fn field<'a>(payload: &'a Map<String, Value>, key: &str) -> Result<&'a Value> {
    payload
        .get(key)
        .ok_or_else(|| Error::Serialization(format!("operation field {key:?} is missing")))
}

fn req_str(payload: &Map<String, Value>, key: &str) -> Result<String> {
    field(payload, key)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| Error::Serialization(format!("operation field {key:?} must be a string")))
}

fn opt_str(payload: &Map<String, Value>, key: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn req_asset(payload: &Map<String, Value>, key: &str) -> Result<Asset> {
    req_str(payload, key)?.parse()
}

fn req_u16(payload: &Map<String, Value>, key: &str) -> Result<u16> {
    let value = field(payload, key)?.as_u64().ok_or_else(|| {
        Error::Serialization(format!("operation field {key:?} must be an integer"))
    })?;
    u16::try_from(value)
        .map_err(|_| Error::Serialization(format!("operation field {key:?} is out of range")))
}

fn req_u32(payload: &Map<String, Value>, key: &str) -> Result<u32> {
    let value = field(payload, key)?.as_u64().ok_or_else(|| {
        Error::Serialization(format!("operation field {key:?} must be an integer"))
    })?;
    u32::try_from(value)
        .map_err(|_| Error::Serialization(format!("operation field {key:?} is out of range")))
}

fn req_i16(payload: &Map<String, Value>, key: &str) -> Result<i16> {
    let value = field(payload, key)?.as_i64().ok_or_else(|| {
        Error::Serialization(format!("operation field {key:?} must be an integer"))
    })?;
    i16::try_from(value)
        .map_err(|_| Error::Serialization(format!("operation field {key:?} is out of range")))
}

fn req_bool(payload: &Map<String, Value>, key: &str) -> Result<bool> {
    field(payload, key)?
        .as_bool()
        .ok_or_else(|| Error::Serialization(format!("operation field {key:?} must be a boolean")))
}

fn req_string_array(payload: &Map<String, Value>, key: &str) -> Result<Vec<String>> {
    let array = field(payload, key)?
        .as_array()
        .ok_or_else(|| Error::Serialization(format!("operation field {key:?} must be an array")))?;
    array
        .iter()
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                Error::Serialization(format!("operation field {key:?} must contain strings"))
            })
        })
        .collect()
}

fn opt_string_array(payload: &Map<String, Value>, key: &str) -> Result<Vec<String>> {
    match payload.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(_) => req_string_array(payload, key),
    }
}

fn req_i64_array(payload: &Map<String, Value>, key: &str) -> Result<Vec<i64>> {
    let array = field(payload, key)?
        .as_array()
        .ok_or_else(|| Error::Serialization(format!("operation field {key:?} must be an array")))?;
    array
        .iter()
        .map(|value| {
            value.as_i64().ok_or_else(|| {
                Error::Serialization(format!("operation field {key:?} must contain integers"))
            })
        })
        .collect()
}

fn req_authority(payload: &Map<String, Value>, key: &str) -> Result<Authority> {
    serde_json::from_value(field(payload, key)?.clone())
        .map_err(|error| Error::Serialization(format!("invalid authority {key:?}: {error}")))
}

fn opt_authority(payload: &Map<String, Value>, key: &str) -> Result<Option<Authority>> {
    match payload.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => Ok(Some(serde_json::from_value(value.clone()).map_err(
            |error| Error::Serialization(format!("invalid authority {key:?}: {error}")),
        )?)),
    }
}

fn req_public_key(payload: &Map<String, Value>, key: &str) -> Result<PublicKey> {
    let value = field(payload, key)?;
    if let Some(text) = value.as_str() {
        text.parse()
    } else {
        serde_json::from_value(value.clone())
            .map_err(|error| Error::Serialization(format!("invalid public key {key:?}: {error}")))
    }
}

fn write_asset(encoder: &mut Encoder, asset: crate::asset::Asset) {
    encoder.write_i64_le(asset.amount);
    encoder.write_u8(asset.precision());
    let mut symbol = [0u8; 7];
    let bytes = asset.symbol().as_str().as_bytes();
    let length = bytes.len().min(symbol.len());
    symbol[..length].copy_from_slice(&bytes[..length]);
    encoder.write_bytes(&symbol);
}

fn write_string_array(encoder: &mut Encoder, items: &[String]) {
    encoder.write_varint32(items.len() as u32);
    for item in items {
        encoder.write_vstring(item);
    }
}

fn write_optional_authority(encoder: &mut Encoder, authority: &Option<Authority>) {
    match authority {
        Some(authority) => {
            encoder.write_u8(1);
            authority.serialize(encoder);
        }
        None => encoder.write_u8(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::Asset;

    #[test]
    fn transfer_payload_is_a_tuple() {
        let op = Operation::Transfer {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: "1.234 STEEM".parse::<Asset>().unwrap(),
            memo: "hello".to_string(),
        };
        assert_eq!(op.name(), "transfer");
        assert_eq!(op.op_id(), 2);
        let tuple = op.to_tuple();
        assert_eq!(tuple[0], "transfer");
        assert_eq!(tuple[1]["amount"], "1.234 STEEM");
    }

    #[test]
    fn parses_wallet_operation_tuples() {
        use serde_json::json;
        const KEY: &str = "STM4zupkLHXpMLBmQEWc2sPBuGcdm2dJb4GNNvaDkVbr1Zo3GNFmC";
        let cases = vec![
            json!(["vote", { "voter": "alice", "author": "bob", "permlink": "post", "weight": 10000 }]),
            json!(["comment", { "parent_author": "", "parent_permlink": "tag", "author": "alice", "permlink": "post", "title": "T", "body": "B", "json_metadata": "{}" }]),
            json!(["transfer", { "from": "alice", "to": "bob", "amount": "1.000 STEEM", "memo": "hi" }]),
            json!(["transfer_to_vesting", { "from": "alice", "to": "alice", "amount": "1.000 STEEM" }]),
            json!(["withdraw_vesting", { "account": "alice", "vesting_shares": "1.000000 VESTS" }]),
            json!(["delegate_vesting_shares", { "delegator": "alice", "delegatee": "bob", "vesting_shares": "1.000000 VESTS" }]),
            json!(["account_witness_vote", { "account": "alice", "witness": "bob", "approve": true }]),
            json!(["account_witness_proxy", { "account": "alice", "proxy": "bob" }]),
            json!(["claim_reward_balance", { "account": "alice", "reward_steem": "0.000 STEEM", "reward_sbd": "0.000 SBD", "reward_vests": "0.000000 VESTS" }]),
            json!(["transfer_to_savings", { "from": "alice", "to": "bob", "amount": "1.000 STEEM", "memo": "" }]),
            json!(["transfer_from_savings", { "from": "alice", "request_id": 1, "to": "bob", "amount": "1.000 STEEM", "memo": "" }]),
            json!(["cancel_transfer_from_savings", { "from": "alice", "request_id": 1 }]),
            json!(["set_withdraw_vesting_route", { "from_account": "alice", "to_account": "bob", "percent": 5000, "auto_vest": true }]),
            json!(["update_proposal_votes", { "voter": "alice", "proposal_ids": [1, 2], "approve": true, "extensions": [] }]),
            json!(["change_recovery_account", { "account_to_recover": "alice", "new_recovery_account": "bob", "extensions": [] }]),
            json!(["request_account_recovery", { "recovery_account": "bob", "account_to_recover": "alice", "new_owner_authority": { "weight_threshold": 1, "key_auths": [[KEY, 1]], "account_auths": [] }, "extensions": [] }]),
            json!(["custom_json", { "required_auths": [], "required_posting_auths": ["alice"], "id": "test", "json": "{}" }]),
            json!(["witness_update", { "owner": "alice", "url": "https://example.com", "block_signing_key": KEY, "props": { "account_creation_fee": "1.000 STEEM", "maximum_block_size": 65536, "sbd_interest_rate": 1000 }, "fee": "0.000 STEEM" }]),
            json!(["account_update", { "account": "alice", "owner": null, "active": null, "posting": null, "memo_key": KEY, "json_metadata": "" }]),
            json!(["recurrent_transfer", { "from": "alice", "to": "bob", "amount": "1.000 STEEM", "memo": "", "recurrence": 24, "executions": 2, "extensions": [] }]),
        ];
        for case in cases {
            let operation = Operation::from_tuple(&case).expect("parse operation");
            let tuple = operation.to_tuple();
            assert_eq!(tuple[0], case[0], "operation name round-trip");
        }
    }
}
