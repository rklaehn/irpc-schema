use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt, io, vec,
};

use anyhow::Context;
use irpc::util::AsyncReadVarintExt;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::error;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Named(pub String, pub Schema);

// Define the ReifiedSchema enum
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
            Schema::Named(named) => format!("{}", named.pretty_print(indent)),

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
        let hash = blake3::hash(&bytes);
        hash
    }
}

// The Schema trait now returns a ReifiedSchema
pub trait HasSchema {
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

pub struct Dispatcher {
    pub(crate) handlers: HashMap<
        [u8; 32],
        Box<
            dyn Fn(&[u8], quinn::SendStream, quinn::RecvStream) -> n0_future::boxed::BoxFuture<()>
                + Send
                + Sync,
        >,
    >,
}

impl Dispatcher {
    pub fn add_handler<T: HasSchema + DeserializeOwned>(
        &mut self,
        handler: impl Fn(T, quinn::RecvStream, quinn::SendStream) -> n0_future::boxed::BoxFuture<()>
        + Send
        + Sync
        + 'static,
    ) {
        let schema = T::schema();
        let hash = schema.stable_hash();
        self.handlers.insert(
            hash.into(),
            Box::new(move |data, send, recv| {
                if let Ok(value) = postcard::from_bytes(data) {
                    handler(value, recv, send)
                } else {
                    error!("Failed to deserialize data");
                    Box::pin(async move {})
                }
            }),
        );
    }

    pub async fn handle(
        &self,
        send: quinn::SendStream,
        mut recv: quinn::RecvStream,
    ) -> anyhow::Result<()> {
        let mut hash = [0u8; 32];
        recv.read_exact(&mut hash).await?;
        let size = recv.read_varint_u64().await?.context("EOF")?;
        let mut buf = vec![0; size as usize];
        recv.read_exact(&mut buf).await?;
        if let Some(handler) = self.handlers.get(&hash) {
            handler(&buf, send, recv).await;
        } else {
            error!("No handler found for data");
        }
        Ok(())
    }
}

pub async fn send<T: Serialize + HasSchema>(
    send: &mut quinn::SendStream,
    data: &T,
) -> anyhow::Result<()> {
    let data = postcard::to_allocvec(data).context("Failed to serialize data")?;
    send.write_all(T::schema().stable_hash().as_bytes()).await?;
    send.write_all(&data).await?;
    Ok(())
}
