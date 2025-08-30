pub(crate) mod string;

use super::GlobalZIndexTop;
use crate::traits::{
	insert_ui_content::InsertUiContent,
	tooltip_ui_control::{
		DespawnAllTooltips,
		DespawnOutdatedTooltips,
		MouseVec2,
		SpawnTooltips,
		UpdateTooltipPosition,
	},
};
use bevy::prelude::*;
use common::traits::{handles_localization::LocalizeToken, thread_safe::ThreadSafe};
use std::{marker::PhantomData, time::Duration};

#[derive(Component, Debug, PartialEq, Clone)]
#[require(Node, Interaction)]
pub(crate) struct Tooltip<T>(T);

pub(crate) trait TooltipUiConfig {
	fn node() -> Node {
		Node::default()
	}

	fn background_color() -> BackgroundColor {
		BackgroundColor::default()
	}
}

impl<T> Tooltip<T>
where
	T: TooltipUiConfig,
	Tooltip<T>: InsertUiContent,
{
	pub(crate) fn new(value: T) -> Self {
		Tooltip(value)
	}

	#[cfg(debug_assertions)]
	pub(crate) fn value(&self) -> &T {
		&self.0
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(
	Node = T::node(),
	BackgroundColor = T::background_color(),
	Visibility = Visibility::Hidden,
	GlobalZIndexTop,
)]
pub(crate) struct TooltipContent<T>
where
	T: TooltipUiConfig,
{
	phantom_data: PhantomData<T>,
	pub(crate) source: Entity,
	pub(crate) delay: Duration,
}

impl<T> TooltipContent<T>
where
	T: TooltipUiConfig,
{
	pub(crate) fn new(source: Entity, delay: Duration) -> Self {
		Self {
			source,
			delay,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Resource, Default)]
pub(crate) struct TooltipUIControl {
	pub(crate) tooltip_delay: Duration,
}

impl<T> DespawnAllTooltips<TooltipContent<T>> for TooltipUIControl
where
	T: TooltipUiConfig + ThreadSafe,
{
	fn despawn_all(
		&self,
		uis: &Query<(Entity, &TooltipContent<T>, &mut Node, &ComputedNode)>,
		commands: &mut Commands,
	) {
		for (entity, ..) in uis {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn();
		}
	}
}

impl<T> DespawnOutdatedTooltips<TooltipContent<T>, T> for TooltipUIControl
where
	T: TooltipUiConfig + ThreadSafe,
{
	fn despawn_outdated(
		&self,
		uis: &Query<(Entity, &TooltipContent<T>, &mut Node, &ComputedNode)>,
		commands: &mut Commands,
		mut outdated_tooltips: RemovedComponents<Tooltip<T>>,
	) {
		let outdated = outdated_tooltips.read().collect::<Vec<_>>();
		let is_outdated = |(_, ui, ..): &(Entity, &TooltipContent<T>, &Node, &ComputedNode)| {
			outdated.contains(&ui.source)
		};

		for (entity, ..) in uis.iter().filter(is_outdated) {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn();
		}
	}
}

impl<T> UpdateTooltipPosition<TooltipContent<T>> for TooltipUIControl
where
	T: TooltipUiConfig + ThreadSafe,
{
	fn update_position(
		&self,
		uis: &mut Query<(Entity, &TooltipContent<T>, &mut Node, &ComputedNode)>,
		MouseVec2(position): MouseVec2,
	) {
		for (.., mut node, computed) in uis.iter_mut() {
			let height = computed.size.y;
			node.left = Val::Px(position.x);
			node.top = Val::Px(position.y - height);
		}
	}
}

impl<T, TLocalization> SpawnTooltips<T, TLocalization> for TooltipUIControl
where
	T: TooltipUiConfig + ThreadSafe,
	Tooltip<T>: InsertUiContent,
	TLocalization: LocalizeToken + ThreadSafe,
{
	fn spawn(
		&self,
		commands: &mut Commands,
		localize: &TLocalization,
		tooltip_entity: Entity,
		tooltip: &Tooltip<T>,
	) {
		commands
			.spawn(TooltipContent::<T>::new(tooltip_entity, self.tooltip_delay))
			.with_children(|container_node| {
				tooltip.insert_ui_content(localize, container_node);
			});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::relationship::RelatedSpawnerCommands;
	use common::traits::handles_localization::{LocalizationResult, Token, localized::Localized};
	use testing::{SingleThreadedApp, assert_count, get_children};

	impl TooltipUiConfig for () {}

	#[derive(Resource, Default)]
	struct _Localize;

	impl LocalizeToken for _Localize {
		fn localize_token<TToken>(&self, token: TToken) -> LocalizationResult
		where
			TToken: Into<Token> + 'static,
		{
			let Token(token) = token.into();
			LocalizationResult::Ok(Localized::from(format!("Token: {token}")))
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Child(Localized);

	impl From<&str> for _Child {
		fn from(value: &str) -> Self {
			Self(Localized::from(value))
		}
	}

	fn new_app() -> App {
		App::new().single_threaded(Update)
	}

	fn setup_despawn_all<T>() -> App
	where
		T: TooltipUiConfig + ThreadSafe,
	{
		let mut app = new_app();
		app.add_systems(
			Update,
			|uis: Query<(Entity, &TooltipContent<T>, &mut Node, &ComputedNode)>,
			 mut commands: Commands| {
				TooltipUIControl::default().despawn_all(&uis, &mut commands);
			},
		);

		app
	}

	fn setup_despawn_outdated<T>() -> App
	where
		T: TooltipUiConfig + Clone + ThreadSafe,
	{
		let mut app = new_app();
		app.add_systems(
			Update,
			|uis: Query<(Entity, &TooltipContent<T>, &mut Node, &ComputedNode)>,
			 mut commands: Commands,
			 outdated_tooltips: RemovedComponents<Tooltip<T>>| {
				TooltipUIControl::default().despawn_outdated(
					&uis,
					&mut commands,
					outdated_tooltips,
				);
			},
		);

		app
	}

	fn setup_update_position<T>(position: MouseVec2) -> App
	where
		T: TooltipUiConfig + ThreadSafe,
	{
		let mut app = new_app();
		app.add_systems(
			Update,
			move |mut uis: Query<(Entity, &TooltipContent<T>, &mut Node, &ComputedNode)>| {
				TooltipUIControl::default().update_position(&mut uis, position);
			},
		);

		app
	}

	#[derive(Clone, Debug, PartialEq)]
	struct _T {
		content: &'static str,
	}

	impl TooltipUiConfig for _T {}

	impl InsertUiContent for Tooltip<_T> {
		fn insert_ui_content<TLocalization>(
			&self,
			localize: &TLocalization,
			parent: &mut RelatedSpawnerCommands<ChildOf>,
		) where
			TLocalization: LocalizeToken,
		{
			let label = localize
				.localize_token(self.0.content)
				.or(|_| String::from("???"));
			parent.spawn(_Child(label));
		}
	}

	fn setup_spawn(tooltip_delay: Duration) -> App {
		let mut app = new_app();

		app.init_resource::<_Localize>();
		app.add_systems(
			Update,
			move |mut commands: Commands,
			      mut localize: ResMut<_Localize>,
			      tooltips: Query<(Entity, &Tooltip<_T>)>| {
				for (entity, tooltip) in &tooltips {
					TooltipUIControl { tooltip_delay }.spawn(
						&mut commands,
						localize.as_mut(),
						entity,
						tooltip,
					);
				}
			},
		);

		app
	}

	#[test]
	fn despawn_all() {
		let mut app = setup_despawn_all::<()>();
		app.world_mut().spawn_batch([
			(
				TooltipContent::<()>::new(Entity::from_raw(100), default()),
				Node::default(),
			),
			(
				TooltipContent::<()>::new(Entity::from_raw(200), default()),
				Node::default(),
			),
		]);

		app.update();

		let tooltip_uis = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<TooltipContent<()>>());

		assert_eq!(0, tooltip_uis.count());
	}

	#[test]
	fn despawn_all_recursively() {
		let mut app = setup_despawn_all::<()>();
		app.world_mut()
			.spawn((
				TooltipContent::<()>::new(Entity::from_raw(100), default()),
				Node::default(),
			))
			.with_children(|parent| {
				parent.spawn(_Child::from(""));
			});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Child>());

		assert_eq!(0, children.count());
	}

	#[test]
	fn despawn_outdated() {
		let mut app = setup_despawn_outdated::<&'static str>();
		let tooltips = [
			app.world_mut().spawn(Tooltip("1")).id(),
			app.world_mut().spawn(Tooltip("2")).id(),
		];
		for entity in tooltips {
			app.world_mut().spawn((
				TooltipContent::<&'static str>::new(entity, default()),
				Node::default(),
			));
		}

		app.update();

		for tooltip in tooltips {
			app.world_mut().entity_mut(tooltip).despawn();
		}

		app.update();

		let tooltip_uis = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<TooltipContent<&'static str>>());

		assert_eq!(0, tooltip_uis.count());
	}

	#[test]
	fn despawn_outdated_recursively() {
		let mut app = setup_despawn_outdated::<&'static str>();
		let tooltips = [
			app.world_mut().spawn(Tooltip("1")).id(),
			app.world_mut().spawn(Tooltip("2")).id(),
		];
		for entity in tooltips {
			app.world_mut()
				.spawn((
					TooltipContent::<&'static str>::new(entity, default()),
					Node::default(),
				))
				.with_children(|parent| {
					parent.spawn(_Child::from(""));
				});
		}

		app.update();

		for tooltip in tooltips {
			app.world_mut().entity_mut(tooltip).despawn();
		}

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<_Child>());

		assert_eq!(0, children.count());
	}

	#[test]
	fn do_not_despawn_when_not_outdated() {
		let mut app = setup_despawn_outdated::<&'static str>();
		let tooltips = [
			app.world_mut().spawn(Tooltip("1")).id(),
			app.world_mut().spawn(Tooltip("2")).id(),
		];
		for entity in tooltips {
			app.world_mut().spawn((
				TooltipContent::<&'static str>::new(entity, default()),
				Node::default(),
			));
		}

		app.update();

		let tooltip_uis = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<TooltipContent<&'static str>>());

		assert_eq!(2, tooltip_uis.count());
	}

	#[test]
	fn update_position() {
		let mut app = setup_update_position::<&'static str>(MouseVec2(Vec2 { x: 42., y: 11. }));
		let uis = app
			.world_mut()
			.spawn_batch([
				(
					TooltipContent::<&'static str>::new(Entity::from_raw(100), default()),
					Node::default(),
				),
				(
					TooltipContent::<&'static str>::new(Entity::from_raw(200), default()),
					Node::default(),
				),
			])
			.collect::<Vec<_>>();

		app.update();

		let tooltip_styles = uis
			.iter()
			.map(|entity| app.world().entity(*entity).get::<Node>())
			.collect::<Vec<_>>();

		assert_eq!(
			vec![
				Some(&Node {
					left: Val::Px(42.),
					top: Val::Px(11.),
					..default()
				}),
				Some(&Node {
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
		let mut app = setup_spawn(default());
		app.world_mut().spawn(Tooltip(_T { content: "" }));

		app.update();

		let tooltip_ui = app
			.world()
			.iter_entities()
			.find(|e| e.contains::<TooltipContent<_T>>())
			.expect("no tooltip spawned");
		assert_eq!(Some(&Visibility::Hidden), tooltip_ui.get::<Visibility>());
	}

	#[test]
	fn spawn_content_of_tooltip() {
		let mut app = setup_spawn(default());
		app.world_mut().spawn(Tooltip(_T {
			content: "My Content",
		}));

		app.update();

		let containers = app
			.world()
			.iter_entities()
			.filter(|e| e.contains::<TooltipContent<_T>>());
		let [container] = assert_count!(1, containers);
		let [content] = assert_count!(1, get_children!(app, container.id()));
		assert_eq!(
			Some(&_Child::from("Token: My Content")),
			content.get::<_Child>(),
		);
	}

	#[test]
	fn spawn_tooltip_ui_with_source_reference() {
		let mut app = setup_spawn(default());
		let tooltip = app.world_mut().spawn(Tooltip(_T { content: "" })).id();

		app.update();

		let tooltip_ui = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<TooltipContent<_T>>())
			.expect("no tooltip spawned");
		assert_eq!(tooltip, tooltip_ui.source);
	}

	#[test]
	fn spawn_tooltip_ui_with_delay() {
		let mut app = setup_spawn(Duration::from_secs(4000));
		app.world_mut().spawn(Tooltip(_T { content: "" }));

		app.update();

		let tooltip_ui = app
			.world()
			.iter_entities()
			.find_map(|e| e.get::<TooltipContent<_T>>())
			.expect("no tooltip spawned");
		assert_eq!(Duration::from_secs(4000), tooltip_ui.delay);
	}
}
