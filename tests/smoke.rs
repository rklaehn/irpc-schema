use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt,
};

use irpc_schema::{HasSchema, Named, Schema};
use irpc_schema_derive::schema;

#[schema(Nominal)]
struct UnitStruct;

#[schema(Nominal)]
enum BottomEnum {}

#[schema(Nominal)]
enum SingleCaseEnum {
    Case1,
}

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
    BottomEnum(BottomEnum),
    SingleCaseEnum(SingleCaseEnum),
    Seq(Vec<u64>),
    Set(BTreeSet<u64>),
    Map(BTreeMap<u64, u64>),
}

#[schema(Structural)]
enum StructuralEnum {
    Tuple(i32, String),
    NominalTupleStruct(NominalTupleStruct),
    Record { id: i32, name: String },
    NominalStruct(NominalStruct),
    Unit,
    UnitStruct(UnitStruct),
    BottomEnum(BottomEnum),
    SingleCaseEnum(SingleCaseEnum),
}

#[test]
fn test_pretty_print() {
    println!("{}", StructuralEnum::schema().pretty_print(0));
    println!("{}", NominalEnum::schema().pretty_print(0));
    println!("{}", UnitStruct::schema().pretty_print(0));
    println!("{}", BottomEnum::schema().pretty_print(0));
    println!("{}", SingleCaseEnum::schema().pretty_print(0));
    println!("{}", NominalTupleStruct::schema().pretty_print(0));
    println!("{}", NominalStruct::schema().pretty_print(0));
    println!("{}", NominalEnum::schema().pretty_print(0));
}

#[test]
fn test_unit_struct_schema() {
    assert_eq!(
        UnitStruct::schema(),
        Schema::named("UnitStruct", Schema::Unit)
    );
}

#[test]
fn test_bottom_enum_schema() {
    assert_eq!(
        BottomEnum::schema(),
        Schema::named("BottomEnum", Schema::Bottom)
    );
}

#[test]
fn test_nominal_enum() {
    println!("NominalEnum: {}", NominalEnum::schema());
    println!("{}", NominalEnum::schema().pretty_print(0));
}

#[test]
fn test_structural_enum() {
    println!("StructuralEnum: {}", StructuralEnum::schema());
    println!("{}", StructuralEnum::schema().pretty_print(0));
}

#[test]
fn test_enum_cases() {
    let schema = NominalEnum::schema();
    let Schema::Named(name) = schema else {
        panic!("Expected Named");
    };
    let Schema::Enum(cases) = name.1 else {
        panic!("Expected Enum");
    };
    for Named(name, value) in cases {
        println!("{}: {}", name, value.stable_hash());
    }
}

mod output {
    use super::*;

    enum Test {
        Foo,
        Bar(),
    }
}
