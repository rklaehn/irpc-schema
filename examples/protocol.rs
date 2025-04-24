use anyhow::Result;
use irpc_schema_derive::{schema, serialize_stable};
use v1::GetRequest;

mod v1 {
    use serde::{Deserialize, Serialize};

    use super::*;
    #[schema(Nominal)]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GetRequest {
        pub key: String,
    }

    #[schema(Nominal)]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct PutRequest {
        pub key: String,
        pub value: String,
    }

    #[serialize_stable]
    pub enum Proto {
        Get(GetRequest),
        Put(PutRequest),
    }
}

fn main() -> Result<()> {
    <GetRequest as ::irpc_schema::HasSchema>::schema();
    Ok(())
}
