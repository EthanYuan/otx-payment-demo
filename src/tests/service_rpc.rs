use super::super::IntegrationTest;

use utils::client::service_client::ServiceRpcClient;
use utils::const_definition::SERVICE_URI;

use ckb_jsonrpc_types::JsonBytes;

inventory::submit!(IntegrationTest {
    name: "service_rpc_test",
    test_fn: service_rpc_test
});
fn service_rpc_test() {
    let service_client = ServiceRpcClient::new(SERVICE_URI.to_string());
    let ret = service_client.submit_otx(JsonBytes::default());
    println!("{:?}", ret)
}
