extern crate enum_map;

use enum_map::Enum;

#[derive(Copy, Clone, Debug, Enum)]
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

pub struct Monitor {
    state: State,
    recipe_id: Option<String>,
    job_id: Option<u32>,
}
