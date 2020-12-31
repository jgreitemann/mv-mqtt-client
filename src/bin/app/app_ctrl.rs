use std::cell::RefCell;
use std::rc::{Rc, Weak};

use enum_map::{enum_map, EnumMap};
use gio::prelude::*;
use glib::VariantType;
use gtk::prelude::*;
use itertools::izip;
use mvjson::*;

use super::client::Client;

macro_rules! weak {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = Rc::downgrade($n); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = Rc::downgrade($n); )+
            move |$(weak!(@param $p),)+| $body
        }
    );
}

pub struct ApplicationController {
    g_actions: EnumMap<ActionType, gio::SimpleAction>,
    actions_stack: Option<gtk::Stack>,
    state_machine_image: Option<gtk::Image>,
    menu_icons: EnumMap<ActionType, Option<gtk::Image>>,
    weak_client: Weak<RefCell<Client>>,
}

impl ApplicationController {
    pub fn new<T: gio::ActionMapExt>(map: &T, weak_client: Weak<RefCell<Client>>) -> Self {
        let g_actions = enum_map! {
            ActionType::SelectModeAutomatic => gio::SimpleAction::new("select_automatic_mode", None),
            ActionType::PrepareRecipe => gio::SimpleAction::new("prepare_recipe",
                                                                Some(&VariantType::new("s").unwrap())),
            ActionType::UnprepareRecipe => gio::SimpleAction::new("unprepare_recipe", None),
            ActionType::StartSingleJob => gio::SimpleAction::new("start_single_job", None),
            ActionType::StartContinuous => gio::SimpleAction::new("start_continuous", None),
            ActionType::Reset => gio::SimpleAction::new("reset", None),
            ActionType::Halt => gio::SimpleAction::new("halt", None),
            ActionType::Stop => gio::SimpleAction::new("stop", None),
            ActionType::Abort => gio::SimpleAction::new("abort", None)
        };

        for (_, g_action) in &g_actions {
            map.add_action(g_action);
        }

        ApplicationController {
            g_actions,
            actions_stack: None,
            state_machine_image: None,
            menu_icons: enum_map! {_ => None},
            weak_client,
        }
    }

    pub fn connect_callbacks(app: &gtk::Application, ctrl: &Rc<RefCell<ApplicationController>>) {
        app.connect_activate(weak!(ctrl => move |app| {
            let ctrl = ctrl.upgrade().unwrap();
            let icon_theme = gtk::IconTheme::get_default().unwrap();
            icon_theme.append_search_path("res/icons/actions");
            ctrl.borrow_mut().build_ui(app);
            ctrl.borrow().change_state(State::Preoperational);
        }));

        for (atype, g_action) in &ctrl.borrow().g_actions {
            g_action.connect_activate(weak!(ctrl => move |_, parameter| {
                let action = match atype {
                    ActionType::SelectModeAutomatic => Action::SelectMode {
                        mode: ModeType::Automatic
                    },
                    ActionType::PrepareRecipe => Action::PrepareRecipe {
                        recipe_id: parameter.unwrap().get_str().unwrap().to_string()
                    },
                    ActionType::UnprepareRecipe => Action::UnprepareRecipe { recipe_id: None },
                    ActionType::StartSingleJob => Action::StartSingleJob { recipe_id: None },
                    ActionType::StartContinuous => Action::StartContinuous { recipe_id: None },
                    ActionType::Reset => Action::Reset,
                    ActionType::Halt => Action::Halt,
                    ActionType::Stop => Action::Stop,
                    ActionType::Abort => Action::Abort,
                };
                ctrl.upgrade().unwrap().borrow().react(action)
            }));
        }
    }

    fn build_ui(&mut self, app: &gtk::Application) {
        let builder = gtk::Builder::from_file("res/ui/MainWindow.ui");
        let window: gtk::ApplicationWindow = builder.get_object("window").unwrap();
        window.set_application(Some(app));

        self.actions_stack = builder.get_object("actions-stack");
        self.state_machine_image = builder.get_object("statemachine-image");
        for (atype, icon_opt) in &mut self.menu_icons {
            *icon_opt = builder.get_object(&*format!("{:?}-menu-icon", atype).to_lowercase());
        }
    }

    fn react(&self, action: Action) {
        let to_state = match &action {
            Action::SelectMode { mode: _ } => State::Initialized,
            Action::PrepareRecipe { recipe_id: _ } => State::Ready,
            Action::UnprepareRecipe { recipe_id: _ } => State::Initialized,
            Action::StartSingleJob { recipe_id: _ } => State::SingleExecution,
            Action::StartContinuous { recipe_id: _ } => State::ContinuousExecution,
            Action::Reset => State::Preoperational,
            Action::Halt => State::Halted,
            Action::Stop => State::Ready,
            Action::Abort => State::Ready,
        };
        self.change_state(to_state);

        self.weak_client
            .upgrade()
            .ok_or("Could not acquire MQTT client instance")
            .map(|strong_client| {
                strong_client
                    .borrow()
                    .publish("merlic/action/json", &action)
            });
    }

    fn change_state(&self, to_state: State) {
        for (allowed, g_action, icon_opt) in izip!(
            available_actions(to_state).values(),
            self.g_actions.values(),
            self.menu_icons.values()
        ) {
            g_action.set_enabled(*allowed);
            if let Some(icon) = icon_opt {
                icon.set_opacity(if *allowed { 1.0 } else { 0.5 });
            }
        }

        if let Some(stack) = &self.actions_stack {
            stack.set_visible_child_name(&*format!("{:?}-pane", to_state).to_lowercase());
        }

        if let Some(image) = &self.state_machine_image {
            image.set_from_file(&*format!("res/img/state_machine/{:?}.png", to_state));
        }
    }
}
