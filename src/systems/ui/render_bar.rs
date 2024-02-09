use crate::{
	components::{Bar, UI},
	traits::ui::UIBarColors,
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, Query},
		world::Mut,
	},
	hierarchy::{BuildChildren, DespawnRecursiveExt},
	math::Vec2,
	prelude::default,
	ui::{node_bundles::NodeBundle, BackgroundColor, PositionType, Style, Val},
};

const BASE_DIMENSIONS: Vec2 = Vec2::new(100., 10.);

#[derive(Component)]
pub struct BackGroundRef(Entity);

pub fn render_bar<T: Send + Sync + 'static>(
	mut commands: Commands,
	mut bars: Query<(Entity, &mut Bar<T>)>,
	mut styles: Query<&mut Style>,
	backgrounds: Query<(Entity, &BackGroundRef)>,
) where
	Bar<T>: UIBarColors,
{
	for (background, ..) in backgrounds.iter().filter(|(_, b)| bars.get(b.0).is_err()) {
		remove_bar_nodes(&mut commands, background);
	}

	for (bar_id, bar) in &mut bars {
		match (bar.position, bar.ui) {
			(Some(position), None) => add_bar_nodes(bar, &mut commands, bar_id, position),
			(Some(position), Some(ui)) => update_bar_nodes(&mut styles, ui, bar, position),
			(None, Some(ui)) => remove_bar_nodes(&mut commands, ui.background),
			_ => noop(),
		}
	}
}

fn add_bar_nodes<T: Send + Sync + 'static>(
	mut bar: Mut<Bar<T>>,
	commands: &mut Commands,
	bar_id: Entity,
	position: Vec2,
) where
	Bar<T>: UIBarColors,
{
	let scaled_dimension = BASE_DIMENSIONS * bar.scale;
	let background = commands
		.spawn((
			BackGroundRef(bar_id),
			NodeBundle {
				style: Style {
					width: Val::Px(scaled_dimension.x),
					height: Val::Px(scaled_dimension.y),
					position_type: PositionType::Absolute,
					left: Val::Px(position.x - scaled_dimension.x / 2.),
					top: Val::Px(position.y - scaled_dimension.y / 2.),
					..default()
				},
				background_color: BackgroundColor::from(Bar::<T>::background_color()),
				..default()
			},
		))
		.id();
	let foreground = commands
		.spawn(NodeBundle {
			style: Style {
				width: Val::Percent(bar.current / bar.max * 100.),
				height: Val::Percent(100.),
				..default()
			},
			background_color: BackgroundColor::from(Bar::<T>::foreground_color()),
			..default()
		})
		.set_parent(background)
		.id();
	bar.ui = Some(UI {
		foreground,
		background,
	});
}

fn update_bar_nodes<T>(styles: &mut Query<&mut Style>, ui: UI, bar: Mut<Bar<T>>, position: Vec2) {
	if let Ok(mut background) = styles.get_mut(ui.background) {
		let scaled_dimension = BASE_DIMENSIONS * bar.scale;
		background.left = Val::Px(position.x - scaled_dimension.x / 2.);
		background.top = Val::Px(position.y - scaled_dimension.y / 2.);
	}
	if let Ok(mut foreground) = styles.get_mut(ui.foreground) {
		foreground.width = Val::Percent(bar.current / bar.max * 100.);
	}
}

fn remove_bar_nodes(commands: &mut Commands, id: Entity) {
	commands.entity(id).despawn_recursive()
}

