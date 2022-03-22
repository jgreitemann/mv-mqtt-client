use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use adw::prelude::*;
use gdk_pixbuf::Pixbuf;
use gio::prelude::*;
use glib::{clone, ToValue, VariantType};
use gtk4::prelude::*;
use libadwaita as adw;

use enum_map::{enum_map, EnumMap};
use itertools::izip;
use mvjson::*;

use super::helpers::*;

pub enum Message {
    StateUpdate(State),
    RecipeListUpdate(Vec<Recipe>),
    NewResult(VisionResult),
}

pub struct ApplicationController {
    g_actions: EnumMap<ActionType, gio::SimpleAction>,
    result_stores: HashMap<String, gtk4::ListStore>,
    state_machine_pixbufs: EnumMap<State, Option<Pixbuf>>,
    actions_stack: Option<gtk4::Stack>,
    state_machine_image: Option<gtk4::Image>,
    menu_icons: EnumMap<ActionType, Option<gtk4::Image>>,
    recipes_menu: Option<gio::Menu>,
    recipes_stack: Option<gtk4::Stack>,
    results_stack: Option<gtk4::Stack>,
}

impl ApplicationController {
    pub fn new<T: gio::traits::ActionMapExt>(map: &T) -> Self {
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

        let mut state_machine_pixbufs = enum_map! { _ => None };
        for (state, pixbuf_opt) in &mut state_machine_pixbufs {
            *pixbuf_opt = Pixbuf::from_resource(
                resource_path(&format!("state-machine/{:?}.png", state)).as_str(),
            )
            .ok();
        }

        ApplicationController {
            g_actions,
            result_stores: HashMap::new(),
            state_machine_pixbufs,
            actions_stack: None,
            recipes_menu: None,
            state_machine_image: None,
            menu_icons: enum_map! {_ => None},
            recipes_stack: None,
            results_stack: None,
        }
    }

    pub fn connect_callbacks(
        app: &adw::Application,
        ctrl: &Arc<RefCell<ApplicationController>>,
        message_receiver: glib::Receiver<Message>,
        action_sender: glib::Sender<Action>,
    ) {
        let rx_cell = Cell::new(Some(message_receiver));

        app.connect_activate(clone!(@weak ctrl => move |app| {
            ctrl.borrow_mut().build_ui(app);

            rx_cell.take().unwrap().attach(
                None,
                clone!(@strong ctrl => move |msg| {
                    let mut ctrl = ctrl.borrow_mut();
                    use Message::*;
                    match msg {
                        StateUpdate(state) => ctrl.update_state(state),
                        RecipeListUpdate(recipe_list) => ctrl.update_recipe_list(recipe_list),
                        NewResult(result) => ctrl.new_result(result),
                    }
                    glib::Continue(true)
                }),
            );
        }));

        for (atype, g_action) in &ctrl.borrow().g_actions {
            let g_action_sender = action_sender.clone();
            g_action.connect_activate(clone!(@weak ctrl => move |_, parameter| {
                g_action_sender.send(match atype {
                    ActionType::SelectModeAutomatic => Action::SelectMode {
                        mode: ModeType::Automatic
                    },
                    ActionType::PrepareRecipe => Action::PrepareRecipe {
                        recipe_id: parameter.unwrap().str().unwrap().to_string()
                    },
                    ActionType::UnprepareRecipe => Action::UnprepareRecipe { recipe_id: None },
                    ActionType::StartSingleJob => Action::StartSingleJob { recipe_id: None },
                    ActionType::StartContinuous => Action::StartContinuous { recipe_id: None },
                    ActionType::Reset => Action::Reset,
                    ActionType::Halt => Action::Halt,
                    ActionType::Stop => Action::Stop,
                    ActionType::Abort => Action::Abort,
                }).unwrap();
            }));
        }
    }

    fn build_ui(&mut self, app: &adw::Application) {
        let builder = gtk4::Builder::from_resource(resource_path("MainWindow.ui").as_str());
        let window: adw::ApplicationWindow = builder.object("window").unwrap();
        window.set_application(Some(app));

        self.actions_stack = builder.object("actions-stack");
        self.state_machine_image = builder.object("statemachine-image");
        for (atype, icon_opt) in &mut self.menu_icons {
            *icon_opt = builder.object(&*format!("{:?}-menu-icon", atype).to_lowercase());
        }

        self.recipes_menu = builder.object("recipes-submenu");

        let recipes_popover: gio::Menu = builder.object("recipes-popover").unwrap();
        recipes_popover.append_section(Some("Prepare Recipe"), self.recipes_menu.as_ref().unwrap());

        self.recipes_stack = builder.object("recipes-stack");
        self.results_stack = builder.object("results-stack");

        // This is a hack: calling unfullscreen causes window to honor default size.
        window.unfullscreen();

        window.present();
    }

    pub fn update_state(&mut self, state: State) {
        for (allowed, g_action, icon_opt) in izip!(
            available_actions(state).values(),
            self.g_actions.values(),
            self.menu_icons.values()
        ) {
            g_action.set_enabled(*allowed);
            if let Some(icon) = icon_opt {
                icon.set_opacity(if *allowed { 1.0 } else { 0.5 });
            }
        }

        if let Some(stack) = &self.actions_stack {
            stack.set_visible_child_name(&*format!("{:?}-pane", state).to_lowercase());
        }

        if let Some(image) = &self.state_machine_image {
            image.set_from_pixbuf(self.state_machine_pixbufs[state].as_ref());
        }
    }

