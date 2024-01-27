use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

mod hash_grid;
mod boids;
mod ui;
use boids::*;
use ui::*;


fn main() {
	App::new()
	.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
	.add_plugins((BoidsPlugin, UiPlugin))
	.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
	.add_systems(Startup, setup)
	.add_systems(Update, bevy::window::close_on_esc)
	.run();
}

fn setup(mut commands: Commands) {

	let mut camera = Camera2dBundle::default();
	camera.projection.scaling_mode = ScalingMode::AutoMin {
		min_width: 512.0,
		min_height: 288.0,
	};

	commands.spawn(camera);
}
