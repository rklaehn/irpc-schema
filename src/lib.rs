#![cfg_attr(irpc_schema_docsrs, feature(doc_cfg))]
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt, vec,
};

use serde::{Deserialize, Serialize};

/// Wraps a schema with a name.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Named(pub String, pub Schema);

#[cfg(all(feature = "derive", feature = "irpc"))]
#[cfg_attr(
    irpc_schema_docsrs,
    doc(cfg(all(feature = "derive", feature = "irpc")))
)]
pub use irpc_schema_derive::serialize_service;
#[cfg(feature = "derive")]
#[cfg_attr(irpc_schema_docsrs, doc(cfg(feature = "derive")))]
pub use irpc_schema_derive::{schema, serialize_stable};

/// The schema enum
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Schema {
    /// the unit type
    Unit,
    /// the bottom type
    Bottom,
    /// an opaque atomic type, identified by its name
    Atom(String),
    /// a product type, aka tuple
    Product(Vec<Schema>),
    /// a sum type, aka unnamed enum
    Sum(Vec<Schema>),
    /// a struct type, tuple with named fields
    Struct(Vec<Named>),
    /// an enum type
    Enum(Vec<Named>),
    /// a named type
    Named(Box<Named>),
    /// a sequence type
    Seq(Box<Schema>),
    /// a set type
    Set(Box<Schema>),
    /// a map type
    Map(Box<Schema>, Box<Schema>),
}

/// Combines a schema with its stable hash.
///
/// This is just to avoid the overhead of calling `stable_hash` every time.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaAndHash {
    pub schema: Schema,
    pub hash: [u8; 32],
}

impl From<Schema> for SchemaAndHash {
    fn from(schema: Schema) -> Self {
        let hash = *schema.stable_hash().as_bytes();
        SchemaAndHash { schema, hash }
    }
}

impl fmt::Display for Named {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\":{}", self.0, self.1)
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Bottom type ⊥
            Schema::Bottom => write!(f, "⊥"),

            // Unit type ()
            Schema::Unit => write!(f, "()"),

            // Atom (String, u32, etc.)
            Schema::Atom(name) => write!(f, "\"{}\"", name),

            // Product types, tuples with one or more fields: X, Y, Z,
            Schema::Product(types) => {
                let elements = types
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "({})", elements)
            }

            // Named struct: "field": X, "field2": Y,
            Schema::Struct(fields) => {
                let fields_str = fields
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "({})", fields_str)
            }

            // Sum types, enums with one or more variants: X | Y | Z |
            Schema::Sum(types) => {
                let variants = types
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join("|");
                write!(f, "({})", variants)
            }

            // Named enum: "variant": X | "variant2": Y |
            Schema::Enum(variants) => {
                let variants_str = variants
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join("|");
                write!(f, "({})", variants_str)
            }

            // Named type: Named("name": X)
            Schema::Named(named) => {
                write!(f, "{}", named)
            }

            // Sequence type (array): Seq(X)
            Schema::Seq(item) => write!(f, "[{}]", item),

            // Set type: Set(X)
            Schema::Set(item) => write!(f, "{{{}}}", item),

            // Map type: Map(X, Y)
            Schema::Map(key, value) => write!(f, "{{{}:{}}}", key, value),
        }
    }
}

impl Named {
    pub fn new(name: impl Into<String>, schema: Schema) -> Self {
        Named(name.into(), schema)
    }

    pub fn pretty_print(&self, indent: usize) -> String {
        let indentation = " ".repeat(indent);
        let inner = self.1.pretty_print(indent);
        format!("{}\"{}\": {}", indentation, self.0, inner.trim_start(),)
    }
}

impl Schema {
    pub fn named(name: impl Into<String>, schema: Schema) -> Schema {
        Schema::Named(Box::new(Named::new(name, schema)))
    }

    pub fn pretty_print(&self, indent: usize) -> String {
        let indentation = " ".repeat(indent);
        match self {
            Schema::Bottom => format!("{}⊥", indentation),
            Schema::Unit => format!("{}()", indentation),
            Schema::Atom(name) => format!("{}\"{}\"", indentation, name),

            // Product: Each field on a new line, indented
            Schema::Product(types) => {
                let elements = types
                    .iter()
                    .map(|t| t.pretty_print(indent + 2))
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!("{}(\n{}\n{})", indentation, elements, indentation)
            }

            // Named Struct: Each "field": value on a new line
            Schema::Struct(fields) => {
                let fields_str = fields
                    .iter()
                    .map(|t| t.pretty_print(indent + 2))
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!("{}(\n{}\n{})", indentation, fields_str, indentation)
            }

            // Sum types: Each variant on a new line, separated by |
            Schema::Sum(types) => {
                let variants = types
                    .iter()
                    .map(|t| t.pretty_print(indent + 2))
                    .collect::<Vec<_>>()
                    .join(" |\n");
                format!("{}(\n{}\n{})", indentation, variants, indentation)
            }

            // Named Enum: Each "variant": value on a new line
            Schema::Enum(variants) => {
                let variants_str = variants
                    .iter()
                    .map(|t| t.pretty_print(indent + 2))
                    .collect::<Vec<_>>()
                    .join(" |\n");
                format!("{}(\n{}\n{})", indentation, variants_str, indentation)
            }

            // Named Type
            Schema::Named(named) => named.pretty_print(indent),

            // Sequence
            Schema::Seq(item) => format!(
                "{}[\n{}\n{}]",
                indentation,
                item.pretty_print(indent + 2),
                indentation
            ),

            // Set
            Schema::Set(item) => format!(
                "{}{{\n{}\n{}}}",
                indentation,
                item.pretty_print(indent + 2),
                indentation
            ),

            // Map
            Schema::Map(key, value) => {
                let k = key.pretty_print(indent + 2);
                let v = value.pretty_print(indent + 2);
                format!(
                    "{}{{\n{}: {}\n{}}}",
                    indentation,
                    k,
                    v.trim_start(),
                    indentation
                )
            }
        }
    }

