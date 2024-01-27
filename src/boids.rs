use bevy::prelude::*;
use rand::prelude::*;
use rayon::prelude::*;

use crate::hash_grid::*;

pub const BOUNDS: Vec2 = Vec2::splat(100.0);
pub const STARTING_VELOCITY: f32 = 100.0;
pub const BOID_SIZE: f32 = 1.0;

#[derive(Clone, Default)]
pub struct BoidsPlugin;

#[derive(Default, Clone, Resource)]
pub struct BoidsValues {
	pub count: usize,
	pub target_pos: Option<Vec2>,
	pub target_steering: f32,
	pub visual_range: f32,
	pub protected_range: f32,
	pub matching: f32,
	pub avoid: f32,
	pub centering: f32,
	pub turn: f32,
	pub min_speed: f32,
	pub max_speed: f32,
}

#[derive(Default, Clone, Resource)]
pub struct DebugValues {
	pub show_grid: bool,
}

#[derive(Default, Clone, Component)]
struct Boid {
	velocity: Vec2,
}

#[derive(Default, Clone, Bundle)]
struct BoidBundle {
	sprite_bundle: SpriteBundle,
	boid: Boid,
}

impl Plugin for BoidsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup)
		.add_systems(Update, (
			update_count,
			apply_deferred,
			update_hash_grid,
			calculate_velocity,
			apply_velocity).chain()
		)
		.add_systems(Update, spacing_view)
		.insert_resource(HashGrid::new(15))
		.insert_resource(BoidsValues {
			count: 400,
			target_pos: Some(Vec2::ZERO),
			target_steering: 0.3,
			visual_range: 40.0,
			protected_range: 2.0,
			matching: 0.03,
			avoid: 50.0,
			centering: 0.005,
			turn: 1.2,
			min_speed: 30.0,
			max_speed: 60.0,
		})
		.insert_resource(DebugValues {
			show_grid: false,
		});
	}
}

fn update_count(
	mut commands: Commands,
	q_boids: Query<Entity, With<Boid>>,
	r_asset_server: Res<AssetServer>,
	r_values: Res<BoidsValues>,
) {
	let current = q_boids.iter().len();
	let count = r_values.count;
	let disparity = (count as isize - current as isize).abs() as usize;

	if current > count {
		for (index, id) in q_boids.iter().enumerate() {
			if index > disparity {
				break;
			}
			commands.entity(id).despawn();
		}
	} else if current < count {
		let mut boid = BoidBundle {
			sprite_bundle: SpriteBundle {
				sprite: Sprite {
					custom_size: Some(Vec2::splat(BOID_SIZE)),
					..default()
				},
				texture: r_asset_server.load("asset.png"),
				..default()
			},
			..default()
		};

		let mut rng = rand::thread_rng();

		for _ in 0..disparity {
			let x = rng.gen_range(-BOUNDS.x..BOUNDS.x);
			let y = rng.gen_range(-BOUNDS.y..BOUNDS.y);
			let vx = rng.gen_range(-STARTING_VELOCITY..STARTING_VELOCITY);
			let vy = rng.gen_range(-STARTING_VELOCITY..STARTING_VELOCITY);
			boid.sprite_bundle.transform = Transform::from_xyz(x, y, 100.0);
			boid.boid.velocity = Vec2::new(vx, vy);
			commands.spawn(boid.clone());
		}
	}
}

fn spacing_view(
	mut to_clear: Local<Vec<Entity>>,
	mut q_boids: Query<(Entity, &Transform, &mut Sprite), With<Boid>>,
	r_grid: Res<HashGrid>,
	r_values: Res<BoidsValues>,
	r_debug_values: Res<DebugValues>,
) {
	for id in to_clear.iter() {
		let Ok(mut clear_sprite) = q_boids.get_component_mut::<Sprite>(*id) else { continue };
		clear_sprite.color = Color::WHITE;
	}
	to_clear.clear();
	let Some((id, t, mut sprite)) = q_boids.iter_mut().last() else { return };

	if r_debug_values.show_grid {
		sprite.color = Color::RED;
		for other_id in r_grid.get_in_radius(t.translation.xy(), r_values.visual_range) {
			if other_id == id { continue };
			let Ok(mut other_sprite) = q_boids.get_component_mut::<Sprite>(other_id) else { continue };
			other_sprite.color = Color::PURPLE;
			to_clear.push(other_id);
		}
	} else {
		sprite.color = Color::WHITE;
	}
}

