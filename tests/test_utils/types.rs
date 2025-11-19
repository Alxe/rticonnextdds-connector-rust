//! Type definitions from `Test.xml` used in tests.

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Enum corresponding to TestEnum in `Test.xml`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Default,
)]
#[repr(u8)]
pub enum TestEnum {
    #[default]
    Red = 0,
    Green = 1,
    Blue = 2,
}

/// Enum corresponding to TestUnionKind in `Test.xml`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
#[derive(Default)]
pub enum TestUnionKind {
    #[default]
    String = 0,
    Number = 1,
    Boolean = 2,
}

/// Union corresponding to TestUnion in `Test.xml`
#[derive(Debug, Clone, PartialEq)]

pub enum TestUnion {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl std::default::Default for TestUnion {
    fn default() -> Self {
        TestUnion::String(String::new())
    }
}

impl Serialize for TestUnion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("TestUnion", 1)?;
        match self {
            TestUnion::String(s) => {
                // "string" : "some text"
                state.serialize_field("string", s)?;
            }
            TestUnion::Number(n) => {
                // "number" : 123.45
                state.serialize_field("number", n)?;
            }
            TestUnion::Boolean(b) => {
                // "boolean" : "true/false"
                state.serialize_field("boolean", b)?;
            }
        }
        state.end()
    }
}

impl<'a> Deserialize<'a> for TestUnion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        #[derive(Deserialize)]
        struct TestUnionHelper {
            string: Option<String>,
            number: Option<f64>,
            boolean: Option<bool>,
        }

        let helper = TestUnionHelper::deserialize(deserializer)?;
        if let Some(s) = helper.string {
            Ok(TestUnion::String(s))
        } else if let Some(n) = helper.number {
            Ok(TestUnion::Number(n))
        } else if let Some(b) = helper.boolean {
            Ok(TestUnion::Boolean(b))
        } else {
            Err(serde::de::Error::custom("No valid union member found"))
        }
    }
}

/// Struct corresponding to SimpleStruct in `Test.xml`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]

pub struct SimpleStruct {
    pub long_field: i32,
    pub double_field: f64,
    pub boolean_field: bool,
    pub string_field: String,
    pub enum_field: TestEnum,
}

/// Struct corresponding to OptionalStruct in `Test.xml`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]

pub struct OptionalStruct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_field: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_field: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boolean_field: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_field: Option<TestEnum>,
}

/// Struct corresponding to ComplexStruct in `Test.xml`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ComplexStruct {
    pub simple: SimpleStruct,
    pub optional: OptionalStruct,
    pub union: TestUnion,
    pub long_matrix: [[i32; 3]; 3],
    pub string_array: [String; 3],
    pub double_sequence: Vec<f64>,
}
