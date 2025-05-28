# Irpc-Schema

This is a simple schema language for rust types. The intent is not to be a fully comprehensive schema language that supports code generation etc, but just to provide the ability to evolve a rust RPC api without losing compatibility or having to do versioning for every single minor change. The approach is similar to what is done in [postcard-rpc](https://github.com/jamesmunns/postcard-rpc), but while postcard-rpc is using u64 hash of endpoint name and schema, this crate is using 32 byte blake3 hashes for schema.

Schema hashes are computed by postcard-encoding the schema and then computing the blake3 hash.

# Deriving schemas

There is a macro to derive schemas for structs and enums.

When deriving a schema, you have three basic choices:

## Atom

When declaring a schema as Atom, the schema type will just be Schema::Atom("typename"). This means that as long as the type name stays the same, the type is considered to be compatible. Note that this is the *local* type name. The schema macro can not figure out the canonical type name, and in any case doing so is out of scope for this simple crate.

## Structural

When declaring a schema as structural, all naming information will be purged. E.g. a struct with named fields will be considered identital to a tuple or product type, an enum with named fields will be considered identical to a sum type.

```rust
#[schema(Structural)]
struct Point {
    x: u64,
    y: u64,
}
```

has the same schema as `(u64, u64)`. Renaming the struct or the fields does not cause a schema change.

```rust
#[schema(Structural)]
enum Test {
    Case1(u64),
    Case2(&'static str)
}
```

has the same schema as `Result<u64, &'static str>`. Renaming the cases or the enum does not cause a schema change.

## Nominal

When declaring a schema as nominal, naming information will be kept. E.g. for a struct with named fields the names of the struct and the names of the fields will be included in the schema.

```rust
#[schema(Nominal)]
struct Point {
    x: f64,
    y: f64,
}
```

is a different schema than

```rust
#[schema(Nominal)]
struct Point {
    r: f64,
    psi: f64,
}
```

despite being compatible in terms of serialized representation. Use nominal if you want to attach meaning in addition to the constituent types.

The order of elements in a nominal or structural enum matters.

# Schema evolution

