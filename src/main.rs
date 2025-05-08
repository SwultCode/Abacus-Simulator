use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::winit::{WinitSettings, UpdateMode};
use bevy::input::mouse::MouseMotion;
use std::time::Duration;

use abacus::*;

mod abacus;

// Configuration that can be saved/loaded
#[derive(Clone, Debug, PartialEq)] // PartialEq for potential future comparisons
struct SavableAbacusConfig {
    name: String, // Name will be part of this struct for simplicity here
    column_count: usize,
    top_bead_count: usize,
    bottom_bead_count: usize,
    top_bead_base_value: u64,
    abacus_base: u64,
    show_top_text: bool,
    show_column_texts: bool,
    ui_bead_color: Color,
    ui_bead_hover_color: Color,
    ui_frame_color: Color,
}

// Resource to hold all user-saved configurations and UI state for saving/loading
#[derive(Resource, Debug)] // Removed Default, will use FromWorld
struct UserConfigurations {
    configs: Vec<SavableAbacusConfig>,
    new_config_name: String, 
    selected_config_name_to_load: String, 
    set_value_input: String,
    modify_value_input: String, // New field for Add/Subtract input
}

impl FromWorld for UserConfigurations {
    fn from_world(_world: &mut World) -> Self {
        // Pre-populate with some default configurations
        let default_configs = vec![
            SavableAbacusConfig {
                name: "Suanpan (Chinese 2/5)".to_string(),
                column_count: 9,
                top_bead_count: 2, // 2 beads in the upper deck
                bottom_bead_count: 5, // 5 beads in the lower deck
                top_bead_base_value: 5, // Each upper bead is worth 5 (when moved against the bar)
                abacus_base: 10, // Typically used for decimal calculations
                show_top_text: true,
                show_column_texts: true,
                // Placeholder colors - you can refine these to match typical abacus colors
                ui_bead_color: Color::srgb(0.6, 0.3, 0.1), // Brownish beads
                ui_bead_hover_color: Color::srgb(0.7, 0.4, 0.2),
                ui_frame_color: Color::srgb(0.3, 0.2, 0.1), // Dark wood frame
            },
            SavableAbacusConfig {
                name: "Soroban (Japanese 1/4)".to_string(),
                column_count: 13, // Sorobans often have more columns
                top_bead_count: 1,   // 1 bead in the upper deck
                bottom_bead_count: 4, // 4 beads in the lower deck
                top_bead_base_value: 5, // Upper bead is worth 5
                abacus_base: 10, // Decimal system
                show_top_text: true,
                show_column_texts: true,
                ui_bead_color: Color::srgb(0.2, 0.2, 0.2), // Dark beads
                ui_bead_hover_color: Color::srgb(0.4, 0.4, 0.4),
                ui_frame_color: Color::srgb(0.5, 0.5, 0.5), // Lighter frame
            },
            SavableAbacusConfig {
                name: "Binary Counter (1/1)".to_string(),
                column_count: 8,
                top_bead_count: 0,
                bottom_bead_count: 1,
                top_bead_base_value: 1,
                abacus_base: 2,
                show_top_text: true,
                show_column_texts: true,
                ui_bead_color: Color::srgb(0.1, 0.5, 0.1), // Green beads
                ui_bead_hover_color: Color::srgb(0.2, 0.7, 0.2),
                ui_frame_color: Color::srgb(0.4, 0.4, 0.4), 
            },
            // Add more predefined configurations as needed
        ];

        // Set the first config as initially selected if available
        let initial_selection = if !default_configs.is_empty() {
            default_configs[0].name.clone()
        } else {
            String::new()
        };

        Self {
            configs: default_configs,
            new_config_name: String::new(),
            selected_config_name_to_load: initial_selection,
            set_value_input: String::new(),
            modify_value_input: String::new(), // Initialize
        }
    }
}

#[derive(Resource)]
struct AbacusSettings {
    column_count: usize,
    top_bead_count: usize,
    bottom_bead_count: usize,
    top_bead_base_value: u64,
    abacus_base: u64,
    show_top_text: bool,
    show_column_texts: bool,

    // Handles to shared materials
    bead_material: Handle<StandardMaterial>,
    bead_hover_material: Handle<StandardMaterial>, // Will be used if hover effects are re-enabled for non-mobile
    frame_material: Handle<StandardMaterial>,