    pub fn update_recipe_list(&mut self, recipe_list: Vec<Recipe>) {
        self.update_recipes_menu(&recipe_list);

        let recipes_stack = self.recipes_stack.as_ref().unwrap();
        while let Some(child) = &recipes_stack.first_child() {
            recipes_stack.remove(child);
        }

        let results_stack = self.results_stack.as_ref().unwrap();
        while let Some(child) = &results_stack.first_child() {
            results_stack.remove(child);
        }

        self.result_stores.clear();

        for recipe in recipe_list {
            let short_desc = ellipt(&recipe.description, 25);

            // Recipes tab stack panes
            let recipe_builder =
                gtk4::Builder::from_resource(resource_path("RecipesPane.ui").as_str());
            let recipe_pane: gtk4::ScrolledWindow =
                recipe_builder.object("recipes-scrolled-window").unwrap();
            recipes_stack.add_titled(
                &recipe_pane,
                Some(&recipe.id),
                format!("{}: {}", recipe.id, &short_desc).as_str(),
            );
            let recipe_desc_group: adw::PreferencesGroup =
                recipe_builder.object("recipe-desc-group").unwrap();
            let input_param_list: adw::PreferencesGroup =
                recipe_builder.object("input-param-list").unwrap();
            let output_param_list: adw::PreferencesGroup =
                recipe_builder.object("output-param-list").unwrap();
            recipe_desc_group.add(&adw::ActionRow::builder().title(&recipe.description).build());
            fill_param_rows(&input_param_list, recipe.inputs.iter());
            fill_param_rows(&output_param_list, recipe.outputs.iter());

            // Results tab stack panes
            let result_builder =
                gtk4::Builder::from_resource(resource_path("ResultsPane.ui").as_str());
            let result_pane: gtk4::Box = result_builder.object("outer-box").unwrap();
            results_stack.add_titled(
                &result_pane,
                Some(&recipe.id),
                format!("{}: {}", recipe.id, &short_desc).as_str(),
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

            let results_tree: gtk4::TreeView = result_builder.object("results-tree").unwrap();
            for (i, (title, _)) in col_entries.clone().enumerate() {
                let col = gtk4::TreeViewColumn::new();
                let cell = gtk4::CellRendererText::new();
                CellRendererTextExt::set_alignment(&cell, gtk4::pango::Alignment::Right);
                col.set_title(title);
                col.pack_start(&cell, true);
                col.add_attribute(&cell, "text", i as i32);
                col.set_resizable(true);
                results_tree.append_column(&col);
            }

            let results_scrolled_window: gtk4::ScrolledWindow =
                result_builder.object("results-scrolled-window").unwrap();
            let autoscroll_toggle: gtk4::ToggleButton =
                result_builder.object("autoscroll-toggle").unwrap();
            let autoscroll_capture = clone!(@strong results_scrolled_window,
                                            @strong autoscroll_toggle => move || {
                if autoscroll_toggle.is_active() {
                     let adj = results_scrolled_window.vadjustment();
                     adj.set_value(adj.upper() - adj.page_size());
                }
            });
            autoscroll_toggle.connect_toggled(
                clone!(@strong autoscroll_capture => move |_| autoscroll_capture()),
            );
            results_tree.connect_vadjustment_notify(move |_| autoscroll_capture());

            let result_store =
                gtk4::ListStore::new(col_entries.map(|(_, t)| t).collect::<Vec<_>>().as_slice());
            results_tree.set_model(Some(&result_store));

            let clear_button: gtk4::Button = result_builder.object("clear-results-button").unwrap();
            clear_button
                .connect_clicked(clone!(@strong result_store => move |_| result_store.clear()));

            self.result_stores.insert(recipe.id.clone(), result_store);
        }
    }

    fn update_recipes_menu(&mut self, recipe_list: &[Recipe]) {
        if let Some(menu) = &self.recipes_menu {
            menu.remove_all();

            for recipe in recipe_list {
                let short_desc = ellipt(&recipe.description, 25);

                menu.append(
                    Some(&*format!("{}: {}", recipe.id, &short_desc)),
                    Some(&*format!("app.prepare_recipe('{}')", recipe.id)),
                );
            }
        }
    }

    pub fn new_result(&mut self, result: VisionResult) {
        if let Some(store) = self.result_stores.get(&result.recipe_id) {
            let ids: Vec<&dyn ToValue> = vec![&result.id, &result.job_id];
            let vals: Vec<_> = std::iter::Iterator::zip(
                (0u32..).into_iter(),
                ids.into_iter().chain(
                    result
                        .content
                        .iter()
                        .map(|item| -> &dyn ToValue { &item.value }),
                ),
            )
            .collect();
            store.insert_with_values(None, vals.as_slice());
            self.results_stack
                .as_ref()
                .unwrap()
                .set_visible_child_name(&result.recipe_id);
        }
    }
}

fn fill_param_rows<'a, T>(group: &adw::PreferencesGroup, param_list: T)
where
    T: Iterator<Item = &'a RecipeParam>,
{
    let mut used = false;
    for param in param_list {
        used = true;
        let row = adw::ActionRow::builder()
            .selectable(false)
            .title(&param.name)
            .subtitle(&param.description)
            .build();
        row.add_suffix(
            &gtk4::Label::builder()
                .label(&*format!("{:?}", param.data_type))
                .css_classes(vec!["dim-label".to_string()])
                .build(),
        );
        group.add(&row);
    }
    AsRef::<gtk4::Widget>::as_ref(group).set_visible(used);
}

fn resource_path(resource_subpath: &str) -> String {
    format!("/io/github/jgreitemann/mv-mqtt-client/{}", resource_subpath)
}
