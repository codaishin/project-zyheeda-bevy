use crate::traits::{
	get_node::GetNode,
	instantiate_content_on::InstantiateContentOn,
	tooltip_ui_control::{
		DespawnAllTooltips,
		DespawnOutdatedTooltips,
		SpawnTooltips,
		UpdateTooltipPosition,
	},
};
use bevy::{
	hierarchy::{BuildChildren, DespawnRecursiveExt},
	math::Vec2,
	prelude::{Commands, Component, Entity, Query, RemovedComponents, Resource},
	render::view::Visibility,
	ui::{node_bundles::NodeBundle, Style, Val},
	utils::default,
};
use std::time::Duration;

#[derive(Component)]
pub(crate) struct Tooltip<T>(pub T);

#[derive(Component)]
pub(crate) struct TooltipUI {
	source: Entity,
	pub(crate) delay: Duration,
}

#[derive(Resource, Default)]
pub(crate) struct TooltipUIControl {
	tooltip_delay: Duration,
}

impl DespawnAllTooltips<TooltipUI> for TooltipUIControl {
	fn despawn_all(&self, uis: &Query<(Entity, &TooltipUI, &mut Style)>, commands: &mut Commands) {
		for (entity, ..) in uis {
			let Some(entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn_recursive();
		}
	}
}

impl<T: Send + Sync + 'static> DespawnOutdatedTooltips<TooltipUI, T> for TooltipUIControl {
	fn despawn_outdated(
		&self,
		uis: &Query<(Entity, &TooltipUI, &mut Style)>,
		commands: &mut Commands,
		mut outdated_tooltips: RemovedComponents<Tooltip<T>>,
	) {
		let outdated = outdated_tooltips.read().collect::<Vec<_>>();
		let is_outdated = |(_, ui, _): &(Entity, &TooltipUI, &Style)| outdated.contains(&ui.source);

		for (entity, ..) in uis.iter().filter(is_outdated) {
			let Some(entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn_recursive();
		}
	}
}

impl UpdateTooltipPosition<TooltipUI> for TooltipUIControl {
	fn update_position(&self, uis: &mut Query<(Entity, &TooltipUI, &mut Style)>, position: Vec2) {
		for (.., mut style) in uis.iter_mut() {
			style.left = Val::Px(position.x);
			style.top = Val::Px(position.y);
		}
	}
}

impl<T> SpawnTooltips<T> for TooltipUIControl {
	fn spawn(
		&self,
		commands: &mut Commands,
		tooltip_entity: Entity,
		tooltip: &Tooltip<T>,
		position: Vec2,
	) where
		Tooltip<T>: InstantiateContentOn + GetNode,
	{
		let container_node = (
			TooltipUI {
				source: tooltip_entity,
				delay: self.tooltip_delay,
			},
			NodeBundle {
				style: Style {
					left: Val::Px(position.x),
					top: Val::Px(position.y),
					..default()
				},
				visibility: Visibility::Hidden,
				..default()
			},
		);
		let mut tooltip_node = tooltip.node();
		tooltip_node.visibility = Visibility::Inherited;

		commands
			.spawn(container_node)
			.with_children(|container_node| {
				container_node
					.spawn(tooltip_node)
					.with_children(|tooltip_node| {
						tooltip.instantiate_content_on(tooltip_node);
					});
			});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		hierarchy::{BuildWorldChildren, ChildBuilder, Children, Parent},
		render::color::Color,
		ui::{node_bundles::NodeBundle, BackgroundColor, Val},
		utils::default,
	};
	use common::{assert_bundle, test_tools::utils::SingleThreadedApp};

	#[derive(Component, Debug, PartialEq)]
	struct _Child(&'static str);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	fn setup_despawn_all() -> App {
		let mut app = setup();
		app.add_systems(
			Update,
			|uis: Query<(Entity, &TooltipUI, &mut Style)>, mut commands: Commands| {
				TooltipUIControl::default().despawn_all(&uis, &mut commands);
			},
		);

		app
	}

	fn setup_despawn_outdated() -> App {
		let mut app = setup();
		app.add_systems(
			Update,
			|uis: Query<(Entity, &TooltipUI, &mut Style)>,
			 mut commands: Commands,
			 outdated_tooltips: RemovedComponents<Tooltip<&'static str>>| {
				TooltipUIControl::default().despawn_outdated(
					&uis,
					&mut commands,
					outdated_tooltips,
				);
			},
		);

