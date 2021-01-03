use enum_map::Enum;
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
pub struct Current {
    pub state: State,
    #[serde(default)]
    pub mode: Option<ModeType>,
    #[serde(default)]
    pub recipe_id: Option<String>,
    #[serde(default)]
    pub job_id: Option<u32>,
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
