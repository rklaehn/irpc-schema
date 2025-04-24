use anyhow::Result;
use irpc_schema_derive::schema;

#[schema(Nominal)]
pub struct GetRequest {}

fn main() -> Result<()> {
    Ok(())
}
