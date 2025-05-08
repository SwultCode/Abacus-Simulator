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

#[cfg(not(target_arch = "wasm32"))]
fn is_mobile_device() -> bool {
    false // Default to desktop for non-wasm builds
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    fn is_mobile_device() -> bool;
}

pub fn spawn_abacus_bead (
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    value: u64,
    bead_material_handle: &Handle<StandardMaterial>,
    bead_hover_material_handle: &Handle<StandardMaterial>,
) -> Entity {
    let norm_material = bead_material_handle.clone();
    let hover_material = bead_hover_material_handle.clone();

    let mut entity_builder = commands.spawn(
        (AbacusBead {
            value: value,
            target: Vec3::new(0.0, 0.0, 0.0),
        },
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            Mesh3d(meshes.add(Extrusion::new(Circle::default(), BEAD_HEIGHT))),
            MeshMaterial3d(norm_material),
            Visibility::Inherited,
            InheritedVisibility::default(),
        )
    );
    
    entity_builder.observe(update_long_value::<Pointer<Click>>());
    
    if !is_mobile_device() {
        entity_builder
            .observe(update_material_on::<Pointer<Over>>(hover_material))
            .observe(update_material_on::<Pointer<Out>>(bead_material_handle.clone()));
    }
    
    entity_builder.id()
}

fn update_material_on<E>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
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
                if abacus_long.value + 1 != bead.value {
                    abacus_long.value = bead.value - 1;
                } else {
                    abacus_long.value = bead.value;
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
    bead_count: usize,
    bead_material_handle: &Handle<StandardMaterial>,
    bead_hover_material_handle: &Handle<StandardMaterial>,
    frame_material_handle: &Handle<StandardMaterial>,
    value: u64,
) -> Entity {
    // Spawn the AbacusLong component entity first. It will always exist logically.
    let abacus_long_entity = commands.spawn((
        AbacusLong {
            value: value, // If bead_count is 0, value will be 0.
        },
        InheritedVisibility::default(),
        Visibility::Inherited,
        Transform::from_xyz(0.0, 0.0, 0.0), // Positioned by parent Abacus
    )).id();

    if bead_count > 0 {
        // Only spawn the visual rod and beads if bead_count > 0
        let abacus_long_height = bead_count as f32 * BEAD_SPACING + LONG_SPACING + FRAME_THICKNESS * 2.0;
        let abacus_long_width = FRAME_THICKNESS;

        let rod_mesh_entity = commands.spawn((
            Mesh3d(meshes.add(Extrusion::new(Circle::new(abacus_long_width), abacus_long_height))),
            MeshMaterial3d(frame_material_handle.clone()),
            Transform::from_xyz(0.0, abacus_long_height / 2.0 - BEAD_SPACING / 2.0 - FRAME_THICKNESS, 0.0)
                .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            Pickable::IGNORE,
            Visibility::Inherited,
            InheritedVisibility::default(),
        )).id();
        commands.entity(abacus_long_entity).add_child(rod_mesh_entity);

        let mut beads = Vec::new(); // This vec is local and not stored in AbacusLong, which is fine.
        for i in 0..bead_count {
            let new_bead = spawn_abacus_bead(commands, meshes, i as u64 + 1, bead_material_handle, bead_hover_material_handle);
            commands.entity(new_bead).insert((
                BelongsTo(abacus_long_entity),
                // Beads are children of the AbacusLong entity so they move with it if the AbacusLong's transform is changed relative to Abacus.
                // Their individual Y position is relative to the AbacusLong entity.
                ChildOf(abacus_long_entity), 
                Visibility::Inherited,
                InheritedVisibility::default(),
            ));
            beads.push(new_bead);
        }
    }
    // If bead_count is 0, no rod mesh or beads are spawned for this AbacusLong.

    abacus_long_entity // Return the logical AbacusLong entity ID
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
    top_bead_base_value: u64,
    abacus_base: u64,
    pub total_value: u64,
}

