use std::fmt;

use schema_macro::schema;

#[derive(Debug, PartialEq, Eq)]
pub struct Named(String, ReifiedSchema);

// Define the ReifiedSchema enum
#[derive(Debug, PartialEq, Eq)]
pub enum ReifiedSchema {
    /// the unit type
    Unit,
    /// the bottom type
    Bottom,
    /// an opaque atomic type, identified by its name
    Atom(String),
    /// a product type, aka tuple
    Product(Vec<ReifiedSchema>),
    /// a sum type, aka unnamed enum
    Sum(Vec<ReifiedSchema>),
    /// a struct type, tuple with named fields
    Struct(Vec<Named>),
    /// an enum type
    Enum(Vec<Named>),
    /// a named type
    Named(Box<Named>),
    /// a sequence type
    Seq(Box<ReifiedSchema>),
    /// a set type
    Set(Box<ReifiedSchema>),
    /// a map type
    Map(Box<ReifiedSchema>, Box<ReifiedSchema>),
}


impl fmt::Display for Named {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\":{}", self.0, self.1)
    }
}

impl fmt::Display for ReifiedSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Bottom type ⊥
            ReifiedSchema::Bottom => write!(f, "⊥"),

            // Unit type ()
            ReifiedSchema::Unit => write!(f, "()"),

            // Atom (String, u32, etc.)
            ReifiedSchema::Atom(name) => write!(f, "\"{}\"", name),

            // Product types, tuples with one or more fields: X, Y, Z,
            ReifiedSchema::Product(types) => {
                let elements = types.iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "({})", elements)
            }

            // Sum types, enums with one or more variants: X | Y | Z |
            ReifiedSchema::Sum(types) => {
                let variants = types.iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join("|");
                write!(f, "({})", variants)
            }

            // Named struct: "field": X, "field2": Y,
            ReifiedSchema::Struct(fields) => {
                let fields_str = fields.iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "({})", fields_str)
            }

            // Named enum: "variant": X | "variant2": Y |
            ReifiedSchema::Enum(variants) => {
                let variants_str = variants.iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join("|");
                write!(f, "({})", variants_str)
            }

            // Named type: Named("name": X)
            ReifiedSchema::Named(named) => {
                write!(f, "{}", named)
            }

            // Sequence type (array): Seq(X)
            ReifiedSchema::Seq(item) => write!(f, "Seq({})", item),

            // Set type: Set(X)
            ReifiedSchema::Set(item) => write!(f, "Set({})", item),

            // Map type: Map(X, Y)
            ReifiedSchema::Map(key, value) => write!(f, "Map({}, {})", key, value),
        }
    }
}

// The Schema trait now returns a ReifiedSchema
pub trait Schema {
    fn schema() -> ReifiedSchema;
}

// Declare Schema for atom types
macro_rules! declare_atom {
    ($($t:ty),*) => {
        $(
            impl Schema for $t {
                fn schema() -> ReifiedSchema {
                    ReifiedSchema::Atom(stringify!($t).to_string())
                }
            }
        )*
    };
}

declare_atom!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String);

impl<T: Schema> Schema for Vec<T> {
    fn schema() -> ReifiedSchema {
        ReifiedSchema::Seq(Box::new(T::schema()))
    }
}

#[schema(Nominal)]
struct UnitStruct;

#[schema(Nominal)]
struct NominalTupleStruct(i32, String);

#[schema(Nominal)]
struct NominalStruct {
    id: i32,
    name: String,
}

#[schema(Nominal)]
enum NominalEnum {
    Tuple(i32, String),
    NominalTupleStruct(NominalTupleStruct),
    Record { id: i32, name: String },
    NominalStruct(NominalStruct),
    Unit,
    UnitStruct(UnitStruct),
}

#[schema(Structural)]
enum StructuralEnum {
    Tuple(i32, String),
    NominalTupleStruct(NominalTupleStruct),
    Record { id: i32, name: String },
    NominalStruct(NominalStruct),
    Unit,
    UnitStruct(UnitStruct),
}

#[test]
fn test_nominal_enum() {
    println!("NominalEnum: {}", NominalEnum::schema());
    assert_eq!(
        NominalEnum::schema(),
        ReifiedSchema::Atom("Request".to_string())
    );
}

#[test]
fn test_structural_enum() {
    println!("StructuralEnum: {}", StructuralEnum::schema());
    assert_eq!(
        StructuralEnum::schema(),
        ReifiedSchema::Atom("Request".to_string())
    );
}

mod output {
    use super::*;

    enum Test {
        Foo,
        Bar(),
    }
}