    // Colors for UI pickers
    ui_bead_color: Color,
    ui_bead_hover_color: Color,
    ui_frame_color: Color,
}

impl FromWorld for AbacusSettings {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<StandardMaterial>>().unwrap();

        let initial_bead_color = Color::from(abacus::BEAD_NORMAL_COLOR);
        let initial_bead_hover_color = Color::from(abacus::BEAD_HOVER_COLOR);
        let initial_frame_color = Color::from(abacus::FRAME_COLOR);

        let bead_material = materials.add(StandardMaterial {
            base_color: initial_bead_color,
            ..default()
        });
        let bead_hover_material = materials.add(StandardMaterial {
            base_color: initial_bead_hover_color,
            ..default()
        });
        let frame_material = materials.add(StandardMaterial {
            base_color: initial_frame_color,
            ..default()
        });

        Self {
            column_count: 9,
            top_bead_count: 2,
            bottom_bead_count: 5,
            top_bead_base_value: 5,
            abacus_base: 10,
            show_top_text: true,
            show_column_texts: true,
            bead_material,
            bead_hover_material,
            frame_material,
            ui_bead_color: initial_bead_color,
            ui_bead_hover_color: initial_bead_hover_color,
            ui_frame_color: initial_frame_color,
        }
    }
}

// Helper to create a SavableAbacusConfig from current AbacusSettings
impl SavableAbacusConfig {
    fn from_settings(name: String, settings: &AbacusSettings) -> Self {
        Self {
            name,
            column_count: settings.column_count,
            top_bead_count: settings.top_bead_count,
            bottom_bead_count: settings.bottom_bead_count,
            top_bead_base_value: settings.top_bead_base_value,
            abacus_base: settings.abacus_base,
            show_top_text: settings.show_top_text,
            show_column_texts: settings.show_column_texts,
            ui_bead_color: settings.ui_bead_color,
            ui_bead_hover_color: settings.ui_bead_hover_color,
            ui_frame_color: settings.ui_frame_color,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Make it resize to the available space
                fit_canvas_to_parent: true,
                // Prevents issues with touch scrolling and back/forward gestures
                prevent_default_event_handling: true,
                // Don't allow resizing (can crash on some mobile browsers if left true)
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((MeshPickingPlugin, EguiPlugin { enable_multipass_for_primary_context: false }))
        .add_event::<AbacusChanged>()
        .init_resource::<AbacusSettings>()
        .init_resource::<UserConfigurations>()
        .add_systems(Startup, setup)
        .add_systems(Update, 
            (
                move_all_abacus_beads,
                animate_beads,
                update_text_visibility,
                ui_system,
                abacus_rotation_system,
            )
        )
        .add_systems(Update, 
        (
                update_abacus_values,
                update_abacus_texts
            ).run_if(on_event::<AbacusChanged>),
        )
        .add_systems(Startup, init_refresh_rate)
        .run();
}

fn init_refresh_rate(mut winit: ResMut<WinitSettings>) {
    winit.focused_mode = UpdateMode::reactive(Duration::from_secs_f32(1.0 / 60.0));
}

#[derive(Component)]
#[require(Transform)]
pub struct MainCameraAnchor;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    settings: Res<AbacusSettings>,
) {
    // Anchor entity — controls transform & projection
    commands.spawn((
        MainCameraAnchor,
        Projection::from(PerspectiveProjection::default()),
        Transform::from_xyz(0.0, 5., -14.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        Visibility::Inherited,
        InheritedVisibility::default(),
        children![
            (
                Camera3d::default(),
                Camera { order: 0, ..default() },
                Projection::from(PerspectiveProjection::default()),
                Visibility::Inherited,
                InheritedVisibility::default(),
            ),
            (
                Camera2d,
                Projection::from(PerspectiveProjection::default()),
                Camera { order: 1, ..default() },
                Visibility::Inherited,
                InheritedVisibility::default(),
            )
        ]
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, -8.0),
        Visibility::Inherited,
        InheritedVisibility::default(),
    ));
    
    abacus::spawn_abacus(
        &mut commands,
        &mut meshes,
        &settings,
    );
}

fn move_all_abacus_beads(
    query: Query<(&BeadsOf, &AbacusLong)>,
    mut beads: Query<&mut AbacusBead>,
) {
    for (beads_of, long) in &query {
        let upper_count = long.value as usize;

        let mut y = 0.0;

        for &bead in &beads_of[..upper_count] {
            if let Ok(mut bead) = beads.get_mut(bead) {
                bead.target = Vec3::new(0.0, y, 0.0);
                y += BEAD_SPACING;
            }
        }

        y += LONG_SPACING;

        for &bead in &beads_of[upper_count..] {
            if let Ok(mut bead) = beads.get_mut(bead) {
                bead.target = Vec3::new(0.0, y, 0.0);
                y += BEAD_SPACING;
            }
        }
    }
}

fn animate_beads(
    mut query: Query<(&mut Transform, &AbacusBead)>,
    time: Res<Time>,
) {
    let speed = 10.0; // units per second, adjust as needed
    for (mut transform, bead) in &mut query {
        let current = transform.translation;
        let target = bead.target;
        if current != target {
            let direction = target - current;
            let distance = direction.length();
            let step = speed * time.delta_secs();
            if distance <= step {
                transform.translation = target;
            } else {
                transform.translation += direction.normalize() * step;
            }
        }
    }
}

fn update_abacus_values(
    mut abacus_query: Query<&mut Abacus>,
    abacus_long_query: Query<&AbacusLong>,
) {
    for mut abacus in &mut abacus_query {
        let _value = abacus.get_total_value(&abacus_long_query);
    }
}

fn update_abacus_texts(
    abacus_query: Query<&Abacus>,
    abacus_long_query: Query<&AbacusLong>,
    mut text_query: Query<&mut Text2d>,
) {
    for abacus in &abacus_query {
        // Update total value text
        if let Ok(mut text) = text_query.get_mut(abacus.total_text) {
            text.0 = abacus.total_value.to_string();
        }
        // Update each column's value text
        for (i, &text_entity) in abacus.column_texts.iter().enumerate() {
            let col_value = abacus.get_column_value(i, &abacus_long_query);
            if let Ok(mut text) = text_query.get_mut(text_entity) {
                text.0 = col_value.to_string();
            }
        }
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<AbacusSettings>,
    mut user_configs: ResMut<UserConfigurations>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut abacus_query: Query<&mut Abacus>,
    mut long_query: Query<&mut AbacusLong>,
    abacus_entity_query: Query<Entity, With<Abacus>>,
    mut abacus_transform_query: Query<&mut Transform, With<Abacus>>,
) {
    let ctx = contexts.ctx_mut();
    
    let mut rebuild_abacus_requested = false;
    
    egui::Window::new("Abacus Settings")
        .default_pos([10.0, 10.0])
        .show(ctx, |ui| {
            ui.heading("Abacus Configuration");
            
            // --- Structure Section --- 
            ui.collapsing("Structure", |ui| {
                if ui.add(egui::Slider::new(&mut settings.column_count, 1..=20).text("Columns")).changed() { rebuild_abacus_requested = true; };
                if ui.add(egui::Slider::new(&mut settings.top_bead_count, 0..=2).text("Top Beads (per section)")).changed() { rebuild_abacus_requested = true; };
                if ui.add(egui::Slider::new(&mut settings.bottom_bead_count, 1..=10).text("Bottom Beads (per section)")).changed() { rebuild_abacus_requested = true; };
                if ui.add(egui::Slider::new(&mut settings.top_bead_base_value, 1..=10).text("Top Bead Base Value")).changed() { rebuild_abacus_requested = true; };
                if ui.add(egui::Slider::new(&mut settings.abacus_base, 2..=36).text("Abacus Numeric Base")).changed() { rebuild_abacus_requested = true; };
            });

            // --- Display Options Section --- 
            ui.collapsing("Display Options", |ui| {
            ui.checkbox(&mut settings.show_top_text, "Show Total Value");
            ui.checkbox(&mut settings.show_column_texts, "Show Column Values");
            });

            // --- Appearance Section --- 
            ui.collapsing("Appearance (Live Update)", |ui| {
                // Directly use .as_rgba() which returns an Srgba, then access fields
                let (mut r_b, mut g_b, mut b_b, mut a_b) = (0.0, 0.0, 0.0, 0.0); // bead_color
                if let Color::Srgba(srgba) = settings.ui_bead_color {
                    r_b = srgba.red;
                    g_b = srgba.green;
                    b_b = srgba.blue;
                    a_b = srgba.alpha;
                }
                let mut bead_color_arr = [r_b, g_b, b_b, a_b];

                let (mut r_bh, mut g_bh, mut b_bh, mut a_bh) = (0.0, 0.0, 0.0, 0.0); // bead_hover_color
                if let Color::Srgba(srgba) = settings.ui_bead_hover_color {
                    r_bh = srgba.red;
                    g_bh = srgba.green;
                    b_bh = srgba.blue;
                    a_bh = srgba.alpha;
                }
                let mut bead_hover_color_arr = [r_bh, g_bh, b_bh, a_bh];

                let (mut r_f, mut g_f, mut b_f, mut a_f) = (0.0, 0.0, 0.0, 0.0); // frame_color
                if let Color::Srgba(srgba) = settings.ui_frame_color {
                    r_f = srgba.red;
                    g_f = srgba.green;
                    b_f = srgba.blue;
                    a_f = srgba.alpha;
                }
                let mut frame_color_arr = [r_f, g_f, b_f, a_f];
                
                ui.horizontal(|ui| {
                    if ui.color_edit_button_rgba_unmultiplied(&mut bead_color_arr).changed() {
                        settings.ui_bead_color = Color::Srgba(bevy::color::Srgba::new(bead_color_arr[0], bead_color_arr[1], bead_color_arr[2], bead_color_arr[3]));
                        if let Some(material) = standard_materials.get_mut(&settings.bead_material) {
                            material.base_color = settings.ui_bead_color;
                        }
                    }
                    ui.label("Bead Color");
                });
                ui.horizontal(|ui| {
                    if ui.color_edit_button_rgba_unmultiplied(&mut bead_hover_color_arr).changed() {
                        settings.ui_bead_hover_color = Color::Srgba(bevy::color::Srgba::new(bead_hover_color_arr[0], bead_hover_color_arr[1], bead_hover_color_arr[2], bead_hover_color_arr[3]));
                        if let Some(material) = standard_materials.get_mut(&settings.bead_hover_material) {
                            material.base_color = settings.ui_bead_hover_color;
                        }
                    }
                    ui.label("Bead Hover (non-mobile)");
                });
                ui.horizontal(|ui| {
                    if ui.color_edit_button_rgba_unmultiplied(&mut frame_color_arr).changed() {
                        settings.ui_frame_color = Color::Srgba(bevy::color::Srgba::new(frame_color_arr[0], frame_color_arr[1], frame_color_arr[2], frame_color_arr[3]));
                        if let Some(material) = standard_materials.get_mut(&settings.frame_material) {
                            material.base_color = settings.ui_frame_color;
                        }
                    }
                    ui.label("Frame Color");
                });
            });

            // --- Controls Section --- 
            ui.collapsing("Controls", |ui| {
                // Reset Rotation Button
                if ui.button("Reset Rotation").clicked() {
                    if let Ok(mut transform) = abacus_transform_query.single_mut() {
                        transform.rotation = Quat::IDENTITY;
                    }
                }
                
                ui.separator();
                
                // Set Value Input and Button
                ui.label("Set Abacus Value:");
                ui.horizontal(|ui| {
                    let set_response = ui.add_sized([100.0, ui.available_height()], 
                        egui::TextEdit::singleline(&mut user_configs.set_value_input)
                            .hint_text("Enter value")
                    );
                    let set_submitted = set_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.button("Set").clicked() || set_submitted {
                        match user_configs.set_value_input.trim().parse::<u64>() {
                            Ok(value) => {
                                if let Ok(mut abacus) = abacus_query.single_mut() {
                                    info!("Setting abacus total value to: {}", value);
                                    abacus.set_total_value(value, &mut long_query, &mut commands);
                                }
                            }
                            Err(_) => { info!("Invalid input for Set: Please enter a non-negative integer."); }
                        }
                    }
                });

                ui.separator();
                
                // Add/Subtract Value Input and Buttons
                ui.label("Modify Abacus Value:");
                ui.horizontal(|ui| {
                    let modify_response = ui.add_sized([100.0, ui.available_height()], 
                        egui::TextEdit::singleline(&mut user_configs.modify_value_input)
                            .hint_text("Enter amount")
                    );
                    let modify_submitted_add = modify_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)); // Treat Enter as Add
                    
                    let add_clicked = ui.button("Add").clicked() || modify_submitted_add;
                    let subtract_clicked = ui.button("Subtract").clicked();

                    if add_clicked || subtract_clicked {
                        match user_configs.modify_value_input.trim().parse::<u64>() {
                            Ok(amount) => {
                                if let Ok(mut abacus) = abacus_query.single_mut() {
                                    let current_value = abacus.total_value;
                                    let new_value = if add_clicked {
                                        current_value.saturating_add(amount)
                                    } else { // subtract_clicked must be true
                                        current_value.saturating_sub(amount)
                                    };
                                    
                                    info!("Setting abacus total value to: {} (from {} {} {})", 
                                        new_value, current_value, if add_clicked {"+"} else {"-"}, amount);
                                    abacus.set_total_value(new_value, &mut long_query, &mut commands);
                                } else {
                                    warn!("Could not find Abacus component to modify value.");
                                }
                                // Optionally clear input after modifying
                                // user_configs.modify_value_input.clear();
                            }
                            Err(_) => { info!("Invalid input for Modify: Please enter a non-negative integer."); }
                        }
                    }
                });
            });

            // --- Save/Load Configurations Section --- 
            ui.collapsing("Save/Load Configurations", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Config Name:");
                    ui.text_edit_singleline(&mut user_configs.new_config_name);
                });
                if ui.button("Save Current Configuration").clicked() {
                    let name_to_save = user_configs.new_config_name.trim().to_string(); // Clone and trim here
                    if !name_to_save.is_empty() {
                        // Prevent duplicates by name, or update existing
                        if let Some(existing_idx) = user_configs.configs.iter().position(|c| c.name == name_to_save) {
                            user_configs.configs[existing_idx] = SavableAbacusConfig::from_settings(name_to_save, &settings);
                        } else {
                            user_configs.configs.push(SavableAbacusConfig::from_settings(name_to_save, &settings));
                        }
                        user_configs.new_config_name.clear(); // Clear the original mutable field
                        info!("Configuration saved.");
                    } else {
                        info!("Please enter a name to save the configuration.");
                    }
                }

                ui.separator();
                
                let mut newly_selected_name: Option<String> = None;
                
                egui::ComboBox::new("load_config_combobox_unique_id", "") 
                    .selected_text(user_configs.selected_config_name_to_load.as_str())
                    .show_ui(ui, |ui| {
                        for conf in user_configs.configs.iter() { // Immutable borrow for iteration
                            // selectable_value internally compares conf.name with the current selected_config_name_to_load
                            // and updates its internal state. We capture if it was clicked.
                            if ui.selectable_label(user_configs.selected_config_name_to_load == conf.name, &conf.name).clicked() {
                                newly_selected_name = Some(conf.name.clone());
                            }
                        }
                    });
                
                // Apply the selection change after the ComboBox UI is built
                if let Some(name) = newly_selected_name {
                    user_configs.selected_config_name_to_load = name;
                }

                // Ensure selected_config_name_to_load is valid or defaults to first if possible
                if !user_configs.configs.is_empty() && 
                   user_configs.configs.iter().find(|c| c.name == user_configs.selected_config_name_to_load).is_none() {
                    user_configs.selected_config_name_to_load = user_configs.configs[0].name.clone();
                }

                if ui.button("Load Selected Configuration").clicked() {
                    let name_to_load = user_configs.selected_config_name_to_load.clone();
                    if !name_to_load.is_empty() {
                        if let Some(loaded_config) = user_configs.configs.iter().find(|c| c.name == name_to_load).cloned() { // Clone the config to avoid borrow issues
                            // Use the helper function
                            apply_config(&mut settings, &mut standard_materials, &loaded_config);
                            
                            rebuild_abacus_requested = true;
                            info!("Configuration '{}' loaded.", loaded_config.name);
                        } else {
                            info!("Selected configuration '{}' not found to load.", name_to_load);    
                        }
                    } else if !user_configs.configs.is_empty() {
                        // Attempt to load the first one
                        let first_config = user_configs.configs[0].clone(); // Clone here too
                        apply_config(&mut settings, &mut standard_materials, &first_config);
                        rebuild_abacus_requested = true;
                        info!("Loaded first available configuration '{}'.", first_config.name);
                    } else {
                        info!("No configuration selected or available to load.");
                    }
                }
                // Optional: Delete button
                if ui.button("Delete Selected Configuration").clicked() {
                    let name_to_delete = user_configs.selected_config_name_to_load.clone();
                    if !name_to_delete.is_empty() {
                        if let Some(pos) = user_configs.configs.iter().position(|c| c.name == name_to_delete) {
                            user_configs.configs.remove(pos);
                            user_configs.selected_config_name_to_load.clear(); // Clear selection after delete
                            info!("Configuration '{}' deleted.", name_to_delete);
                        } else {
                             info!("Configuration '{}' not found to delete.", name_to_delete);
                        }
                    } else {
                        info!("No configuration selected to delete.");
                    }
                }
            });
            
            // --- Rebuild Button --- 
            // ui.add_space(15.0);
            // if ui.button("Rebuild Abacus (Apply Structure Changes)").clicked() {
            //     rebuild_abacus_requested = true;
            // }
        });

    if rebuild_abacus_requested {
        info!("Rebuilding abacus structure");
        for entity in abacus_entity_query.iter() {
                    commands.entity(entity).despawn();
                }
                
                abacus::spawn_abacus(
                    &mut commands,
                    &mut meshes,
            &settings, 
                );
            }
}