impl Abacus {
    pub fn get_column_value(
        &self,
        column_index: usize,
        abacus_long_query: &Query<&AbacusLong>,
    ) -> u64 {
        if column_index >= self.top_longs.len() {
            return 0; // Index out of bounds
        }
        let top_long_entity = self.top_longs[column_index];
        let bottom_long_entity = self.bottom_longs[column_index];

        // Use get() which returns Option, handle potential errors gracefully
        let top_long_val = match abacus_long_query.get(top_long_entity) {
            Ok(long) => long.value,
            Err(_) => return 0, // Or handle error appropriately
        };
        let bottom_long_val = match abacus_long_query.get(bottom_long_entity) {
            Ok(long) => long.value,
            Err(_) => return 0, // Or handle error appropriately
        };
        
        // Value from bottom beads + (is top active * top bead base value)
        // Check top_bead_count > 0 before using top_bead_base_value
        let top_contribution = if self.top_bead_count > 0 {
             (top_long_val % 2) * self.top_bead_base_value
        } else {
            0
        };
        
        (self.bottom_bead_count as u64 - bottom_long_val) + top_contribution
    }

    pub fn get_total_value(
        &mut self,
        abacus_long_query: &Query<&AbacusLong>,
    ) -> u64 {
        let mut current_total_value = 0;
 
        for i in 0..self.top_longs.len() {
            current_total_value += self.get_column_value(i, abacus_long_query) * self.abacus_base.pow(i as u32);
        }
        self.total_value = current_total_value; // Update internal state
        current_total_value
    }

    /// Sets the beads of a specific column to represent the target value.
    /// Clamps the value to the maximum representable by the column configuration.
    pub fn set_column_value(
        &self,
        column_index: usize,
        target_value: u64,
        abacus_long_query: &mut Query<&mut AbacusLong>,
        commands: &mut Commands, 
    ) {
        if column_index >= self.top_longs.len() {
            warn!("set_column_value: Index {} out of bounds", column_index);
            return;
        }

        let max_bottom_value = self.bottom_bead_count as u64;
        // Max value assumes top bead contributes top_bead_base_value if active
        let max_column_value = if self.top_bead_count > 0 {
             max_bottom_value + self.top_bead_base_value 
        } else {
             max_bottom_value
        };

        // Clamp the target value
        let clamped_value = target_value.min(max_column_value);

        let top_long_entity = self.top_longs[column_index];
        let bottom_long_entity = self.bottom_longs[column_index];

        let mut top_active = false;
        let mut value_from_bottom = clamped_value;

        // Determine if top bead needs activation and adjust remaining value
        if self.top_bead_count > 0 && clamped_value >= self.top_bead_base_value {
            top_active = true;
            value_from_bottom = clamped_value - self.top_bead_base_value;
        }

        // Ensure value_from_bottom doesn't exceed what bottom beads can show
        value_from_bottom = value_from_bottom.min(max_bottom_value);

        // Update top AbacusLong
        if let Ok(mut top_long) = abacus_long_query.get_mut(top_long_entity) {
            // Set based on activation state. Using 1 for active (odd), 0 for inactive (even).
            // Assumes get_column_value uses % 2 correctly.
            top_long.value = if top_active { 1 } else { 0 };
        } else {
            error!("Failed to get mutable AbacusLong for top entity at index {}", column_index);
        }

        // Update bottom AbacusLong
        if let Ok(mut bottom_long) = abacus_long_query.get_mut(bottom_long_entity) {
            // bottom_long.value stores count of beads *away* from the bar (inactive)
            bottom_long.value = max_bottom_value - value_from_bottom;
        } else {
            error!("Failed to get mutable AbacusLong for bottom entity at index {}", column_index);
        }
        
        // Signal that the abacus state changed
        commands.send_event(AbacusChanged);
    }

    /// Sets the abacus beads to represent the target total value.
    pub fn set_total_value(
        &mut self,
        mut target_total_value: u64,
        abacus_long_query: &mut Query<&mut AbacusLong>,
        commands: &mut Commands,
    ) {
        let num_columns = self.top_longs.len();
        
        // Calculate the maximum possible value the abacus can hold with current settings
        let max_column_val = if self.top_bead_count > 0 { 
             self.bottom_bead_count as u64 + self.top_bead_base_value 
        } else { 
             self.bottom_bead_count as u64 
        };
        let mut max_abacus_val = 0;
        for i in 0..num_columns {
             max_abacus_val += max_column_val * self.abacus_base.pow(i as u32);
        }
        
        // Clamp the target value to what the abacus can represent
        target_total_value = target_total_value.min(max_abacus_val);
        
        let mut remaining_value = target_total_value;

        // Iterate from most significant column down to least significant
        for i in (0..num_columns).rev() {
            let base_power = self.abacus_base.pow(i as u32);
            if base_power == 0 && remaining_value > 0 && i > 0 { // Avoid division by zero for large bases/powers
                 warn!("Abacus base calculation overflow for column {}, skipping", i);
                 continue;
            }
            if base_power == 0 && i == 0 { // Handle the last column if base is huge
                 let column_value = remaining_value;
                 self.set_column_value(i, column_value, abacus_long_query, commands);
                 remaining_value = 0;
            } else {
                let column_value = remaining_value / base_power;
                self.set_column_value(i, column_value, abacus_long_query, commands);
                remaining_value %= base_power;
            }
        }
        
        // Update the internal total_value state (might be slightly redundant if get_total_value is called later, but good practice)
        self.total_value = target_total_value;
        // Final event send handled by set_column_value calls
    }
}
        

