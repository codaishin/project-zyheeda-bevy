use crate::{
	components::tooltip::Tooltip,
	traits::{children::Children, colors::HasBackgroundColor, get_node::GetNode},
};
use bevy::{
	hierarchy::{BuildChildren, DespawnRecursiveExt},
	math::Vec2,
	prelude::{Changed, Commands, Component, Entity, Query, RemovedComponents},
	ui::{node_bundles::NodeBundle, Interaction, Style, Val, ZIndex},
	utils::default,
};
use common::traits::mouse_position::MousePosition;

pub(crate) fn tooltip<T, TWindow>(
	mut commands: Commands,
	windows: Query<&TWindow>,
	new_tooltip_interactions: Query<(Entity, &Tooltip<T>, &Interaction), Changed<Interaction>>,
	mut tooltip_uis: Query<(Entity, &TooltipUI, &mut Style)>,
	removed_tooltips: RemovedComponents<Tooltip<T>>,
) where
	T: Sync + Send + 'static,
	Tooltip<T>: Children + GetNode + HasBackgroundColor,
	TWindow: Component + MousePosition,
{
	let Ok(window) = windows.get_single() else {
		return;
	};
	let Some(position) = window.mouse_position() else {
		return;
	};

	if !new_tooltip_interactions.is_empty() {
		TooltipUI::despawn_all(&tooltip_uis, &mut commands);
	} else {
		TooltipUI::update_position(&mut tooltip_uis, position);
	}

	if !removed_tooltips.is_empty() {
		TooltipUI::despawn_outdated(&tooltip_uis, &mut commands, removed_tooltips);
	}

	for (entity, tooltip, _) in new_tooltip_interactions.iter().filter(is_hovering) {
		TooltipUI::spawn(&mut commands, entity, tooltip, position);
	}
}

fn is_hovering<T: Sync + Send + 'static>(
	(.., interaction): &(Entity, &Tooltip<T>, &Interaction),
) -> bool {
	interaction == &&Interaction::Hovered
}

#[derive(Component)]
pub struct TooltipUI {
	tooltip: Entity,
}

