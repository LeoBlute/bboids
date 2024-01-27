use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui,
    EguiContexts, EguiPlugin,
};

use crate::boids::{BoidsValues, DebugValues, BOUNDS};

#[derive(Clone, Default)]
pub struct UiPlugin;

#[derive(Component)]
struct FPSText;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_plugins((FrameTimeDiagnosticsPlugin::default(), EguiPlugin))
		.add_systems(Startup, setup)
		.add_systems(Update, measure_fps)
		.add_systems(Update, click_to_point_target)
		.add_systems(Update, ui_settings);
	}
}

fn setup(
	mut commands: Commands
) {
	let mut fps_text = TextBundle::from_sections([
		TextSection {
			value: "FPS:".to_owned(),
			style: TextStyle {
				font_size: 30.0,
				color: Color::WHITE,
				..default()
			},
		},
		TextSection {
			value: "0".to_owned(),
			style: TextStyle {
				font_size: 30.0,
				color: Color::GREEN,
				..default()
			},
		}
	]);

	fps_text = fps_text.with_style(Style {
		position_type: PositionType::Absolute,
		top: Val::Px(10.0),
		left: Val::Px(10.0),
		..Default::default()
	});

	commands.spawn((fps_text, FPSText));
}

fn click_to_point_target(
	q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
	q_camera: Query<(&Camera, &GlobalTransform)>,
	r_click: Res<Input<MouseButton>>,
	mut rm_values: ResMut<BoidsValues>,
) {
	if !r_click.just_pressed(MouseButton::Left)|| !rm_values.target_pos.is_some() {
		return;
	}

	let Ok(window) = q_window.get_single() else { return };
	let Ok((camera, gt_camera)) = q_camera.get_single() else { return };

	let Some(cursor_pos) = window.cursor_position()
		.and_then(|cursor| camera.viewport_to_world_2d(gt_camera, cursor))
	else { return };

	let pos = cursor_pos.min(BOUNDS).max(-BOUNDS);
	
	rm_values.target_pos = Some(pos);
}

fn ui_settings(
	mut context: EguiContexts,
	mut rm_values: ResMut<BoidsValues>,
	mut rm_debug_values: ResMut<DebugValues>,
) {
	egui::Window::new("Setting").show(context.ctx_mut(), |ui| {
		let is_using_target = rm_values.target_pos.is_some();
		let mut use_target = is_using_target;

		ui.add(egui::Checkbox::new(&mut rm_debug_values.show_grid, "Show grid around single boid"));
		ui.add(egui::Slider::new(&mut rm_values.count, 1..=1000).text("Count"));
		ui.add(egui::Checkbox::new(&mut use_target, "Use Target with click"));
		ui.add(egui::Slider::new(&mut rm_values.target_steering, 0.002..=2.0).text("Target Steering"));
		ui.add(egui::Slider::new(&mut rm_values.visual_range, 1.0..=100.0).text("Visual Range"));
		ui.add(egui::Slider::new(&mut rm_values.protected_range, 1.0..=100.0).text("Protected Range"));
		ui.add(egui::Slider::new(&mut rm_values.matching, 0.002..=0.2).text("Matching Factor"));
		ui.add(egui::Slider::new(&mut rm_values.avoid, 2.0..=200.0).text("Avoid Factor"));
		ui.add(egui::Slider::new(&mut rm_values.centering, 0.0002..=0.02).text("Centering Factor"));
		ui.add(egui::Slider::new(&mut rm_values.turn, 0.01..=10.0).text("Turn from edges Factor"));
		ui.add(egui::Slider::new(&mut rm_values.min_speed, 1.0..=100.0).text("Minimum Speed"));
		ui.add(egui::Slider::new(&mut rm_values.max_speed, 1.0..=100.0).text("Maximum Speed"));

		if use_target != is_using_target {
			rm_values.target_pos = if is_using_target {
				None
			} else {
				Some(Vec2::ZERO)
			}
		}
	});
}

fn measure_fps(
	r_diagnostics: Res<DiagnosticsStore>,
	mut q: Query<&mut Text, With<FPSText>>
) {
	let Ok(mut text) = q.get_single_mut() else { return };
	let Some(fps) = r_diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) else { return };
	let Some(avg) = fps.average() else { return };
	text.sections[1].value = format!("{:.0}", avg);
}