fn update_text_visibility(
    settings: Res<AbacusSettings>,
    abacus_query: Query<&Abacus>,
    mut visibility_query: Query<&mut Visibility>,
) {
    if !settings.is_changed() {
        return;
    }
    
    for abacus in &abacus_query {
        // Update total text visibility
        if let Ok(mut visibility) = visibility_query.get_mut(abacus.total_text) {
            *visibility = if settings.show_top_text {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        
        // Update column texts visibility
        for &text_entity in &abacus.column_texts {
            if let Ok(mut visibility) = visibility_query.get_mut(text_entity) {
                *visibility = if settings.show_column_texts {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

fn abacus_rotation_system(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Abacus>>,
) {
    // Only process motion when right mouse button is pressed
    if mouse_button.pressed(MouseButton::Right) {
        let mut rotation_delta = Vec2::ZERO;
        
        // Accumulate mouse motion
        for event in mouse_motion_events.read() {
            rotation_delta += event.delta;
        }
        
        // Apply rotation if there was mouse movement
        if rotation_delta.length_squared() > 0.0 {
            // Scale the rotation speed
            let rotation_speed = 0.005;
            
            // Apply horizontal movement to Y-axis rotation (left/right)
            // Apply vertical movement to X-axis rotation (up/down)
            if let Ok(mut transform) = query.single_mut() {
                transform.rotate_y(rotation_delta.x * rotation_speed);
                transform.rotate_x(-rotation_delta.y * rotation_speed);
            }
        }
    } else {
        // Clear any pending events when not rotating
        mouse_motion_events.clear();
    }
}

/// Applies a saved configuration to the active settings and materials.
fn apply_config(
    settings: &mut AbacusSettings,
    materials: &mut Assets<StandardMaterial>,
    config: &SavableAbacusConfig,
) {
    // Apply structural settings
    settings.column_count = config.column_count;
    settings.top_bead_count = config.top_bead_count;
    settings.bottom_bead_count = config.bottom_bead_count;
    settings.top_bead_base_value = config.top_bead_base_value;
    settings.abacus_base = config.abacus_base;
    settings.show_top_text = config.show_top_text;
    settings.show_column_texts = config.show_column_texts;

    // Apply color settings and update materials
    settings.ui_bead_color = config.ui_bead_color;
    if let Some(material) = materials.get_mut(&settings.bead_material) {
        material.base_color = settings.ui_bead_color;
    }
    settings.ui_bead_hover_color = config.ui_bead_hover_color;
    if let Some(material) = materials.get_mut(&settings.bead_hover_material) {
        material.base_color = settings.ui_bead_hover_color;
    }
    settings.ui_frame_color = config.ui_frame_color;
    if let Some(material) = materials.get_mut(&settings.frame_material) {
        material.base_color = settings.ui_frame_color;
    }
}