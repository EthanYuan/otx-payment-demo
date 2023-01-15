mod otx_service_test;
mod sdk_open_tx_examples;
mod transfer_udt;
mod user_case;

#[derive(Debug)]
pub struct IntegrationTest {
    pub name: &'static str,
    pub test_fn: fn(),
}

impl IntegrationTest {
    pub fn _all_test_names() -> Vec<&'static str> {
        inventory::iter::<IntegrationTest>
            .into_iter()
            .map(|x| x.name)
            .collect::<Vec<&str>>()
    }

    pub fn from_name<S: AsRef<str>>(test_name: S) -> Option<&'static IntegrationTest> {
        inventory::iter::<IntegrationTest>
            .into_iter()
            .find(|t| t.name == test_name.as_ref())
    }
}

inventory::collect!(IntegrationTest);
