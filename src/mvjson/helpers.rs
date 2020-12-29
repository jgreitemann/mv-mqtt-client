extern crate enum_map;

use super::definitions::{ActionType, State};
use enum_map::{enum_map, EnumMap};

pub fn available_actions(state: State) -> EnumMap<ActionType, bool> {
    match state {
        State::Preoperational => enum_map! {
            ActionType::SelectModeAutomatic | ActionType::Halt => true,
            _ => false,
        },
        State::Halted => enum_map! {
            ActionType::Reset => true,
            _ => false
        },
        State::Error => enum_map! {
            _ => false
        },
        State::Initialized => enum_map! {
            ActionType::PrepareRecipe | ActionType::Reset | ActionType::Halt => true,
            _ => false
        },
        State::Ready => enum_map! {
            ActionType::PrepareRecipe | ActionType::UnprepareRecipe
            | ActionType::StartSingleJob | ActionType::StartContinuous
            | ActionType::Reset | ActionType::Halt => true,
            _ => false
        },
        State::SingleExecution => enum_map! {
            ActionType::Reset | ActionType::Halt | ActionType::Stop | ActionType::Abort => true,
            _ => false
        },
        State::ContinuousExecution => enum_map! {
            ActionType::Reset | ActionType::Halt | ActionType::Stop | ActionType::Abort => true,
            _ => false
        },
        State::FrontendAccess => enum_map! {
            _ => false
        },
    }
}
