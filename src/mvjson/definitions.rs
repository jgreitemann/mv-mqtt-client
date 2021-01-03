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
