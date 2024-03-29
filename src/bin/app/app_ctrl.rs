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
use gtk4::Align;
use itertools::izip;
use mvjson::*;

use super::helpers::*;

pub enum Message {
    StateUpdate(State),
    PreparedRecipeIdsUpdate(Vec<String>),
    RecipeListUpdate(Vec<Recipe>),
    NewResult(VisionResult),
    Error(Error),
}

pub struct ApplicationController {
    prepared_recipe: Option<String>,
    window: Option<adw::ApplicationWindow>,
    g_actions: EnumMap<ActionType, gio::SimpleAction>,
    result_stores: HashMap<String, gtk4::ListStore>,
    alert_store: Option<gtk4::ListStore>,
    clear_alerts_action: gio::SimpleAction,
    alert_stack_page: Option<adw::ViewStackPage>,
    state_machine_pixbufs: EnumMap<State, Option<Pixbuf>>,
    toast_overlay: Option<adw::ToastOverlay>,
    actions_stack: Option<gtk4::Stack>,
    state_machine_image: Option<gtk4::Image>,
    menu_icons: EnumMap<ActionType, Option<gtk4::Image>>,
    recipes_menu: Option<gio::Menu>,
    recipes_stack: Option<gtk4::Stack>,
    results_stack: Option<gtk4::Stack>,
    recipe_input_fields: HashMap<String, Vec<gtk4::Entry>>,
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

        let clear_alerts_action = gio::SimpleAction::new("clear_alerts", None);
        map.add_action(&clear_alerts_action);

