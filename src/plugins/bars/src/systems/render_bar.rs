use crate::{
	components::{bar::Bar, bar_values::BarValues, ui::UI},
	traits::UIBarColors,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	components::ui_node_for::UiNodeFor,
	traits::{
		accessors::get::GetContextMut,
		handles_graphics::{CameraHandle, RenderUi},
		thread_safe::ThreadSafe,
	},
};

const BASE_DIMENSIONS: Vec2 = Vec2::new(100., 10.);

pub(crate) fn render_bar<T, TCamera>(
	mut commands: Commands,
	mut bars: Query<(Entity, &Bar, &mut BarValues<T>)>,
	mut styles: Query<&mut Node>,
	mut camera: StaticSystemParam<TCamera>,
) where
	T: ThreadSafe,
	BarValues<T>: UIBarColors,
	TCamera: for<'c> GetContextMut<CameraHandle, TContext<'c>: RenderUi>,
{
	let cam = &mut TCamera::get_context_mut(&mut camera, CameraHandle);

	for (bar_id, bar, bar_values) in &mut bars {
		match (bar.position, bar_values.ui) {
			(Some(position), None) => add_ui(&mut commands, bar_id, bar, bar_values, position, cam),
			(Some(position), Some(ui)) => update_ui(&mut styles, ui, bar, bar_values, position),
			_ => noop(),
		}
	}
}

fn add_ui<T>(
	commands: &mut Commands,
	bar_id: Entity,
	bar: &Bar,
	mut bar_values: Mut<BarValues<T>>,
	position: Vec2,
	camera: &mut impl RenderUi,
) where
	T: ThreadSafe,
	BarValues<T>: UIBarColors,
{
	let scaled_dimension = BASE_DIMENSIONS * bar.scale;
	let background = commands
		.spawn((
			UiNodeFor::<Bar>::with(bar_id),
			Node {
				width: Val::Px(scaled_dimension.x),
				height: Val::Px(scaled_dimension.y),
				position_type: PositionType::Absolute,
				left: Val::Px(position.x - scaled_dimension.x / 2.),
				top: Val::Px(position.y - scaled_dimension.y / 2.),
				..default()
			},
			BackgroundColor::from(BarValues::<T>::background_color()),
		))
		.id();
	let foreground = commands
		.spawn((
			Node {
				width: Val::Percent(bar_values.current / bar_values.max * 100.),
				height: Val::Percent(100.),
				..default()
			},
			BackgroundColor::from(BarValues::<T>::foreground_color()),
		))
		.insert(ChildOf(background))
		.id();

	camera.render_ui(background);
	bar_values.ui = Some(UI {
		foreground,
		background,
	});
}

fn update_ui<T>(
	styles: &mut Query<&mut Node>,
	ui: UI,
	bar: &Bar,
	bar_values: Mut<BarValues<T>>,
	position: Vec2,
) {
	if let Ok(mut background) = styles.get_mut(ui.background) {
		let scaled_dimension = BASE_DIMENSIONS * bar.scale;
		background.left = Val::Px(position.x - scaled_dimension.x / 2.);
		background.top = Val::Px(position.y - scaled_dimension.y / 2.);
	}
	if let Ok(mut foreground) = styles.get_mut(ui.foreground) {
		foreground.width = Val::Percent(bar_values.current / bar_values.max * 100.);
	}
}

