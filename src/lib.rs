use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt, vec,
};

use serde::{Deserialize, Serialize};

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