		app
	}

	fn setup_update_position(position: Vec2) -> App {
		let mut app = setup();
		app.add_systems(
			Update,
			move |mut uis: Query<(Entity, &TooltipUI, &mut Style)>| {
				TooltipUIControl::default().update_position(&mut uis, position);
			},
		);

		app
	}

	struct _T {
		color: Color,
		visibility: Visibility,
		content: &'static str,
	}

	impl GetNode for Tooltip<_T> {
		fn node(&self) -> NodeBundle {
			NodeBundle {
				background_color: self.0.color.into(),
				visibility: self.0.visibility,
				..default()
			}
		}
	}

	impl InstantiateContentOn for Tooltip<_T> {
		fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
			parent.spawn(_Child(self.0.content));
		}
	}

	fn setup_spawn(position: Vec2, tooltip_delay: Duration) -> App {
		let mut app = setup();
		app.add_systems(
			Update,
			move |mut commands: Commands, tooltips: Query<(Entity, &Tooltip<_T>)>| {
				for (entity, tooltip) in &tooltips {
					TooltipUIControl { tooltip_delay }.spawn(
						&mut commands,
						entity,
						tooltip,
						position,
					);
				}
			},
		);

		app
	}

	#[test]
	fn despawn_all() {
		let mut app = setup_despawn_all();
		app.world.spawn_batch([
			(
				TooltipUI {
					source: Entity::from_raw(100),
					delay: default(),
				},
				Style::default(),
			),
			(
				TooltipUI {
					source: Entity::from_raw(200),
					delay: default(),
				},
				Style::default(),
			),
		]);

		app.update();

		let tooltip_uis = app
			.world
			.iter_entities()
			.filter(|e| e.contains::<TooltipUI>());

		assert_eq!(0, tooltip_uis.count());
	}

	#[test]
	fn despawn_all_recursively() {
		let mut app = setup_despawn_all();
		app.world
			.spawn((
				TooltipUI {
					source: Entity::from_raw(100),
					delay: default(),
				},
				Style::default(),
			))
			.with_children(|parent| {
				parent.spawn(_Child(""));
			});

		app.update();

		let children = app.world.iter_entities().filter(|e| e.contains::<_Child>());

		assert_eq!(0, children.count());
	}

	#[test]
	fn despawn_outdated() {
		let mut app = setup_despawn_outdated();
		let tooltips = [
			app.world.spawn(Tooltip("1")).id(),
			app.world.spawn(Tooltip("2")).id(),
		];
		for entity in tooltips {
			app.world.spawn((
				TooltipUI {
					source: entity,
					delay: default(),
				},
				Style::default(),
			));
		}

		app.update();

		for tooltip in tooltips {
			app.world.entity_mut(tooltip).despawn();
		}

		app.update();

		let tooltip_uis = app
			.world
			.iter_entities()
			.filter(|e| e.contains::<TooltipUI>());

		assert_eq!(0, tooltip_uis.count());
	}

	#[test]
	fn despawn_outdated_recursively() {
		let mut app = setup_despawn_outdated();
		let tooltips = [
			app.world.spawn(Tooltip("1")).id(),
			app.world.spawn(Tooltip("2")).id(),
		];
		for entity in tooltips {
			app.world
				.spawn((
					TooltipUI {
						source: entity,
						delay: default(),
					},
					Style::default(),
				))
				.with_children(|parent| {
					parent.spawn(_Child(""));
				});
		}

		app.update();

		for tooltip in tooltips {
			app.world.entity_mut(tooltip).despawn();
		}

		app.update();

		let children = app.world.iter_entities().filter(|e| e.contains::<_Child>());

		assert_eq!(0, children.count());
	}

	#[test]
	fn do_not_despawn_when_not_outdated() {
		let mut app = setup_despawn_outdated();
		let tooltips = [
			app.world.spawn(Tooltip("1")).id(),
			app.world.spawn(Tooltip("2")).id(),
		];
		for entity in tooltips {
			app.world.spawn((
				TooltipUI {
					source: entity,
					delay: default(),
				},
				Style::default(),
			));
		}

		app.update();

		let tooltip_uis = app
			.world
			.iter_entities()
			.filter(|e| e.contains::<TooltipUI>());

		assert_eq!(2, tooltip_uis.count());
	}

	#[test]
	fn update_position() {
		let mut app = setup_update_position(Vec2 { x: 42., y: 11. });
		let uis = app
			.world
			.spawn_batch([
				(
					TooltipUI {
						source: Entity::from_raw(100),
						delay: default(),
					},
					Style::default(),
				),
				(
					TooltipUI {
						source: Entity::from_raw(200),
						delay: default(),
					},
					Style::default(),
				),
			])
			.collect::<Vec<_>>();

		app.update();

		let tooltip_styles = uis
			.iter()
			.map(|entity| app.world.entity(*entity).get::<Style>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![
				Some(&Style {
					left: Val::Px(42.),
					top: Val::Px(11.),
					..default()
				}),
				Some(&Style {
					left: Val::Px(42.),
					top: Val::Px(11.),
					..default()
				})
			],
			tooltip_styles
		);
	}

	#[test]
	fn spawn_tooltip() {
		let mut app = setup_spawn(Vec2 { x: 11., y: 101. }, default());
		app.world.spawn(Tooltip(_T {
			color: Color::GOLD,
			content: "",
			visibility: Visibility::Visible,
		}));

		app.update();

		let tooltip_ui = app
			.world
			.iter_entities()
			.find(|e| e.contains::<TooltipUI>())
			.expect("no tooltip spawned");

		assert_bundle!(
			NodeBundle,
			&app,
			tooltip_ui,
			With::assert(|color: &BackgroundColor| assert_eq!(Color::NONE, color.0)),
			With::assert(|style: &Style| assert_eq!(
				&Style {
					left: Val::Px(11.),
					top: Val::Px(101.),
					..default()
				},
				style
			)),
			With::assert(|visibility: &Visibility| assert_eq!(&Visibility::Hidden, visibility))
		);
	}

	#[test]
	fn spawn_node_bundle_of_tooltip_on_child() {
		let mut app = setup_spawn(default(), default());
		app.world.spawn(Tooltip(_T {
			color: Color::GOLD,
			content: "",
			visibility: Visibility::Visible,
		}));

		app.update();

		let tooltip_ui_child = app
			.world
			.iter_entities()
			.find(|e| e.contains::<TooltipUI>())
			.and_then(|t| t.get::<Children>())
			.and_then(|c| c.first())
			.expect("no tooltip child found");
		let tooltip_ui_child = app.world.entity(*tooltip_ui_child);

		assert_bundle!(
			NodeBundle,
			&app,
			tooltip_ui_child,
			With::assert(|color: &BackgroundColor| assert_eq!(Color::GOLD, color.0)),
			With::assert(|visibility: &Visibility| assert_eq!(&Visibility::Inherited, visibility))
		);
	}

	#[test]
	fn spawn_content_of_tooltip_on_child() {
		let mut app = setup_spawn(default(), default());
		app.world.spawn(Tooltip(_T {
			color: Color::GOLD,
			content: "My Content",
			visibility: Visibility::Visible,
		}));

		app.update();

		let tooltip_ui_child = app
			.world
			.iter_entities()
			.find(|e| e.contains::<TooltipUI>())
			.and_then(|t| t.get::<Children>())
			.and_then(|c| c.first())
			.expect("no tooltip child found");
		let content = app
			.world
			.iter_entities()
			.find(|e| e.get::<Parent>().map(|p| p.get()) == Some(*tooltip_ui_child))
			.expect("not matching child found");

		assert_eq!(Some(&_Child("My Content")), content.get::<_Child>());
	}

	#[test]
	fn spawn_tooltip_ui_with_source_reference() {
		let mut app = setup_spawn(default(), default());
		let tooltip = app
			.world
			.spawn(Tooltip(_T {
				color: Color::GOLD,
				content: "",
				visibility: Visibility::Visible,
			}))
			.id();

		app.update();

		let tooltip_ui = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<TooltipUI>())
			.expect("no tooltip spawned");

		assert_eq!(tooltip, tooltip_ui.source);
	}

	#[test]
	fn spawn_tooltip_ui_with_delay() {
		let mut app = setup_spawn(default(), Duration::from_secs(4000));
		app.world.spawn(Tooltip(_T {
			color: Color::GOLD,
			content: "",
			visibility: Visibility::Visible,
		}));

		app.update();

		let tooltip_ui = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<TooltipUI>())
			.expect("no tooltip spawned");

		assert_eq!(Duration::from_secs(4000), tooltip_ui.delay);
	}
}
