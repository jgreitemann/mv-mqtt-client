extern crate enum_map;

use crate::{ActionType, DataType, State};
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

impl DataType {
    pub fn as_glib_type(&self) -> glib::Type {
        match self {
            DataType::Bool => glib::Type::Bool,
            DataType::Int8 => glib::Type::I8,
            DataType::UInt8 => glib::Type::U8,
            DataType::Int16 => glib::Type::I32,
            DataType::UInt16 => glib::Type::U32,
            DataType::Int32 => glib::Type::I32,
            DataType::UInt32 => glib::Type::U32,
            DataType::Int64 => glib::Type::I64,
            DataType::UInt64 => glib::Type::U64,
            DataType::Float => glib::Type::F32,
            DataType::Double => glib::Type::F64,
            DataType::String => glib::Type::String,
            DataType::Variant => glib::Type::String,
        }
    }
}