fn setup(
	mut commands: Commands,
	r_asset_server: Res<AssetServer>,
	r_values: Res<BoidsValues>,
) {
	let bounds_area = SpriteBundle {
		sprite: Sprite {
			custom_size: Some(BOUNDS * 2.0),
			color: Color::rgb(0.0, 0.0, 1.0),
			..default()
		},
		..default()
	};
	commands.spawn(bounds_area);

	let mut boid = BoidBundle {
		sprite_bundle: SpriteBundle {
			sprite: Sprite {
				custom_size: Some(Vec2::splat(BOID_SIZE)),
				..default()
			},
			texture: r_asset_server.load("asset.png"),
			..default()
		},
		..default()
	};

	let mut rng = rand::thread_rng();

	for _ in 0..r_values.count {
		let x = rng.gen_range(-BOUNDS.x..BOUNDS.x);
		let y = rng.gen_range(-BOUNDS.y..BOUNDS.y);
		let vx = rng.gen_range(-STARTING_VELOCITY..STARTING_VELOCITY);
		let vy = rng.gen_range(-STARTING_VELOCITY..STARTING_VELOCITY);
		boid.sprite_bundle.transform = Transform::from_xyz(x, y, 100.0);
		boid.boid.velocity = Vec2::new(vx, vy);
		commands.spawn(boid.clone());
	}
}

fn update_hash_grid(
	q_boids: Query<(Entity, &Transform), With<Boid>>,
	mut rm_hash_grid: ResMut<HashGrid>,
) {
	q_boids.iter().for_each(|(id, transform)|{
		rm_hash_grid.update_entity(id, transform.translation.xy());
	});
}

fn calculate_velocity(
	mut q_boids: Query<(Entity, &Transform, &mut Boid)>,
	r_grid: Res<HashGrid>,
	r_values: Res<BoidsValues>
) {
	unsafe {
		let mut v_boids = q_boids.iter_unsafe().collect::<Vec<_>>();
		let q_boids = q_boids.to_readonly();
		v_boids.par_iter_mut().for_each(|(id, t, b)| {
			let mut vel_avg = Vec2::ZERO;
			let mut pos_avg = Vec2::ZERO;
			let mut close = Vec2::ZERO;
			let mut count = 0;
			let mut avoid_count = 0;

			let pos = t.translation.xy();

			let id = *id;
			for other_id in r_grid.get_in_radius(pos, r_values.visual_range) {
				if id == other_id { continue };
				let Ok((_, other_t, other_b)) = q_boids.get(other_id) else { continue };
				
				let other_pos = other_t.translation.xy();
				let distance = pos.distance(other_pos);

				if distance < r_values.visual_range {
					//Cohesion factor
					//Aligment factor
					vel_avg += other_b.velocity;
					pos_avg += other_pos;
					count += 1;
				};
				if distance < r_values.protected_range {
					//Separation factor
					//close += pos - other_pos;
					close += (pos - other_pos).normalize() / distance;
					avoid_count +=1;
				}
			}

			let mut final_velocity = Vec2::ZERO;
			if avoid_count > 0 {
				close /= avoid_count as f32;
				close *= r_values.avoid;
				final_velocity+=close;
			}

			if count > 0 {
				vel_avg = vel_avg / (count as f32);
				pos_avg = pos_avg / (count as f32);
				
				final_velocity += (vel_avg - b.velocity)*r_values.matching;
				final_velocity += (pos_avg - pos)*r_values.centering;
 			}

			b.velocity += final_velocity;
		});
	}
}

fn apply_velocity(
	mut query: Query<(&mut Boid, &mut Transform)>,
	r_values: Res<BoidsValues>,
	r_time: Res<Time>,
) {
	let left_margin = -BOUNDS.x;
	let right_margin = BOUNDS.x;
	let bottom_margin = -BOUNDS.y;
	let top_margin = BOUNDS.y;
	query.par_iter_mut().for_each(|(mut boid, mut transform)| {
		let mut pos = transform.translation.xy();
		let mut velocity = boid.velocity;

		if let Some(target) = r_values.target_pos {
			velocity -= (pos - target).normalize() * r_values.target_steering;
		}

		let speed = f32::sqrt(velocity.x*velocity.x + velocity.y*velocity.y);

		if pos.x < left_margin { velocity.x += r_values.turn };
		if pos.x > right_margin { velocity.x -= r_values.turn };
		if pos.y > bottom_margin { velocity.y -= r_values.turn };
		if pos.y < top_margin { velocity.y += r_values.turn };
		
		if speed < r_values.min_speed {
			velocity = (velocity/speed)*r_values.min_speed;
		}
		if speed > r_values.max_speed {
			velocity = (velocity/speed)*r_values.max_speed;
		}

		boid.velocity = velocity;

		pos += boid.velocity * r_time.delta_seconds();
		transform.translation = pos.extend(100.0);
		let direction = boid.velocity.normalize();
		let angle = direction.x.atan2(direction.y);
		transform.rotation = Quat::from_rotation_z(-angle);
	});
}