        ApplicationController {
            prepared_recipe: None,
            window: None,
            g_actions,
            result_stores: HashMap::new(),
            alert_store: None,
            clear_alerts_action,
            alert_stack_page: None,
            state_machine_pixbufs,
            toast_overlay: None,
            actions_stack: None,
            recipes_menu: None,
            state_machine_image: None,
            menu_icons: enum_map! {_ => None},
            recipes_stack: None,
            results_stack: None,
            recipe_input_fields: HashMap::new(),
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
                        PreparedRecipeIdsUpdate(ids) => ctrl.update_prepared_recipe_ids(ids),
                        RecipeListUpdate(recipe_list) => ctrl.update_recipe_list(recipe_list),
                        NewResult(result) => ctrl.new_result(result),
                        Error(err) => ctrl.error(err),
                    }
                    glib::Continue(true)
                }),
            );
        }));

        for (atype, g_action) in &ctrl.borrow().g_actions {
            let g_action_sender = action_sender.clone();
            g_action.connect_activate(clone!(@weak ctrl => move |_, parameter| {
                g_action_sender
                    .send(match atype {
                        ActionType::SelectModeAutomatic => Action::SelectMode {
                            mode: ModeType::Automatic,
                        },
                        ActionType::PrepareRecipe => Action::PrepareRecipe {
                            recipe_id: parameter.unwrap().str().unwrap().to_string(),
                        },
                        ActionType::UnprepareRecipe => Action::UnprepareRecipe { recipe_id: None },
                        ActionType::StartSingleJob => Action::StartSingleJob {
                            recipe_id: None,
                            parameters: ctrl.borrow().gather_start_parameters(),
                        },
                        ActionType::StartContinuous => Action::StartContinuous {
                            recipe_id: None,
                            parameters: ctrl.borrow().gather_start_parameters(),
                        },
                        ActionType::Reset => Action::Reset,
                        ActionType::Halt => Action::Halt,
                        ActionType::Stop => Action::Stop,
                        ActionType::Abort => Action::Abort,
                    })
                    .unwrap();
            }));
        }

        ctrl.borrow()
            .clear_alerts_action
            .connect_activate(clone!(@weak ctrl => move |_, _| {
                let ctrl = ctrl.borrow();
                ctrl.alert_store.as_ref().map(|store| store.clear());
                ctrl.alert_stack_page.as_ref().map(|page| {
                    page.set_badge_number(0);
                    page.set_needs_attention(false);
                });
            }));
    }

    fn build_ui(&mut self, app: &adw::Application) {
        let builder = gtk4::Builder::from_resource(resource_path("MainWindow.ui").as_str());
        self.window = builder.object("window");
        self.window.as_ref().unwrap().set_application(Some(app));

        self.toast_overlay = builder.object("toast-overlay");
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

        self.alert_store = Some(gtk4::ListStore::new(&[
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::STRING,
            glib::Type::STRING,
        ]));
        let alerts_tree: gtk4::TreeView = builder.object("alerts-tree").unwrap();
        alerts_tree.set_model(self.alert_store.as_ref());

        self.alert_stack_page = builder.object("alert-stack-page");
        let alert_page = self.alert_stack_page.as_ref().unwrap().clone();
        let content_stack: adw::ViewStack = builder.object("content-stack").unwrap();
        content_stack.connect_visible_child_name_notify(move |stack| {
            if stack
                .visible_child_name()
                .as_ref()
                .map(glib::GString::as_str)
                == Some("alert")
            {
                alert_page.set_badge_number(0);
                alert_page.set_needs_attention(false);
            }
        });

        // This is a hack: calling unfullscreen causes window to honor default size.
        self.window.as_ref().map(|win| {
            win.unfullscreen();
            win.present();
        });
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

    pub fn update_prepared_recipe_ids(&mut self, ids: Vec<String>) {
        self.prepared_recipe = ids.first().cloned();
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
            fill_param_rows(
                &input_param_list,
                recipe.inputs.iter(),
                Some(
                    &mut self
                        .recipe_input_fields
                        .entry(recipe.id.clone())
                        .or_insert(Vec::new()),
                ),
            );
            fill_param_rows(&output_param_list, recipe.outputs.iter(), None);

            let input_param_switch: gtk4::Switch =
                recipe_builder.object("input-param-switch").unwrap();
            let recipe_id = recipe.id.clone();
            input_param_switch.connect_state_set(
                clone!(@strong self.recipe_input_fields as fields,
                        @strong self.window as window => move |_, state| {
                    let entries = &fields[&recipe_id];
                    for entry in entries {
                        entry.set_css_classes(if state { &[]} else {&["flat"]});
                        entry.set_can_focus(state);
                        entry.set_focusable(state);
                        entry.set_editable(state);
                        entry.set_text("");
                    }

                    let first_editable = if state { entries.first() } else { None };
                    window.as_ref().map(|win| win.set_focus(first_editable));
                    first_editable.as_ref().map(|e| e.grab_focus());

                    gtk4::Inhibit(false)
                }),
            );

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

    pub fn error(&self, err: Error) {
        let toast = adw::Toast::builder()
            .title(&format!("{}", err.brief))
            .priority(adw::ToastPriority::Normal)
            .build();
        self.toast_overlay
            .as_ref()
            .map(|overlay| overlay.add_toast(&toast));

        self.alert_store.as_ref().map(|store| {
            let severity_icon = match err.severity {
                Severity::Warning => "dialog-warning-symbolic",
                Severity::Error => "edit-delete-symbolic",
                Severity::Critical => "dialog-error-symbolic",
                _ => "",
            };

            let cause_str = err
                .cause
                .map(|cause| format!("{:?}", cause))
                .unwrap_or(String::new());

            store.insert_with_values(
                None,
                &[
                    (0, &severity_icon),
                    (1, &format!("0x{:04x}", &err.code)),
                    (2, &cause_str),
                    (3, &err.brief),
                    (4, &err.message),
                ],
            )
        });

        match err.severity {
            Severity::Warning | Severity::Error | Severity::Critical => {
                self.increment_unread_alert_count()
            }
            _ => {}
        };
    }

    fn increment_unread_alert_count(&self) {
        self.alert_stack_page.as_ref().map(|page| {
            let new_count = page.badge_number() + 1;
            page.set_badge_number(new_count);
            page.set_needs_attention(true);
        });
    }

    fn gather_start_parameters(&self) -> Option<Vec<String>> {
        self.prepared_recipe
            .as_ref()
            .and_then(|id| self.recipe_input_fields.get(id))
            .and_then(|inputs| {
                if inputs.iter().all(|field| field.is_editable()) {
                    Some(
                        inputs
                            .iter()
                            .map(|field| field.text().to_string())
                            .collect(),
                    )
                } else {
                    None
                }
            })
    }
}

fn fill_param_rows<'a, T>(
    group: &adw::PreferencesGroup,
    param_list: T,
    mut fields: Option<&mut Vec<gtk4::Entry>>,
) where
    T: Iterator<Item = &'a RecipeParam>,
{
    let mut used = false;
    for param in param_list {
        used = true;
        let row = adw::ActionRow::builder()
            .selectable(false)
            .focusable(false)
            .title(&param.name)
            .subtitle(&param.description)
            .build();
        let entry = gtk4::Entry::builder()
            .valign(Align::Center)
            .placeholder_text(&*format!("{:?}", param.data_type))
            .editable(false)
            .can_focus(false)
            .css_classes(vec!["flat".to_string()])
            .xalign(1f32)
            .build();
        row.add_suffix(&entry);
        group.add(&row);

        fields.as_mut().map(|entries| entries.push(entry));
    }
    AsRef::<gtk4::Widget>::as_ref(group).set_visible(used);
}

fn resource_path(resource_subpath: &str) -> String {
    format!("/io/github/jgreitemann/mv-mqtt-client/{}", resource_subpath)
}
