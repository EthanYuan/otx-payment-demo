use super::IntegrationTest;
use crate::const_definition::MERCURY_URI;
use crate::const_definition::{UDT_1_HASH, UDT_1_HOLDER_ACP_ADDRESS, UDT_1_HOLDER_ACP_ADDRESS_PK};
use crate::utils::instruction::mercury::issue_udt_1;
use crate::utils::mercury_client_rpc::MercuryRpcClient;

use core_rpc_types::{GetBalancePayload, JsonItem};

use std::collections::HashSet;

inventory::submit!(IntegrationTest {
    name: "test_issue_udt",
    test_fn: test_issue_udt
});
fn test_issue_udt() {
    // prepare udt
    issue_udt_1().unwrap();
    let _udt_hash = UDT_1_HASH.get().unwrap();
    let acp_address_with_udt = UDT_1_HOLDER_ACP_ADDRESS.get().unwrap();
    let _acp_address_pk = UDT_1_HOLDER_ACP_ADDRESS_PK.get().unwrap();

    let payload = GetBalancePayload {
        item: JsonItem::Address(acp_address_with_udt.to_string()),
        asset_infos: HashSet::new(),
        extra: None,
        tip_block_number: None,
    };
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    let response = mercury_client.get_balance(payload).unwrap();
    assert_eq!(response.balances.len(), 2);
    assert_eq!(0u128, response.balances[0].free.into());
    assert_eq!(146_0000_0000u128, response.balances[0].occupied.into());
    assert_eq!(0u128, response.balances[0].frozen.into());
    assert_eq!(200_0000_0000u128, response.balances[1].free.into());
}
