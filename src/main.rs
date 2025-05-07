use bevy::prelude::*;
use bevy::window::PresentMode;

use abacus::*;

mod abacus;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_event::<AbacusChanged>()
        .add_systems(Startup, setup)
        .add_systems(Update, 
            (
                move_all_abacus_beads,
                animate_beads,
            )
        )
        .add_systems(Update, 
        (
                update_abacus_values,
                update_abacus_texts
            ).run_if(on_event::<AbacusChanged>),
        )
        .run();
}

#[derive(Component)]
#[require(Transform)]
pub struct MainCameraAnchor;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Anchor entity — controls transform & projection
    commands.spawn((
        MainCameraAnchor,
        Projection::from(PerspectiveProjection::default()),
        Transform::from_xyz(0.0, 4., -14.0).looking_at(Vec3::new(0., 2., 0.), Vec3::Y),
        children![
            (
                Camera2d,
                Projection::from(PerspectiveProjection::default()),
                Camera { order: 1, ..default() },
            ),
            (
                Camera3d::default(),
                Camera { order: 0, ..default() },
                Projection::from(PerspectiveProjection::default()),
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
    ));
    
    abacus::spawn_abacus(&mut commands, &mut meshes, &mut materials, 9, 2, 5);
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
        let value = abacus.get_total_value(&abacus_long_query);
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