    pub fn stable_hash(&self) -> blake3::Hash {
        let bytes = postcard::to_allocvec(self).unwrap();
        blake3::hash(&bytes)
    }
}

/// Trait to attach a schema to a type.
pub trait HasSchema {
    /// Returns the schema for this type.
    fn schema() -> Schema;
}

// Declare Schema for atom types
macro_rules! declare_atom {
    ($($t:ty),*) => {
        $(
            impl HasSchema for $t {
                fn schema() -> Schema {
                    Schema::Atom(stringify!($t).to_string())
                }
            }
        )*
    };
}

declare_atom!(
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    String,
    &str,
    &[u8]
);

impl<T: HasSchema> HasSchema for Vec<T> {
    fn schema() -> Schema {
        Schema::Seq(Box::new(T::schema()))
    }
}

impl<T: HasSchema> HasSchema for BTreeSet<T> {
    fn schema() -> Schema {
        Schema::Set(Box::new(T::schema()))
    }
}

impl<K: HasSchema, V: HasSchema> HasSchema for BTreeMap<K, V> {
    fn schema() -> Schema {
        Schema::Map(Box::new(K::schema()), Box::new(V::schema()))
    }
}

impl<T: HasSchema> HasSchema for HashSet<T> {
    fn schema() -> Schema {
        Schema::Set(Box::new(T::schema()))
    }
}

impl<T: HasSchema> HasSchema for Option<T> {
    fn schema() -> Schema {
        Schema::Sum(vec![Schema::Unit, T::schema()])
    }
}

impl<T: HasSchema> HasSchema for Box<T> {
    fn schema() -> Schema {
        T::schema()
    }
}

impl<T: HasSchema> HasSchema for std::sync::Arc<T> {
    fn schema() -> Schema {
        T::schema()
    }
}

impl<T: HasSchema> HasSchema for std::rc::Rc<T> {
    fn schema() -> Schema {
        T::schema()
    }
}

impl HasSchema for () {
    fn schema() -> Schema {
        Schema::Unit
    }
}

impl<A: HasSchema, B: HasSchema> HasSchema for std::result::Result<A, B> {
    fn schema() -> Schema {
        Schema::Enum(vec![
            Named("Ok".to_string(), A::schema()),
            Named("Err".to_string(), B::schema()),
        ])
    }
}

impl<A: HasSchema, B: HasSchema> HasSchema for (A, B) {
    fn schema() -> Schema {
        Schema::Product(vec![A::schema(), B::schema()])
    }
}

impl<A: HasSchema, B: HasSchema, C: HasSchema> HasSchema for (A, B, C) {
    fn schema() -> Schema {
        Schema::Product(vec![A::schema(), B::schema(), C::schema()])
    }
}

impl<K: HasSchema, V: HasSchema> HasSchema for HashMap<K, V> {
    fn schema() -> Schema {
        Schema::Map(Box::new(K::schema()), Box::new(V::schema()))
    }
}

#[cfg(feature = "irpc")]
mod irpc_instances {
    use super::{HasSchema, Schema};

    impl<T: HasSchema> HasSchema for irpc::channel::oneshot::Receiver<T> {
        fn schema() -> Schema {
            Schema::named("irpc::channel::oneshot::Receiver", T::schema())
        }
    }

    impl<T: HasSchema> HasSchema for irpc::channel::spsc::Receiver<T> {
        fn schema() -> Schema {
            Schema::named("irpc::channel::spsc::Receiver", T::schema())
        }
    }

    impl HasSchema for irpc::channel::none::NoReceiver {
        fn schema() -> Schema {
            Schema::Atom("irpc::channel::none::NoReceiver".to_string())
        }
    }

    impl<T: HasSchema> HasSchema for irpc::channel::oneshot::Sender<T> {
        fn schema() -> Schema {
            Schema::named("irpc::channel::oneshot::Sender", T::schema())
        }
    }

    impl<T: HasSchema> HasSchema for irpc::channel::spsc::Sender<T> {
        fn schema() -> Schema {
            Schema::named("irpc::channel::spsc::Sender", T::schema())
        }
    }

    impl HasSchema for irpc::channel::none::NoSender {
        fn schema() -> Schema {
            Schema::Atom("irpc::channel::none::NoSender".to_string())
        }
    }

    /// Helper trait to summon a schema for that includes the initial message type
    /// as well as the receiver and sender types, for a given service.
    pub trait ChannelsSchema<S: irpc::Service>: irpc::Channels<S> {
        /// Returns the schema for this type, including the receiver and sender kinds and types.
        fn schema() -> Schema;
    }

    impl<S, C> ChannelsSchema<S> for C
    where
        S: irpc::Service,
        C: irpc::Channels<S>,
        C::Rx: HasSchema,
        C::Tx: HasSchema,
        C: HasSchema,
    {
        fn schema() -> Schema {
            <(C, C::Rx, C::Tx)>::schema()
        }
    }
}

#[cfg_attr(irpc_schema_docsrs, doc(cfg(feature = "irpc")))]
#[cfg(feature = "irpc")]
pub use irpc_instances::ChannelsSchema;
