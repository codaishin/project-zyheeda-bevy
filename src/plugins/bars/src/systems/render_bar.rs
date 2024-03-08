use std::marker::PhantomData;

use crate::{
	components::{Bar, BarValues, UI},
	traits::UIBarColors,
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
pub struct Owned<TOwner> {
	owner: Entity,
	owner_type: PhantomData<TOwner>,
}

impl<T> Owned<T> {
	fn by(owner: Entity) -> Self {
		Self {
			owner,
			owner_type: PhantomData,
		}
	}
}

pub(crate) fn render_bar<T: Send + Sync + 'static>(
	mut commands: Commands,
	mut bars: Query<(Entity, &Bar, &mut BarValues<T>)>,
	mut styles: Query<&mut Style>,
	backgrounds: Query<(Entity, &Owned<Bar>)>,
) where
	BarValues<T>: UIBarColors,
{
	let not_owned = |(_, owned): &(Entity, &Owned<Bar>)| bars.get(owned.owner).is_err();

	for (background, ..) in backgrounds.iter().filter(not_owned) {
		remove(&mut commands, background);
	}

	for (bar_id, bar, bar_values) in &mut bars {
		match (bar.position, bar_values.ui) {
			(Some(position), None) => add_ui(&mut commands, bar_id, bar, bar_values, position),
			(Some(position), Some(ui)) => update_ui(&mut styles, ui, bar, bar_values, position),
			_ => noop(),
		}
	}
}

fn add_ui<T: Send + Sync + 'static>(
	commands: &mut Commands,
	bar_id: Entity,
	bar: &Bar,
	mut bar_values: Mut<BarValues<T>>,
	position: Vec2,
) where
	BarValues<T>: UIBarColors,
{
	let scaled_dimension = BASE_DIMENSIONS * bar.scale;
	let background = commands
		.spawn((
			Owned::<Bar>::by(bar_id),
			NodeBundle {
				style: Style {
					width: Val::Px(scaled_dimension.x),
					height: Val::Px(scaled_dimension.y),
					position_type: PositionType::Absolute,
					left: Val::Px(position.x - scaled_dimension.x / 2.),
					top: Val::Px(position.y - scaled_dimension.y / 2.),
					..default()
				},
				background_color: BackgroundColor::from(BarValues::<T>::background_color()),
				..default()
			},
		))
		.id();
	let foreground = commands
		.spawn(NodeBundle {
			style: Style {
				width: Val::Percent(bar_values.current / bar_values.max * 100.),
				height: Val::Percent(100.),
				..default()
			},
			background_color: BackgroundColor::from(BarValues::<T>::foreground_color()),
			..default()
		})
		.set_parent(background)
		.id();
	bar_values.ui = Some(UI {
		foreground,
		background,
	});
}

fn update_ui<T>(
	styles: &mut Query<&mut Style>,
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

fn remove(commands: &mut Commands, id: Entity) {
	let Some(entity) = commands.get_entity(id) else {
		return;
	};
	entity.despawn_recursive();
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

	impl UIBarColors for BarValues<_Display> {
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

		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world.spawn((bar, bar_values)).id();

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
		let bar = app.world.entity(bar).get::<BarValues<_Display>>().unwrap();

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

		let bar = Bar {
			scale: 1.,
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world.spawn((bar, bar_values));

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

		let bar = Bar {
			scale: 2.,
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world.spawn((bar, bar_values));

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

		let bar = Bar {
			scale: 1.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world.spawn((bar, bar_values));

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
	fn set_position_scaled() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar {
			scale: 2.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world.spawn((bar, bar_values));

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
	fn set_background_color() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world.spawn((bar, bar_values));

		app.update();

		let color = app
			.world
			.iter_entities()
			.filter(no_parent)
			.find_map(|e| e.get::<BackgroundColor>())
			.unwrap();

		assert_eq!(BarValues::<_Display>::background_color(), color.0);
	}

	#[test]
	fn set_foreground_color() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		app.world.spawn((bar, bar_values));

		app.update();

		let color = app
			.world
			.iter_entities()
			.filter(with_parent)
			.find_map(|e| e.get::<BackgroundColor>())
			.unwrap();

		assert_eq!(BarValues::<_Display>::foreground_color(), color.0);
	}

	#[test]
	fn set_fill() {
		let mut app = App::new();
		app.add_systems(Update, render_bar::<_Display>);

		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(10., 50.);
		app.world.spawn((bar, bar_values));

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

		let bar = Bar {
			scale: 1.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world.spawn((bar, bar_values)).id();

		app.update();

		let mut bar = app.world.entity_mut(bar);
		let mut bar = bar.get_mut::<Bar>().unwrap();
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

		let bar = Bar {
			scale: 2.,
			position: Some(Vec2::new(300., 400.)),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world.spawn((bar, bar_values)).id();

		app.update();

		let mut bar = app.world.entity_mut(bar);
		let mut bar = bar.get_mut::<Bar>().unwrap();
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

		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world.spawn((bar, bar_values)).id();

		app.update();

		let mut bar = app.world.entity_mut(bar);
		let mut bar_values = bar.get_mut::<BarValues<_Display>>().unwrap();
		bar_values.max = 200.;
		bar_values.current = 120.;

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

		let bar = Bar {
			position: Some(default()),
			..default()
		};
		let bar_values = BarValues::<_Display>::new(0., 0.);
		let bar = app.world.spawn((bar, bar_values)).id();

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
}
