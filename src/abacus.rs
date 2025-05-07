use bevy::prelude::*;
use std::f32::consts::PI;
use bevy::color::palettes::tailwind;

#[derive(Event)]
pub struct AbacusChanged;

pub const BEAD_HEIGHT: f32 = 0.4;
pub const BEAD_SPACING: f32 = 0.5;
pub const LONG_SPACING: f32 = 0.8;
pub const COLUMN_SPACING: f32 = 1.1;
pub const ROW_SPACING: f32 = 0.4;
//pub const BEAD_COUNT: usize = 5;
pub const FRAME_THICKNESS: f32 = 0.1;

pub const BEAD_NORMAL_COLOR: Srgba = tailwind::RED_600;
pub const BEAD_HOVER_COLOR: Srgba = tailwind::RED_200;

pub const FRAME_COLOR: Srgba = tailwind::ZINC_700;

#[derive(Component)]
#[relationship(relationship_target = BeadsOf)]
pub struct BelongsTo(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = BelongsTo)]
pub struct BeadsOf(Vec<Entity>);

#[derive(Component)]
#[require(Transform)]
pub struct AbacusBead {
    pub value: u64,
    pub target: Vec3,
}

pub fn spawn_abacus_bead (
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    value: u64,
) -> Entity {
    let norm_material = materials.add(Color::from(BEAD_NORMAL_COLOR));
    let hover_material = materials.add(Color::from(BEAD_HOVER_COLOR));

    commands.spawn(
        (AbacusBead {
            value: value,
            target: Vec3::new(0.0, 0.0, 0.0),
        },
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            Mesh3d(meshes.add(Extrusion::new(Circle::default(), BEAD_HEIGHT))),
            MeshMaterial3d(norm_material.clone())
        )
    )
    //.observe(hover_bead::<Pointer<Over>>())
    .observe(update_long_value::<Pointer<Click>>())
    .observe(update_material_on::<Pointer<Over>>(hover_material.clone()))
    .observe(update_material_on::<Pointer<Out>>(norm_material.clone()))
    .id()
}

fn hover_bead<E>() -> impl Fn(Trigger<E>, Query<&mut AbacusBead>) {
    move |trigger, mut query| {
        if let Ok(bead) = query.get_mut(trigger.target()) {
            info!("{}", bead.value);
        }
    }
}

fn update_material_on<E>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |trigger, mut query| {
        if let Ok(mut material) = query.get_mut(trigger.target()) {
            material.0 = new_material.clone();
        }
    }
}

