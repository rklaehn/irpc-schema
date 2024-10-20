use schema_macro::schema;

// Declare Schema for atom types
macro_rules! declare_atom {
    ($($t:ty),*) => {
        $(
            impl Schema for $t {
                fn schema() -> String {
                    stringify!($t).to_string()
                }
            }
        )*
    };
}

declare_atom!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, String);

// Trait to define the schema
pub trait Schema {
    fn schema() -> String;
}

impl<T: Schema> Schema for Vec<T> {
    fn schema() -> String {
        format!("[{}]", T::schema())
    }
}

#[schema(Structural)]
struct TestProto {
    field: i32,
    name: String,
}

#[schema(Nominal)]
enum Request {
    Get(i32),
    Set(i32, String),
    Sub(Vec<TestProto>),
}

enum ReifiedSchema {
    Atom(String),
    Product(Vec<Box<ReifiedSchema>>),
    Sum(Vec<Box<ReifiedSchema>>),
    Struct(String, Vec<(String, Box<ReifiedSchema>)>),
    Enum(String, Vec<(String, Vec<Box<ReifiedSchema>>)>),
    Seq(Box<ReifiedSchema>),
    Set(Box<ReifiedSchema>),
    Map(Box<ReifiedSchema>, Box<ReifiedSchema>),
}

#[test]
fn test_atom_schema() {
    // assert_eq!(TestProto::schema(), "TestProto");
    assert_eq!(Request::schema(), "TestProto");
}

