use enum_map::Enum;
use glib::{Type, Value};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Enum, Deserialize, Serialize)]
pub enum State {
    Preoperational,
    Halted,
    Error,
    Initialized,
    Ready,
    SingleExecution,
    ContinuousExecution,
    FrontendAccess,
}

#[derive(Copy, Clone, Debug, Enum)]
pub enum ActionType {
    SelectModeAutomatic,
    PrepareRecipe,
    UnprepareRecipe,
    StartSingleJob,
    StartContinuous,
    Reset,
    Halt,
    Stop,
    Abort,
}

#[derive(Copy, Clone, Debug, Enum, Deserialize, Serialize)]
pub enum ModeType {
    Automatic,
    FrontendAccess,
}

#[derive(Copy, Clone, Debug, Enum, Deserialize, Serialize)]
pub enum ValueType {
    #[serde(rename = "Scalar")]
    ScalarValue,
    #[serde(rename = "Array")]
    ArrayValue,
}

#[derive(Copy, Clone, Debug, Enum, Deserialize, Serialize)]
pub enum DataType {
    Bool,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float,
    Double,
    String,
    Variant,
}

#[derive(Copy, Clone, Debug, Enum, Deserialize, Serialize)]
pub enum ResultState {
    Completed,
    Processing,
    Aborted,
    Failed,
}

#[derive(Serialize)]
#[serde(tag = "actionType")]
pub enum Action {
    SelectMode {
        mode: ModeType,
    },
    PrepareRecipe {
        #[serde(rename = "recipeId")]
        recipe_id: String,
    },
    UnprepareRecipe {
        #[serde(rename = "recipeId", skip_serializing_if = "Option::is_none")]
        recipe_id: Option<String>,
    },
    StartSingleJob {
        #[serde(rename = "recipeId", skip_serializing_if = "Option::is_none")]
        recipe_id: Option<String>,
    },
    StartContinuous {
        #[serde(rename = "recipeId", skip_serializing_if = "Option::is_none")]
        recipe_id: Option<String>,
    },
    Reset,
    Halt,
    Stop,
    Abort,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeParam {
    pub name: String,
    pub description: String,
    pub value_type: ValueType,
    pub data_type: DataType,
}

#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub description: String,
    pub inputs: Vec<RecipeParam>,
    pub outputs: Vec<RecipeParam>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResultValue {
    Boolean(bool),
    Integer(i64),
    FloatingPoint(f64),
    String(String),
}

#[derive(Debug, Deserialize)]
pub struct ResultItem {
    pub name: String,
    pub value: ResultValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionResult {
    pub id: u32,
    pub recipe_id: String,
    pub job_id: u32,
    pub timestamp: String,
    pub result_state: ResultState,
    pub content: Vec<ResultItem>,
}

impl glib::ToValue for ResultValue {
    fn to_value(&self) -> Value {
        match self {
            ResultValue::Boolean(b) => b.to_value(),
            ResultValue::Integer(i) => i.to_value(),
            ResultValue::FloatingPoint(f) => f.to_value(),
            ResultValue::String(s) => s.to_value(),
        }
    }

    fn value_type(&self) -> Type {
        match self {
            ResultValue::Boolean(_) => Type::BOOL,
            ResultValue::Integer(_) => Type::I64,
            ResultValue::FloatingPoint(_) => Type::F64,
            ResultValue::String(_) => Type::STRING,
        }
    }
}
