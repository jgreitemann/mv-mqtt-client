extern crate gio;
extern crate gtk;
extern crate enum_map;

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;
use gio::Action;
use enum_map::{enum_map, Enum, EnumMap};

#[derive(Debug, Enum)]
enum State {
    Preoperational,
    Halted,
    Error,
    Initialized,
    Ready,
    SingleExecution,
    ContinuousExecution,
    FrontendAccess,
}

#[derive(Debug, Enum)]
enum ActionType {
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

struct Monitor {
    state: State,
    recipe_id: Option<String>,
    job_id: Option<u32>,
}

fn build_ui(application: &gtk::Application) {
    let builder = gtk::Builder::from_file("res/ui/MainWindow.ui");
    let window: gtk::ApplicationWindow = builder.get_object("window").unwrap();
    window.set_application(Some(application));
}

fn available_actions(state: State) -> EnumMap<ActionType, bool> {
    match state {
        State::Preoperational => enum_map! {
            ActionType::SelectModeAutomatic => true,
            ActionType::Halt => true,
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
            ActionType::PrepareRecipe => true,
            ActionType::Reset => true,
            ActionType::Halt => true,
            _ => false
        },
        State::Ready => enum_map! {
            ActionType::PrepareRecipe => true,
            ActionType::UnprepareRecipe => true,
            ActionType::StartSingleJob => true,
            ActionType::StartContinuous => true,
            ActionType::Reset => true,
            ActionType::Halt => true,
            _ => false
        },
        State::SingleExecution => enum_map! {
            ActionType::Reset => true,
            ActionType::Halt => true,
            ActionType::Stop => true,
            ActionType::Abort => true,
            _ => false
        },
        State::ContinuousExecution => enum_map! {
            ActionType::Reset => true,
            ActionType::Halt => true,
            ActionType::Stop => true,
            ActionType::Abort => true,
            _ => false
        },
        State::FrontendAccess => enum_map! {
            _ => false
        }
    }
}

fn change_state(state: State, amap: &EnumMap<ActionType, Option<gio::SimpleAction>>) {
    for (atype, allowed) in available_actions(state) {
        if let Some(g_action) = &amap[atype] {
            g_action.set_enabled(allowed);
        }
    }
}

fn main() {
    let application =
        gtk::Application::new(Some("io.github.jgreitemann.mv-mqtt-client"), Default::default())
            .expect("Initialization failed...");

    let g_actions = enum_map! {
        ActionType::SelectModeAutomatic => Some(gio::SimpleAction::new("select_automatic_mode", None)),
        ActionType::PrepareRecipe => None,
        ActionType::UnprepareRecipe => Some(gio::SimpleAction::new("unprepare_recipe", None)),
        ActionType::StartSingleJob => Some(gio::SimpleAction::new("start_single_job", None)),
        ActionType::StartContinuous => Some(gio::SimpleAction::new("start_continuous", None)),
        ActionType::Reset => Some(gio::SimpleAction::new("reset", None)),
        ActionType::Halt => Some(gio::SimpleAction::new("halt", None)),
        ActionType::Stop => Some(gio::SimpleAction::new("stop", None)),
        ActionType::Abort => Some(gio::SimpleAction::new("abort", None))
    };

    let app_action_map: &gio::ActionMap = application.as_ref();
    for (_, g_action_opt) in &g_actions {
        if let Some(g_action) = g_action_opt {
            app_action_map.add_action(g_action);
        }
    }

    g_actions[ActionType::Stop].as_ref().unwrap().connect_activate(|_, _| { println!("Stop!") });

    application.connect_activate(|app| {
        let icon_theme = gtk::IconTheme::get_default().unwrap();
        icon_theme.append_search_path("res/icons/actions");

        build_ui(app);
    });

    change_state(State::Preoperational, &g_actions);

    application.run(&args().collect::<Vec<_>>());
}