pub fn spawn_abacus(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    settings: &crate::AbacusSettings,
) {
    let mut top_longs_temp = Vec::new();
    let mut bottom_longs_temp = Vec::new();
    let mut column_texts = Vec::new();
    
    let text_font = TextFont {
        font_size: 64.0,
        ..default()
    };
    let scale = Vec3::new(-0.01, 0.01, 0.01);

    let column_count = settings.column_count;
    let top_bead_count = settings.top_bead_count;
    let bottom_bead_count = settings.bottom_bead_count;
    let top_bead_base_value = settings.top_bead_base_value;
    let abacus_base = settings.abacus_base;
    let bead_material_handle = &settings.bead_material;
    let bead_hover_material_handle = &settings.bead_hover_material;
    let frame_material_handle = &settings.frame_material;

    let top_long_y = (bottom_bead_count as f32) * BEAD_SPACING + LONG_SPACING + ROW_SPACING;
    let top_abacus_y = top_long_y + (top_bead_count as f32) * BEAD_SPACING + LONG_SPACING;

    for i in 0..column_count {
        let top_long = spawn_abacus_long(commands, meshes, top_bead_count, bead_material_handle, bead_hover_material_handle, frame_material_handle, 0);
        let bottom_long = spawn_abacus_long(commands, meshes, bottom_bead_count, bead_material_handle, bead_hover_material_handle, frame_material_handle, bottom_bead_count as u64);

        let x = (i as f32 - ((column_count as f32 - 1.0) / 2.0)) * COLUMN_SPACING;
        
        commands.entity(top_long).insert(Transform {
            translation: Vec3::new(x, top_long_y - top_abacus_y/2.0, 0.0),
            ..default()
        });

        commands.entity(bottom_long).insert(Transform {
            translation: Vec3::new(x, - top_abacus_y/2.0, 0.0),
            ..default()
        });

        top_longs_temp.push(top_long);
        bottom_longs_temp.push(bottom_long);

        let y = -0.7; 
        let text_entity = commands.spawn((
            Text2d::new("0"),
            text_font.clone(),
            Transform::from_xyz(x, y- top_abacus_y/2.0, 0.0).with_scale(scale.clone()),
            Visibility::Inherited,
            InheritedVisibility::default(),
        )).id();
        column_texts.push(text_entity);
    }

    let total_text_entity = commands.spawn((
        Text2d::new("0"),
        text_font.clone(),
        Transform::from_xyz(0.0, top_abacus_y/2.0 + 0.1, 0.0).with_scale(scale.clone()),
        Visibility::Inherited,
        InheritedVisibility::default(),
    )).id();

    let abacus_id = commands.spawn((
        Abacus {
            top_longs: top_longs_temp.clone(),
            bottom_longs: bottom_longs_temp.clone(),
            column_texts: column_texts.clone(),
            total_text: total_text_entity,
            top_bead_count,
            bottom_bead_count,
            top_bead_base_value,
            abacus_base,
            total_value: 0,
        },
        InheritedVisibility::default(),
    )).id();

    for &top_long_entity in &top_longs_temp {
        commands.entity(abacus_id).add_child(top_long_entity);
    }
    for &bottom_long_entity in &bottom_longs_temp {
        commands.entity(abacus_id).add_child(bottom_long_entity);
    }
    for &text_entity in &column_texts {
        commands.entity(abacus_id).add_child(text_entity);
    }
    commands.entity(abacus_id).add_child(total_text_entity);

    commands.send_event(AbacusChanged);
}