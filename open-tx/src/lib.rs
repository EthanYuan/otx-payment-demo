mod generated;

pub use generated::packed;

#[cfg(test)]
mod test {
    use super::*;
    use ckb_jsonrpc_types::JsonBytes;
    use generated::packed::OpenTransactionBuilder;
    use molecule::prelude::*;

    #[test]
    fn test_serialize() {
        let builder = OpenTransactionBuilder::default();
        let opentx = builder.build();
        let json_rpc_format = JsonBytes::from_bytes(opentx.as_bytes());
        println!("{:?}", opentx);
        println!("{:?}", json_rpc_format);
    }
}
