use crate::authority::Authority;
use crate::codec::Encoder;
use crate::crypto::PublicKey;

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
        }
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
}