impl TooltipUI {
	fn despawn_all(uis: &Query<(Entity, &TooltipUI, &mut Style)>, commands: &mut Commands) {
		for (entity, ..) in uis {
			let Some(entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn_recursive();
		}
	}

	fn despawn_outdated<T: Sync + Send + 'static>(
		uis: &Query<(Entity, &TooltipUI, &mut Style)>,
		commands: &mut Commands,
		mut outdated_tooltips: RemovedComponents<Tooltip<T>>,
	) {
		let outdated_tooltips = outdated_tooltips.read().collect::<Vec<_>>();
		let is_outdated =
			|(_, ui, _): &(Entity, &TooltipUI, &Style)| outdated_tooltips.contains(&ui.tooltip);

		for (entity, ..) in uis.iter().filter(is_outdated) {
			let Some(entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn_recursive();
		}
	}

	fn update_position(uis: &mut Query<(Entity, &TooltipUI, &mut Style)>, position: Vec2) {
		for (.., mut style) in uis {
			style.left = Val::Px(position.x);
			style.top = Val::Px(position.y);
		}
	}

	fn spawn<T>(commands: &mut Commands, entity: Entity, tooltip: &Tooltip<T>, position: Vec2)
	where
		T: Sync + Send + 'static,
		Tooltip<T>: Children + GetNode + HasBackgroundColor,
	{
		let ui_container = (
			TooltipUI { tooltip: entity },
			NodeBundle {
				style: Style {
					left: Val::Px(position.x),
					top: Val::Px(position.y),
					..default()
				},
				z_index: ZIndex::Global(1),
				..default()
			},
		);
		let ui_content = tooltip.node();

		commands.spawn(ui_container).with_children(|ui_container| {
			ui_container.spawn(ui_content).with_children(|ui_content| {
				tooltip.children(ui_content);
			});
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{tools::assert_node_bundle, traits::children::Children as ChildrenTrait};
	use bevy::{
		app::{App, Update},
		hierarchy::{ChildBuilder, Children, Parent},
		math::Vec2,
		render::color::Color,
		ui::{Style, Val},
		utils::default,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use std::ops::Deref;

	#[derive(Component)]
	struct _Window(Option<Vec2>);

	impl MousePosition for _Window {
		fn mouse_position(&self) -> Option<Vec2> {
			self.0
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Content(&'static str);

	struct _T<const HAS_BACKGROUND_COLOR: bool> {
		content: &'static str,
		node: NodeBundle,
	}

	impl<const HAS_BACKGROUND_COLOR: bool> GetNode for Tooltip<_T<HAS_BACKGROUND_COLOR>> {
		fn node(&self) -> NodeBundle {
			self.0.node.clone()
		}
	}

	impl<const HAS_BACKGROUND_COLOR: bool> ChildrenTrait for Tooltip<_T<HAS_BACKGROUND_COLOR>> {
		fn children(&self, parent: &mut ChildBuilder) {
			parent.spawn(_Content(self.0.content));
		}
	}

	impl HasBackgroundColor for Tooltip<_T<true>> {
		const BACKGROUND_COLOR: Option<Color> = Some(Color::CRIMSON);
	}

	impl HasBackgroundColor for Tooltip<_T<false>> {
		const BACKGROUND_COLOR: Option<Color> = None;
	}

	macro_rules! try_get_latest_container {
		($app:expr) => {
			$app.world
				.iter_entities()
				.filter(|e| {
					!e.contains::<Parent>()
						&& !e.contains::<Tooltip<_T<true>>>()
						&& !e.contains::<Tooltip<_T<false>>>()
						&& !e.contains::<_Window>()
				})
				.last()
		};
	}

	macro_rules! get_latest_container {
		($app:expr) => {
			try_get_latest_container!($app).expect("no additional top level component spawned")
		};
	}

	macro_rules! get_first_child {
		($app:expr, $parent:expr) => {{
			let child = $parent
				.get::<Children>()
				.and_then(|c| c.deref().first())
				.unwrap_or_else(|| panic!("{:?} does not have a child ", $parent.id()));
			$app.world.entity(*child)
		}};
	}

	fn setup<const HAS_BACKGROUND_COLOR: bool>() -> App {
		let mut app = App::new().single_threaded(Update);
		if HAS_BACKGROUND_COLOR {
			app.add_systems(Update, tooltip::<_T<true>, _Window>);
		} else {
			app.add_systems(Update, tooltip::<_T<false>, _Window>);
		}

		app
	}

	#[test]
	fn spawn_container_node_bundle() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));

		app.update();

		let entity = get_latest_container!(app);

		assert_node_bundle!(entity);
	}

	#[test]
	fn spawn_container_on_mouse_position() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(Vec2 { x: 4., y: 2. })));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));

		app.update();

		let entity = get_latest_container!(app);

		assert_eq!(
			Some((Val::Px(4.), Val::Px(2.))),
			entity.get::<Style>().map(|s| (s.left, s.top))
		)
	}

