# Rust-Rpc-Schema

This is a simple schema language for rust types. The intent is not to be a fully comprehensive schema language that supports code generation etc, but just to provide the ability to evolve a rust RPC api without losing compatibility or having to do versioning for every single minor change. The approach is similar to what is done in [postcard-rpc](https://github.com/jamesmunns/postcard-rpc), but while postcard-rpc is using u64 hash of endpoint name and schema, this crate is using 32 byte blake3 hashes for schema.