fn update_long_value<E>() -> impl Fn(Trigger<E>, Query<(&AbacusBead, &BelongsTo)>, Query<&mut AbacusLong>, Commands) {
    move |trigger, beads, mut longs, mut commands| {
        if let Ok((bead, BelongsTo(long))) = beads.get(trigger.target()) {
            if let Ok(mut abacus_long) = longs.get_mut(*long) {
                if abacus_long.value != bead.value {
                    abacus_long.value = bead.value;
                } else {
                    abacus_long.value = bead.value.saturating_sub(1);
                }

                commands.send_event(AbacusChanged);
                info!("Abacus Long Value Now {}", abacus_long.value);
            }
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct AbacusLong {
    pub value: u64,
}

pub fn spawn_abacus_long(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    bead_count: usize,
) -> Entity {
    let mut beads = Vec::new();

    let abacus_long_height = bead_count as f32 * BEAD_SPACING + LONG_SPACING + FRAME_THICKNESS*2.0;
    let abacus_long_width = FRAME_THICKNESS;
    
    // Spawn the abacus long with the beads
    let abacus_long = commands.spawn((
        AbacusLong {
            value: 0,
        },
        InheritedVisibility::default(),
        children![
            (
                Mesh3d(meshes.add(Extrusion::new(Circle::new(abacus_long_width), abacus_long_height))),
                MeshMaterial3d(materials.add(Color::from(FRAME_COLOR))),
                Transform::from_xyz(0.0, abacus_long_height/2.0 - BEAD_SPACING/2.0 - FRAME_THICKNESS, 0.0).with_rotation(Quat::from_rotation_x(PI / 2.0)),
                Pickable::IGNORE,
            )
        ],
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();

    // Spawn 5 beads and collect their entity IDs
    for i in 0..bead_count {
        let new_bead = spawn_abacus_bead(commands, meshes, materials, i as u64 + 1);
        commands.entity(new_bead).insert(BelongsTo(abacus_long));
        commands.entity(new_bead).insert(ChildOf(abacus_long));
        beads.push(new_bead);
    }

    abacus_long
}

#[derive(Component)]
#[require(Transform)]
pub struct Abacus {
    pub top_longs: Vec<Entity>,
    pub bottom_longs: Vec<Entity>,
    pub column_texts: Vec<Entity>,
    pub total_text: Entity,
    top_bead_count: usize,
    bottom_bead_count: usize,
    pub total_value: u64,
}

impl Abacus {
    pub fn get_column_value(
        &self,
        column_index: usize,
        abacus_long_query: &Query<&AbacusLong>,
    ) -> u64 {
        let top_long_entity = self.top_longs[column_index];
        let bottom_long_entity = self.bottom_longs[column_index];

        let top_long_value = abacus_long_query.get(top_long_entity).expect("Top AbacusLong entity not found or missing AbacusLong component").value;
        let bottom_long_value = abacus_long_query.get(bottom_long_entity).expect("Bottom AbacusLong entity not found or missing AbacusLong component").value;
        
        // if top % 2 == 0, then value is bottom, otherwise it's bottom + bottom_bead_count
        (self.bottom_bead_count as u64 - bottom_long_value) + (top_long_value % 2) * self.bottom_bead_count as u64
    }

    pub fn get_total_value(
        &mut self,
        abacus_long_query: &Query<&AbacusLong>,
    ) -> u64 {
        let mut total_value = 0;

        // total will be base (bottom_bead_count * 2)
        for i in 0..self.top_longs.len() {
            total_value += self.get_column_value(i, abacus_long_query) * (self.bottom_bead_count as u64 * 2).pow(i as u32);
        }
        self.total_value = total_value;
        total_value
    }
}
        

pub fn spawn_abacus(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    column_count: usize,
    top_bead_count: usize,
    bottom_bead_count: usize,
) {
    let mut top_longs_temp = Vec::new();
    let mut bottom_longs_temp = Vec::new();
    let mut column_texts = Vec::new();
    //let font = asset_server.load("fonts/FiraMono-Medium.ttf"); // Make sure this font exists in your assets
    let text_font = TextFont {
        // font: font.clone(),
        font_size: 64.0,
        ..default()
    };

    let scale = Vec3::new(-0.01, 0.01, 0.01);

    let top_long_y = (bottom_bead_count as f32) * BEAD_SPACING + LONG_SPACING + ROW_SPACING;

    for i in 0..column_count {
        let top_long = spawn_abacus_long(commands, meshes, materials, top_bead_count);
        let bottom_long = spawn_abacus_long(commands, meshes, materials, bottom_bead_count);
        
        let x = (i as f32 - ((column_count as f32 - 1.0) / 2.0)) * COLUMN_SPACING;
        
        commands.entity(top_long).insert(Transform {
            translation: Vec3::new(x, top_long_y, 0.0),
            ..default()
        });

        commands.entity(bottom_long).insert(Transform {
            translation: Vec3::new(x, 0.0, 0.0),
            ..default()
        });

        top_longs_temp.push(top_long);
        bottom_longs_temp.push(bottom_long);

        // Position for the text (below the bottom long)
        let y = -0.7; // Adjust as needed to be below the abacus

        let text_entity = commands.spawn((
            Text2d::new("0"),
            text_font.clone(),
            Transform::from_xyz(x, y, 0.0).with_scale(scale.clone()),
        )).id();
        column_texts.push(text_entity);
    }

    // Spawn total value text above the abacus
    let total_text_entity = commands.spawn((
        Text2d::new("0"),
        text_font.clone(),
        Transform::from_xyz(0.0, top_long_y + 2.0, 0.0).with_scale(scale.clone()),
    )).id();


    // Spawn the Abacus component with clones of the entity lists
    let abacus_id = commands.spawn(Abacus {
        top_longs: top_longs_temp.clone(),
        bottom_longs: bottom_longs_temp.clone(),
        column_texts: column_texts.clone(),
        total_text: total_text_entity,
        top_bead_count: top_bead_count,
        bottom_bead_count: bottom_bead_count,
        total_value: 0,
    }).id();

    // Set up Bevy's standard parent-child hierarchy
    for &top_long_entity in &top_longs_temp {
        commands.entity(abacus_id).add_child(top_long_entity);
    }

    for &bottom_long_entity in &bottom_longs_temp {
        commands.entity(abacus_id).add_child(bottom_long_entity);
    }
}