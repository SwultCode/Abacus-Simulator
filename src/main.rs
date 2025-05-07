use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::winit::{WinitSettings, UpdateMode};
use std::time::Duration;

use abacus::*;

mod abacus;

#[derive(Resource)]
struct AbacusSettings {
    column_count: usize,
    top_bead_count: usize,
    bottom_bead_count: usize,
    show_top_text: bool,
    show_column_texts: bool,
}

impl Default for AbacusSettings {
    fn default() -> Self {
        Self {
            column_count: 9,
            top_bead_count: 2,
            bottom_bead_count: 5,
            show_top_text: true,
            show_column_texts: true,
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
        .add_systems(Startup, setup)
        .add_systems(Update, 
            (
                move_all_abacus_beads,
                animate_beads,
                update_text_visibility,
                ui_system,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<AbacusSettings>,
) {
    // Anchor entity — controls transform & projection
    commands.spawn((
        MainCameraAnchor,
        Projection::from(PerspectiveProjection::default()),
        Transform::from_xyz(0.0, 5., -14.0).looking_at(Vec3::new(0., 3., 0.), Vec3::Y),
        Visibility::Inherited,
        InheritedVisibility::default(),
        children![
            (
                Camera2d,
                Projection::from(PerspectiveProjection::default()),
                Camera { order: 1, ..default() },
                Visibility::Inherited,
                InheritedVisibility::default(),
            ),
            (
                Camera3d::default(),
                Camera { order: 0, ..default() },
                Projection::from(PerspectiveProjection::default()),
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
        &mut materials,
        settings.column_count,
        settings.top_bead_count,
        settings.bottom_bead_count
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
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    abacus_query: Query<Entity, With<Abacus>>,
) {
    let ctx = contexts.ctx_mut();
    
    egui::Window::new("Abacus Settings")
        .default_pos([10.0, 10.0])
        .show(ctx, |ui| {
            ui.heading("Abacus Configuration");
            
            ui.add_space(10.0);
            ui.label("Structure:");
            ui.add(egui::Slider::new(&mut settings.column_count, 1..=20).text("Columns"));
            ui.add(egui::Slider::new(&mut settings.top_bead_count, 0..=2).text("Top Beads"));
            ui.add(egui::Slider::new(&mut settings.bottom_bead_count, 1..=10).text("Bottom Beads"));
            
            ui.add_space(10.0);
            ui.label("Display Options:");
            ui.checkbox(&mut settings.show_top_text, "Show Total Value");
            ui.checkbox(&mut settings.show_column_texts, "Show Column Values");
            
            ui.add_space(15.0);
            if ui.button("Rebuild Abacus").clicked() {
                info!("Rebuilding abacus");
                // Delete existing abacus
                for entity in &abacus_query {
                    commands.entity(entity).despawn();
                }
                
                // Spawn new abacus with current settings
                abacus::spawn_abacus(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    settings.column_count,
                    settings.top_bead_count,
                    settings.bottom_bead_count,
                );
            }
        });
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