	#[test]
	fn spawn_container_with_global_z_index_1() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(Vec2 { x: 4., y: 2. })));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));

		app.update();

		let entity = get_latest_container!(app);

		assert_eq!(
			Some(1),
			entity.get::<ZIndex>().map(|i| match i {
				ZIndex::Global(i) => *i,
				_ => -1,
			})
		)
	}

	#[test]
	fn spawn_tooltip_node_bundle() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));

		app.update();

		let container = get_latest_container!(app);
		let tooltip = get_first_child!(app, container);

		assert_node_bundle!(tooltip);
	}

	#[test]
	fn spawn_tooltip_with_tooltip_node() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle {
					style: Style {
						left: Val::Percent(42.),
						..default()
					},
					..default()
				},
			}),
			Interaction::Hovered,
		));

		app.update();

		let container = get_latest_container!(app);
		let tooltip = get_first_child!(app, container);

		assert_node_bundle!(tooltip);
		assert_eq!(
			Some(Val::Percent(42.)),
			tooltip.get::<Style>().map(|s| s.left)
		)
	}

	#[test]
	fn spawn_tooltip_with_tooltip_children() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));

		app.update();

		let container = get_latest_container!(app);
		let tooltip = get_first_child!(app, container);
		let content = get_first_child!(app, tooltip);

		assert_eq!(Some(&_Content("my content")), content.get::<_Content>())
	}

	#[test]
	fn do_not_spawn_when_not_hovering() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::None,
		));

		app.update();

		let container = try_get_latest_container!(app);

		assert!(container.is_none());
	}

	#[test]
	fn only_spawn_one_container() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));

		app.update();

		let fist = get_latest_container!(app).id();

		app.update();

		let latest = get_latest_container!(app).id();

		assert_eq!(fist, latest);
	}

	#[test]
	fn update_container_position() {
		let mut app = setup::<true>();
		let window = app.world.spawn(_Window(Some(default()))).id();
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));

		app.update();

		app.world.entity_mut(window).get_mut::<_Window>().unwrap().0 = Some(Vec2 { x: 4., y: 2. });

		app.update();

		let container = get_latest_container!(app);

		assert_eq!(
			Some((Val::Px(4.), Val::Px(2.))),
			container.get::<Style>().map(|s| (s.left, s.top))
		)
	}

	#[test]
	fn despawn_container_when_interaction_none() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		let tooltip = app
			.world
			.spawn((
				Tooltip(_T::<true> {
					content: "my content",
					node: NodeBundle::default(),
				}),
				Interaction::Hovered,
			))
			.id();

		app.update();

		let mut tooltip_entity = app.world.entity_mut(tooltip);
		let mut interaction = tooltip_entity.get_mut::<Interaction>().unwrap();
		*interaction = Interaction::None;

		app.update();

		let container = try_get_latest_container!(app);

		assert!(container.is_none());
	}

	#[test]
	fn despawn_container_when_interaction_pressed() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		let tooltip = app
			.world
			.spawn((
				Tooltip(_T::<true> {
					content: "my content",
					node: NodeBundle::default(),
				}),
				Interaction::Hovered,
			))
			.id();

		app.update();

		let mut tooltip_entity = app.world.entity_mut(tooltip);
		let mut interaction = tooltip_entity.get_mut::<Interaction>().unwrap();
		*interaction = Interaction::Pressed;

		app.update();

		let container = try_get_latest_container!(app);

		assert!(container.is_none());
	}

	#[test]
	fn spawn_container_again_after_despawn() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		let tooltip = app
			.world
			.spawn((
				Tooltip(_T::<true> {
					content: "my content",
					node: NodeBundle::default(),
				}),
				Interaction::Hovered,
			))
			.id();

		app.update();

		let mut tooltip_entity = app.world.entity_mut(tooltip);
		let mut interaction = tooltip_entity.get_mut::<Interaction>().unwrap();
		*interaction = Interaction::None;

		app.update();

		let mut tooltip_entity = app.world.entity_mut(tooltip);
		let mut interaction = tooltip_entity.get_mut::<Interaction>().unwrap();
		*interaction = Interaction::Hovered;

		app.update();

		let container = try_get_latest_container!(app);

		assert!(container.is_some());
	}

	#[test]
	fn spawn_tooltip_with_tooltip_children_when_multiple_tooltips_present() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content not hovered",
				node: NodeBundle::default(),
			}),
			Interaction::None,
		));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content hovered",
				node: NodeBundle::default(),
			}),
			Interaction::Hovered,
		));
		app.world.spawn((
			Tooltip(_T::<true> {
				content: "my content not hovered",
				node: NodeBundle::default(),
			}),
			Interaction::None,
		));

		app.update();

		let container = get_latest_container!(app);
		let tooltip = get_first_child!(app, container);
		let content = get_first_child!(app, tooltip);

		assert_eq!(
			Some(&_Content("my content hovered")),
			content.get::<_Content>()
		)
	}

	#[test]
	fn remove_container_when_tooltip_removed() {
		let mut app = setup::<true>();
		app.world.spawn(_Window(Some(default())));
		let tooltip = app
			.world
			.spawn((
				Tooltip(_T::<true> {
					content: "my content",
					node: NodeBundle::default(),
				}),
				Interaction::Hovered,
			))
			.id();

		app.update();

		app.world.entity_mut(tooltip).despawn();

		app.update();

		let container = try_get_latest_container!(app);

		assert!(container.is_none());
	}
}
