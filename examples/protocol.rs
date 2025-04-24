use anyhow::Result;
use irpc_schema_derive::schema;
use irpc_schema::{Schema, HasSchema, Named};

mod v1 {
    use super::*;
    #[schema(Nominal)]
    pub struct GetRequest {
        pub key: String,
    }

    #[schema(Nominal)]
    pub struct PutRequest {
        pub key: String,
        pub value: String,
    }
}

fn main() -> Result<()> {
    Ok(())
}
