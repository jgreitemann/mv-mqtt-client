use gio::prelude::*;
use gtk::prelude::*;

use enum_map::{enum_map, EnumMap};
use itertools::izip;
use mvjson::{available_actions, ActionType, State};
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub struct App {
    application: gtk::Application,
    _app_ctrl: Rc<RefCell<ApplicationController>>,
}

impl App {
    pub fn new() -> App {
        let app = gtk::Application::new(
            Some("io.github.jgreitemann.mv-mqtt-client"),
            Default::default(),
        )
        .expect("Initialization failed...");

        let ctrl = Rc::new(RefCell::new(ApplicationController::new(&app)));
        ApplicationController::connect_callbacks(&app, &ctrl);

        App {
            application: app,
            _app_ctrl: ctrl,
        }
    }

    pub fn run(self: &App, args: Vec<String>) {
        self.application.run(&args);
    }
}

struct ApplicationController {
    g_actions: EnumMap<ActionType, gio::SimpleAction>,
    actions_stack: Option<gtk::Stack>,
    menu_icons: EnumMap<ActionType, Option<gtk::Image>>,
}

impl ApplicationController {
    fn new<T: gio::ActionMapExt>(map: &T) -> ApplicationController {
        let new = ApplicationController {
            g_actions: enum_map! {
                ActionType::SelectModeAutomatic => gio::SimpleAction::new("select_automatic_mode", None),
                ActionType::PrepareRecipe => gio::SimpleAction::new("prepare_recipe", None),
                ActionType::UnprepareRecipe => gio::SimpleAction::new("unprepare_recipe", None),
                ActionType::StartSingleJob => gio::SimpleAction::new("start_single_job", None),
                ActionType::StartContinuous => gio::SimpleAction::new("start_continuous", None),
                ActionType::Reset => gio::SimpleAction::new("reset", None),
                ActionType::Halt => gio::SimpleAction::new("halt", None),
                ActionType::Stop => gio::SimpleAction::new("stop", None),
                ActionType::Abort => gio::SimpleAction::new("abort", None)
            },
            actions_stack: None,
            menu_icons: enum_map! {_ => None},
        };

        for (_, g_action) in &new.g_actions {
            map.add_action(g_action);
        }

        return new;
    }

    fn build_ui(&mut self, app: &gtk::Application) {
        let builder = gtk::Builder::from_file("res/ui/MainWindow.ui");
        let window: gtk::ApplicationWindow = builder.get_object("window").unwrap();
        window.set_application(Some(app));

        self.actions_stack = builder.get_object("actions-stack");
        for (atype, icon_opt) in &mut self.menu_icons {
            *icon_opt = builder.get_object(&*format!("{:?}-menu-icon", atype).to_lowercase());
        }
    }

    fn connect_callbacks(app: &gtk::Application, ctrl: &Rc<RefCell<ApplicationController>>) {
        app.connect_activate(clone!(ctrl => move |app| {
            let icon_theme = gtk::IconTheme::get_default().unwrap();
            icon_theme.append_search_path("res/icons/actions");
            ctrl.borrow_mut().build_ui(app);
            ctrl.borrow().change_state(State::Preoperational);
        }));

        for (atype, g_action) in &ctrl.borrow().g_actions {
            g_action.connect_activate(clone!(ctrl => move |_, _| ctrl.borrow().react(atype)));
        }
    }

    fn react(&self, atype: ActionType) {
        let to_state = match atype {
            ActionType::SelectModeAutomatic => State::Initialized,
            ActionType::PrepareRecipe => State::Ready,
            ActionType::UnprepareRecipe => State::Initialized,
            ActionType::StartSingleJob => State::SingleExecution,
            ActionType::StartContinuous => State::ContinuousExecution,
            ActionType::Reset => State::Preoperational,
            ActionType::Halt => State::Halted,
            ActionType::Stop => State::Ready,
            ActionType::Abort => State::Ready,
        };
        self.change_state(to_state);
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
    }
}