fn noop() {}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{component::Component, world::EntityRef},
		hierarchy::{BuildWorldChildren, Parent},
		math::Vec2,
		render::color::Color,
		ui::{BackgroundColor, Node},
	};

	struct _Display;

	impl UIBarColors for Bar<_Display> {
		fn background_color() -> Color {
			Color::BLACK
		}

		fn foreground_color() -> Color {
			Color::GOLD
		}
	}

	fn no_parent(entity: &EntityRef) -> bool {
		!entity.contains::<Parent>()
	}

	fn with_parent(entity: &EntityRef) -> bool {
		entity.contains::<Parent>()
	}

	#[test]
	fn add_node_bundle() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::default()), 0., 0., 1.);
		let bar = app.world.spawn(bar).id();

		app.update();

		let (background, ..) = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| Some((e.id(), e.get::<Node>()?)))
			.unwrap();
		let (foreground, ..) = app
			.world
			.iter_entities()
			.filter(with_parent)
			.find_map(|e| Some((e.id(), e.get::<Node>()?)))
			.unwrap();
		let bar = app.world.entity(bar).get::<Bar<_Display>>().unwrap();

		assert_eq!(
			Some(UI {
				background,
				foreground
			}),
			bar.ui
		);
	}

	#[test]
	fn set_dimensions() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::default()), 0., 0., 1.);
		app.world.spawn(bar);

		app.update();

		let style = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| e.get::<Style>())
			.unwrap();

		assert_eq!(
			(Val::Px(BASE_DIMENSIONS.x), Val::Px(BASE_DIMENSIONS.y)),
			(style.width, style.height)
		);
	}

	#[test]
	fn set_dimensions_scaled() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::default()), 0., 0., 2.);
		app.world.spawn(bar);

		app.update();

		let style = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| e.get::<Style>())
			.unwrap();

		assert_eq!(
			(
				Val::Px(BASE_DIMENSIONS.x * 2.),
				Val::Px(BASE_DIMENSIONS.y * 2.)
			),
			(style.width, style.height)
		);
	}

	#[test]
	fn set_position() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::new(300., 400.)), 0., 0., 1.);
		app.world.spawn(bar);

		app.update();

		let style = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| e.get::<Style>())
			.unwrap();

		assert_eq!(
			(
				PositionType::Absolute,
				Val::Px(300. - BASE_DIMENSIONS.x / 2.),
				Val::Px(400. - BASE_DIMENSIONS.y / 2.)
			),
			(style.position_type, style.left, style.top)
		);
	}

	#[test]
	fn set_background_color() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::default()), 0., 0., 1.);
		app.world.spawn(bar);

		app.update();

		let color = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| e.get::<BackgroundColor>())
			.unwrap();

		assert_eq!(Bar::<_Display>::background_color(), color.0);
	}

	#[test]
	fn set_foreground_color() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::default()), 0., 0., 1.);
		app.world.spawn(bar);

		app.update();

		let color = app
			.world
			.iter_entities()
			.filter(with_parent)
			.find_map(|e| e.get::<BackgroundColor>())
			.unwrap();

		assert_eq!(Bar::<_Display>::foreground_color(), color.0);
	}

	#[test]
	fn set_position_scaled() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::new(300., 400.)), 0., 0., 2.);
		app.world.spawn(bar);

		app.update();

		let style = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| e.get::<Style>())
			.unwrap();

		assert_eq!(
			(
				PositionType::Absolute,
				Val::Px(300. - BASE_DIMENSIONS.x * 2. / 2.),
				Val::Px(400. - BASE_DIMENSIONS.y * 2. / 2.)
			),
			(style.position_type, style.left, style.top)
		);
	}

	#[test]
	fn set_fill() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::new(300., 400.)), 10., 50., 2.);
		app.world.spawn(bar);

		app.update();

		let style = app
			.world
			.iter_entities()
			.filter(with_parent)
			.find_map(|e| e.get::<Style>())
			.unwrap();

		assert_eq!(Val::Percent(20.), style.width);
	}

	#[test]
	fn update_position() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::new(300., 400.)), 0., 0., 1.);
		let bar = app.world.spawn(bar).id();

		app.update();

		let mut bar = app.world.entity_mut(bar);
		let mut bar = bar.get_mut::<Bar<_Display>>().unwrap();
		bar.position = Some(Vec2::new(100., 200.));

		app.update();

		let styles = app
			.world
			.iter_entities()
			.filter(no_parent)
			.filter_map(|e| e.get::<Style>())
			.collect::<Vec<_>>();
		let style = styles[0];

		assert_eq!(
			(
				1,
				Val::Px(100. - BASE_DIMENSIONS.x / 2.),
				Val::Px(200. - BASE_DIMENSIONS.y / 2.)
			),
			(styles.len(), style.left, style.top)
		);
	}

	#[test]
	fn update_position_scaled() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::new(300., 400.)), 0., 0., 2.);
		let bar = app.world.spawn(bar).id();

		app.update();

		let mut bar = app.world.entity_mut(bar);
		let mut bar = bar.get_mut::<Bar<_Display>>().unwrap();
		bar.position = Some(Vec2::new(100., 200.));

		app.update();

		let styles = app
			.world
			.iter_entities()
			.filter(no_parent)
			.filter_map(|e| e.get::<Style>())
			.collect::<Vec<_>>();
		let style = styles[0];

		assert_eq!(
			(
				1,
				Val::Px(100. - BASE_DIMENSIONS.x * 2. / 2.),
				Val::Px(200. - BASE_DIMENSIONS.y * 2. / 2.)
			),
			(styles.len(), style.left, style.top)
		);
	}

	#[test]
	fn update_fill() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::new(300., 400.)), 0., 0., 2.);
		let bar = app.world.spawn(bar).id();

		app.update();

		let mut bar = app.world.entity_mut(bar);
		let mut bar = bar.get_mut::<Bar<_Display>>().unwrap();
		bar.max = 200.;
		bar.current = 120.;

		app.update();

		let style = app
			.world
			.iter_entities()
			.filter(with_parent)
			.find_map(|e| e.get::<Style>())
			.unwrap();

		assert_eq!(Val::Percent(120. / 200. * 100.), style.width);
	}

	#[test]
	fn remove_node_recursive_when_bar_removed() {
		#[derive(Component)]
		struct _Child;

		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::default()), 0., 0., 1.);
		let bar = app.world.spawn(bar).id();

		app.update();

		let (node_id, ..) = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| Some((e.id(), e.get::<Node>()?)))
			.unwrap();
		app.world.entity_mut(node_id).with_children(|parent| {
			parent.spawn(_Child);
		});
		app.world.entity_mut(bar).despawn();

		app.update();

		let nodes = app.world.iter_entities().filter_map(|e| e.get::<Node>());
		let children = app.world.iter_entities().filter_map(|e| e.get::<_Child>());

		assert_eq!((0, 0), (nodes.count(), children.count()));
	}

	#[test]
	fn remove_node_recursive_when_bar_position_is_none() {
		#[derive(Component)]
		struct _Child;

		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar::<_Display>::new(Some(Vec2::default()), 0., 0., 1.);
		let bar = app.world.spawn(bar).id();

		app.update();

		let (node_id, ..) = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| Some((e.id(), e.get::<Node>()?)))
			.unwrap();
		app.world.entity_mut(node_id).with_children(|parent| {
			parent.spawn(_Child);
		});
		let mut bar = app.world.entity_mut(bar);
		let mut bar = bar.get_mut::<Bar<_Display>>().unwrap();
		bar.position = None;

		app.update();

		let nodes = app.world.iter_entities().filter_map(|e| e.get::<Node>());
		let children = app.world.iter_entities().filter_map(|e| e.get::<_Child>());

		assert_eq!((0, 0), (nodes.count(), children.count()));
	}
}
