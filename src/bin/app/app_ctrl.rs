use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Weak};

use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use glib::{ToValue, VariantType};
use gtk::prelude::*;

use enum_map::{enum_map, EnumMap};
use itertools::izip;
use mvjson::*;

use super::client::Client;
use super::helpers::*;

pub struct ApplicationController {
    current: Option<Current>,
    g_actions: EnumMap<ActionType, gio::SimpleAction>,
    recipes_menu: gio::Menu,
    recipes_menu_section: gio::Menu,
    result_stores: HashMap<String, gtk::ListStore>,
    state_machine_pixbufs: EnumMap<State, Option<Pixbuf>>,
    actions_stack: Option<gtk::Stack>,
    state_machine_image: Option<gtk::Image>,
    menu_icons: EnumMap<ActionType, Option<gtk::Image>>,
    results_stack: Option<gtk::Stack>,
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
            ActionType::Abort => gio::SimpleAction::new("abort", None),
        };

        for (_, g_action) in &g_actions {
            map.add_action(g_action);
        }

        let recipes_menu_section = gio::Menu::new();
        let recipes_menu = gio::Menu::new();
        recipes_menu.append_section(Some("Prepare Recipe"), &recipes_menu_section);

        let mut state_machine_pixbufs = enum_map! { _ => None };
        for (state, pixbuf_opt) in &mut state_machine_pixbufs {
            *pixbuf_opt = Pixbuf::from_file(format!("res/img/state_machine/{:?}.svg", state)).ok();
        }

        ApplicationController {
            current: None,
            g_actions,
            recipes_menu,
            recipes_menu_section,
            result_stores: HashMap::new(),
            state_machine_pixbufs,
            actions_stack: None,
            state_machine_image: None,
            menu_icons: enum_map! {_ => None},
            results_stack: None,
            weak_client,
        }
    }

    pub fn connect_callbacks(
        app: &gtk::Application,
        ctrl: &Arc<RefCell<ApplicationController>>,
        current_rx: glib::Receiver<Current>,
        rlist_rx: glib::Receiver<Vec<Recipe>>,
        result_rx: glib::Receiver<VisionResult>,
    ) {
        let current_rx_cell = Cell::new(Some(current_rx));
        let rlist_rx_cell = Cell::new(Some(rlist_rx));
        let result_rx_cell = Cell::new(Some(result_rx));

        app.connect_activate(weak!(ctrl => move |app| {
            let ctrl_strong = ctrl.upgrade().unwrap();
            let icon_theme = gtk::IconTheme::get_default().unwrap();
            icon_theme.append_search_path("res/icons/actions");
            ctrl_strong.borrow_mut().build_ui(app);

            current_rx_cell.take().unwrap().attach(
                None,
                clone!(ctrl => move |current| {
                    ctrl.upgrade().unwrap().borrow_mut().update_current(current);
                    glib::Continue(true)
                }),
            );

            rlist_rx_cell.take().unwrap().attach(
                None,
                clone!(ctrl => move |recipe_list| {
                    ctrl.upgrade().unwrap().borrow_mut().update_recipe_list(recipe_list);
                    glib::Continue(true)
                }),
            );

            result_rx_cell.take().unwrap().attach(
                None,
                clone!(ctrl => move |result| {
                    ctrl.upgrade().unwrap().borrow_mut().new_result(result);
                    glib::Continue(true)
                }),
            );
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

        let recipes_popover: gtk::Popover = builder.get_object("recipes-popover").unwrap();
        recipes_popover.bind_model(Some(&self.recipes_menu), None);

        let recipes_submenu: gtk::Box = builder.get_object("recipes-submenu").unwrap();
        let recipes_submenu_offscreen_popover = gtk::PopoverMenu::new();
        recipes_submenu_offscreen_popover.bind_model(Some(&self.recipes_menu), None);
        let offscreen_stack: gtk::Stack = recipes_submenu_offscreen_popover
            .get_child()
            .unwrap()
            .downcast()
            .unwrap();
        let recipes_submenu_box: gtk::Box = offscreen_stack.get_children()[0]
            .clone()
            .downcast()
            .unwrap();
        recipes_submenu_box.set_property_margin(0);
        offscreen_stack.remove(&recipes_submenu_box);
        recipes_submenu.add(&recipes_submenu_box);

        self.results_stack = builder.get_object("results-stack");
    }

    fn react(&self, action: Action) {
        self.weak_client
            .upgrade()
            .ok_or("Could not acquire MQTT client instance")
            .map(|strong_client| {
                strong_client
                    .borrow()
                    .publish("merlic/action/json", &action)
            })
            .unwrap();
    }

    pub fn update_current(&mut self, current: Current) {
        for (allowed, g_action, icon_opt) in izip!(
            available_actions(current.state).values(),
            self.g_actions.values(),
            self.menu_icons.values()
        ) {
            g_action.set_enabled(*allowed);
            if let Some(icon) = icon_opt {
                icon.set_opacity(if *allowed { 1.0 } else { 0.5 });
            }
        }

        if let Some(stack) = &self.actions_stack {
            stack.set_visible_child_name(&*format!("{:?}-pane", current.state).to_lowercase());
        }

        if let Some(image) = &self.state_machine_image {
            image.set_from_pixbuf(self.state_machine_pixbufs[current.state].as_ref());
        }

        self.current = Some(current);
    }

    pub fn update_recipe_list(&mut self, recipe_list: Vec<Recipe>) {
        self.recipes_menu_section.remove_all();

        let results_stack = self.results_stack.as_ref().unwrap();
        for child in &results_stack.get_children() {
            results_stack.remove(child);
        }

        self.result_stores.clear();

        for recipe in recipe_list {
            let short_desc = ellipt(&recipe.description, 25);
            self.recipes_menu_section.append(
                Some(&*format!("{}: {}", recipe.id, &short_desc)),
                Some(&*format!("app.prepare_recipe('{}')", recipe.id)),
            );

            let result_builder = gtk::Builder::from_file("res/ui/ResultsPane.ui");
            let result_pane: gtk::Box = result_builder.get_object("outer-box").unwrap();
            results_stack.add(&result_pane);
            results_stack.set_child_name(&result_pane, Some(&recipe.id));
            results_stack.set_child_title(
                &result_pane,
                Some(&*format!("{}: {}", recipe.id, &short_desc)),
            );

            let col_entries = [("Result ID", glib::Type::U32), ("Job ID", glib::Type::U32)]
                .iter()
                .copied()
                .chain(
                    recipe
                        .outputs
                        .iter()
                        .map(|p| (p.name.as_str(), p.data_type.as_glib_type())),
                );

            let results_tree: gtk::TreeView = result_builder.get_object("results-tree").unwrap();
            for (i, (title, _)) in col_entries.clone().enumerate() {
                let col = gtk::TreeViewColumn::new();
                let cell = gtk::CellRendererText::new();
                cell.set_property_alignment(pango::Alignment::Right);
                // cell.set_property_xalign(1.0);
                col.set_title(title);
                col.pack_start(&cell, true);
                col.add_attribute(&cell, "text", i as i32);
                col.set_resizable(true);
                results_tree.append_column(&col);
            }

            let result_store =
                gtk::ListStore::new(col_entries.map(|(_, t)| t).collect::<Vec<_>>().as_slice());
            results_tree.set_model(Some(&result_store));

            self.result_stores.insert(recipe.id.clone(), result_store);
        }
    }

    pub fn new_result(&mut self, result: VisionResult) {
        if let Some(store) = self.result_stores.get(&result.recipe_id) {
            let ids: Vec<&dyn ToValue> = vec![&result.id, &result.job_id];
            let vals: Vec<_> = ids
                .into_iter()
                .chain(
                    result
                        .content
                        .iter()
                        .map(|item| -> &dyn ToValue { &item.value }),
                )
                .collect();
            store.insert_with_values(
                None,
                (0u32..(vals.len() as u32)).collect::<Vec<u32>>().as_slice(),
                vals.as_slice(),
            );
        }
    }
}