fn noop() {}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Resource, Default)]
	struct _Camera {
		renders: Vec<Entity>,
	}

	impl RenderUi for &mut _Camera {
		fn render_ui(&mut self, ui: Entity) {
			self.renders.push(ui);
		}
	}

	struct _Display;

	impl UIBarColors for BarValues<_Display> {
		fn background_color() -> Color {
			Color::BLACK
		}

		fn foreground_color() -> Color {
			Color::srgb(0.8, 0.7, 0.23)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Camera>();
		app.add_systems(Update, render_bar::<_Display, ResMut<_Camera>>);

		app
	}

	#[test]
	fn add_node_bundle() {
		let mut app = setup();
		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world_mut().spawn((bar, bar_values)).id();

		app.update();

		let mut backgrounds = app
			.world_mut()
			.query_filtered::<Entity, (With<Node>, Without<ChildOf>)>();
		let mut foregrounds = app
			.world_mut()
			.query_filtered::<Entity, (With<Node>, With<ChildOf>)>();
		let mut bar_values = app.world_mut().query::<&BarValues<_Display>>();
		let background = backgrounds.iter(app.world()).next().unwrap();
		let foreground = foregrounds.iter(app.world()).next().unwrap();
		let bar = bar_values.get(app.world(), bar).unwrap();
		assert_eq!(
			Some(UI {
				background,
				foreground
			}),
			bar.ui
		);
	}

	#[test]
	fn add_camera_target() {
		let mut app = setup();
		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut backgrounds = app
			.world_mut()
			.query_filtered::<Entity, (With<Node>, Without<ChildOf>)>();
		let background = backgrounds.iter(app.world()).next().unwrap();
		assert_eq!(vec![background], app.world().resource::<_Camera>().renders);
	}

	#[test]
	fn add_ownership_on_top_node() {
		let mut app = setup();
		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world_mut().spawn((bar, bar_values)).id();

		app.update();

		let mut backgrounds = app
			.world_mut()
			.query_filtered::<&UiNodeFor<Bar>, (With<Node>, Without<ChildOf>)>();
		let background = backgrounds.iter(app.world()).next();
		assert_eq!(Some(&UiNodeFor::<Bar>::with(bar)), background);
	}

	#[test]
	fn set_dimensions() {
		let mut app = setup();
		let bar = Bar {
			scale: 1.,
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut backgrounds = app.world_mut().query_filtered::<&Node, Without<ChildOf>>();
		let background = backgrounds.iter(app.world()).next().unwrap();
		assert_eq!(
			(Val::Px(BASE_DIMENSIONS.x), Val::Px(BASE_DIMENSIONS.y)),
			(background.width, background.height)
		);
	}

	#[test]
	fn set_dimensions_scaled() {
		let mut app = setup();
		let bar = Bar {
			scale: 2.,
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut backgrounds = app.world_mut().query_filtered::<&Node, Without<ChildOf>>();
		let background = backgrounds.iter(app.world()).next().unwrap();
		assert_eq!(
			(
				Val::Px(BASE_DIMENSIONS.x * 2.),
				Val::Px(BASE_DIMENSIONS.y * 2.)
			),
			(background.width, background.height)
		);
	}

	#[test]
	fn set_position() {
		let mut app = setup();
		let bar = Bar {
			scale: 1.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut backgrounds = app.world_mut().query_filtered::<&Node, Without<ChildOf>>();
		let background = backgrounds.iter(app.world()).next().unwrap();
		assert_eq!(
			(
				PositionType::Absolute,
				Val::Px(300. - BASE_DIMENSIONS.x / 2.),
				Val::Px(400. - BASE_DIMENSIONS.y / 2.)
			),
			(background.position_type, background.left, background.top)
		);
	}

	#[test]
	fn set_position_scaled() {
		let mut app = setup();
		let bar = Bar {
			scale: 2.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut backgrounds = app.world_mut().query_filtered::<&Node, Without<ChildOf>>();
		let background = backgrounds.iter(app.world()).next().unwrap();
		assert_eq!(
			(
				PositionType::Absolute,
				Val::Px(300. - BASE_DIMENSIONS.x * 2. / 2.),
				Val::Px(400. - BASE_DIMENSIONS.y * 2. / 2.)
			),
			(background.position_type, background.left, background.top)
		);
	}

	#[test]
	fn set_background_color() {
		let mut app = setup();
		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut backgrounds = app
			.world_mut()
			.query_filtered::<&BackgroundColor, (With<Node>, Without<ChildOf>)>();
		let background = backgrounds.iter(app.world()).next().unwrap();
		assert_eq!(BarValues::<_Display>::background_color(), background.0);
	}

	#[test]
	fn set_foreground_color() {
		let mut app = setup();
		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut foreground = app
			.world_mut()
			.query_filtered::<&BackgroundColor, (With<Node>, With<ChildOf>)>();
		let foreground = foreground.iter(app.world()).next().unwrap();
		assert_eq!(BarValues::<_Display>::foreground_color(), foreground.0);
	}

	#[test]
	fn set_fill() {
		let mut app = setup();
		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(10., 50.);
		app.world_mut().spawn((bar, bar_values));

		app.update();

		let mut foreground = app.world_mut().query_filtered::<&Node, With<ChildOf>>();
		let foreground = foreground.iter(app.world()).next().unwrap();
		assert_eq!(Val::Percent(20.), foreground.width);
	}

	#[test]
	fn update_position() {
		let mut app = setup();
		let bar = Bar {
			scale: 1.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world_mut().spawn((bar, bar_values)).id();

		app.update();

		let mut bar = app.world_mut().entity_mut(bar);
		let mut bar = bar.get_mut::<Bar>().unwrap();
		bar.position = Some(Vec2::new(100., 200.));

		app.update();

		let mut backgrounds = app.world_mut().query_filtered::<&Node, Without<ChildOf>>();
		let [background] = assert_count!(1, backgrounds.iter(app.world()));
		assert_eq!(
			(
				Val::Px(100. - BASE_DIMENSIONS.x / 2.),
				Val::Px(200. - BASE_DIMENSIONS.y / 2.)
			),
			(background.left, background.top)
		);
	}

	#[test]
	fn update_position_scaled() {
		let mut app = setup();
		let bar = Bar {
			scale: 2.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world_mut().spawn((bar, bar_values)).id();

		app.update();

		let mut bar = app.world_mut().entity_mut(bar);
		let mut bar = bar.get_mut::<Bar>().unwrap();
		bar.position = Some(Vec2::new(100., 200.));

		app.update();

		let mut backgrounds = app.world_mut().query_filtered::<&Node, Without<ChildOf>>();
		let [background] = assert_count!(1, backgrounds.iter(app.world()));
		assert_eq!(
			(
				Val::Px(100. - BASE_DIMENSIONS.x * 2. / 2.),
				Val::Px(200. - BASE_DIMENSIONS.y * 2. / 2.)
			),
			(background.left, background.top)
		);
	}

	#[test]
	fn update_fill() {
		let mut app = setup();
		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world_mut().spawn((bar, bar_values)).id();

		app.update();

		let mut bar = app.world_mut().entity_mut(bar);
		let mut bar_values = bar.get_mut::<BarValues<_Display>>().unwrap();
		bar_values.max = 200.;
		bar_values.current = 120.;

		app.update();

		let mut foreground = app.world_mut().query_filtered::<&Node, With<ChildOf>>();
		let foreground = foreground.iter(app.world()).next().unwrap();
		assert_eq!(Val::Percent(120. / 200. * 100.), foreground.width);
	}
}
