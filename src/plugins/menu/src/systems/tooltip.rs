use crate::{
	components::tooltip::{Tooltip, TooltipUiConfig},
	traits::{
		insert_ui_content::InsertUiContent,
		tooltip_ui_control::{
			DespawnAllTooltips,
			DespawnOutdatedTooltips,
			MouseVec2,
			SpawnTooltips,
			UpdateTooltipPosition,
		},
	},
};
use bevy::prelude::*;
use common::traits::{
	handles_localization::Localize,
	mouse_position::MousePosition,
	thread_safe::ThreadSafe,
};

pub(crate) fn tooltip<T, TLocalization, TUI, TUIControl, TWindow>(
	mut commands: Commands,
	localize: Res<TLocalization>,
	ui_control: Res<TUIControl>,
	windows: Query<&TWindow>,
	changed_tooltip_interactions: Query<(Entity, &Tooltip<T>, &Interaction), Changed<Interaction>>,
	mut tooltip_uis: Query<(Entity, &TUI, &mut Node, &ComputedNode)>,
	removed_tooltips: RemovedComponents<Tooltip<T>>,
) where
	T: TooltipUiConfig + ThreadSafe,
	Tooltip<T>: InsertUiContent,
	TLocalization: Localize + Resource,
	TUI: Component,
	TUIControl: Resource
		+ DespawnAllTooltips<TUI>
		+ DespawnOutdatedTooltips<TUI, T>
		+ UpdateTooltipPosition<TUI>
		+ SpawnTooltips<T, TLocalization>,
	TWindow: Component + MousePosition,
{
	let Ok(window) = windows.single() else {
		return;
	};
	let Some(position) = window.mouse_position() else {
		return;
	};

	if !changed_tooltip_interactions.is_empty() {
		ui_control.despawn_all(&tooltip_uis, &mut commands);
	} else {
		ui_control.update_position(&mut tooltip_uis, MouseVec2(position));
	}

	if !removed_tooltips.is_empty() {
		ui_control.despawn_outdated(&tooltip_uis, &mut commands, removed_tooltips);
	}

	for (entity, tooltip, _) in changed_tooltip_interactions.iter().filter(is_hovering) {
		ui_control.spawn(&mut commands, &localize, entity, tooltip);
	}
}

fn is_hovering<T>((.., interaction): &(Entity, &Tooltip<T>, &Interaction)) -> bool
where
	T: TooltipUiConfig + ThreadSafe,
	Tooltip<T>: InsertUiContent,
{
	interaction == &&Interaction::Hovered
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::tooltip::TooltipUiConfig, traits::insert_ui_content::InsertUiContent};
	use bevy::ecs::relationship::RelatedSpawnerCommands;
	use common::traits::handles_localization::{LocalizationResult, Localize, Token};
	use macros::NestedMocks;
	use mockall::mock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Window(Option<Vec2>);

	impl MousePosition for _Window {
		fn mouse_position(&self) -> Option<Vec2> {
			self.0
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Content(&'static str);

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _T {
		content: &'static str,
	}

	impl TooltipUiConfig for _T {}

	impl InsertUiContent for Tooltip<_T> {
		fn insert_ui_content<TLocalization>(
			&self,
			_: &TLocalization,
			_: &mut RelatedSpawnerCommands<ChildOf>,
		) {
		}
	}

	#[derive(Component)]
	struct _UI;

	#[derive(Resource, NestedMocks)]
	struct _UIControl {
		mock: Mock_UIControl,
	}

	impl DespawnAllTooltips<_UI> for _UIControl {
		fn despawn_all(
			&self,
			uis: &Query<(Entity, &_UI, &mut Node, &ComputedNode)>,
			commands: &mut Commands,
		) where
			_UI: Component + Sized,
		{
			self.mock.despawn_all(uis, commands)
		}
	}

	impl DespawnOutdatedTooltips<_UI, _T> for _UIControl {
		fn despawn_outdated(
			&self,
			uis: &Query<(Entity, &_UI, &mut Node, &ComputedNode)>,
			commands: &mut Commands,
			outdated_tooltips: RemovedComponents<Tooltip<_T>>,
		) where
			_UI: Component + Sized,
		{
			self.mock.despawn_outdated(uis, commands, outdated_tooltips)
		}
	}

	impl UpdateTooltipPosition<_UI> for _UIControl {
		fn update_position(
			&self,
			uis: &mut Query<(Entity, &_UI, &mut Node, &ComputedNode)>,
			position: MouseVec2,
		) where
			_UI: Component + Sized,
		{
			self.mock.update_position(uis, position)
		}
	}

	impl SpawnTooltips<_T, _Localize> for _UIControl {
		fn spawn(
			&self,
			commands: &mut Commands,
			localize: &_Localize,
			tooltip_entity: Entity,
			tooltip: &Tooltip<_T>,
		) where
			Tooltip<_T>: InsertUiContent,
		{
			self.mock.spawn(commands, localize, tooltip_entity, tooltip)
		}
	}

	#[derive(Resource, Default, Debug, PartialEq, Clone, Copy)]
	struct _Localize;

	impl Localize for _Localize {
		fn localize(&self, _: &Token) -> LocalizationResult {
			panic!("NOT USED")
		}
	}

	mock! {
		_UIControl {}
		impl DespawnAllTooltips<_UI> for _UIControl {
			fn despawn_all<'a, 'b, 'c, 'd, 'e, 'f, 'g>(
				&self,
				uis: &Query<'a, 'b, (Entity, &'c _UI, &'d  mut Node, &'e ComputedNode)>,
				commands: & mut Commands<'f, 'g>
			) where
				Self: Component + Sized;
		}
		impl DespawnOutdatedTooltips<_UI, _T> for _UIControl {
			fn despawn_outdated<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i>(
				&self,
				uis: &Query<'a, 'b, (Entity, &'c  _UI, &'d  mut Node, &'e ComputedNode)>,
				commands: &mut Commands<'f, 'g>,
				outdated_tooltips: RemovedComponents<'h, 'i, Tooltip<_T>>,
			) where
				Self: Component + Sized;
		}
		impl UpdateTooltipPosition<_UI> for _UIControl {
			fn update_position<'a, 'b, 'c, 'd, 'e>(
				&self,
				uis: &mut Query<'a, 'b, (Entity, &'c _UI, &'d mut Node, &'e ComputedNode)>,
				position: MouseVec2
			) where
				Self: Component + Sized;
		}
		impl SpawnTooltips<_T, _Localize> for _UIControl {
			fn spawn<'a, 'b>(
				&self,
				commands: &mut Commands<'a, 'b>,
				localize: & _Localize,
				entity: Entity,
				tooltip: &Tooltip<_T>,
			) where
				Tooltip<_T>: InsertUiContent;
		}
	}

	fn setup(ui_control: _UIControl) -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Localize>();
		app.insert_resource(ui_control);
		app.add_systems(Update, tooltip::<_T, _Localize, _UI, _UIControl, _Window>);

		app
	}

	#[test]
	fn call_spawn() {
		#[derive(Resource, Debug, PartialEq)]
		struct _Spawn {
			entity: Entity,
			tooltip: Tooltip<_T>,
			localize: _Localize,
		}

		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_spawn()
				.returning(|commands, localize, entity, tooltip| {
					commands.insert_resource(_Spawn {
						entity,
						tooltip: tooltip.clone(),
						localize: *localize,
					});
				});
		}));
		app.world_mut()
			.spawn(_Window(Some(Vec2 { x: 33., y: 66. })));
		let tooltip_id = app
			.world_mut()
			.spawn((
				Tooltip::new(_T {
					content: "My Content",
				}),
				Interaction::Hovered,
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Spawn {
				entity: tooltip_id,
				tooltip: Tooltip::new(_T {
					content: "My Content",
				}),
				localize: _Localize,
			}),
			app.world().get_resource::<_Spawn>()
		);
	}

	#[test]
	fn do_not_call_spawn_when_not_hovering() {
		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_spawn().never().return_const(());
		}));
		app.world_mut().spawn(_Window(Some(default())));
		app.world_mut()
			.spawn((Tooltip::new(_T { content: "" }), Interaction::None));

		app.update();
	}

	#[test]
	fn call_spawn_only_once() {
		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_spawn().times(1).return_const(());
		}));
		app.world_mut().spawn(_Window(Some(default())));
		app.world_mut()
			.spawn((Tooltip::new(_T { content: "" }), Interaction::Hovered));

		app.update();
		app.update();
	}

	#[test]
	fn call_spawn_again_when_interaction_changed_to_hovered() {
		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_spawn().times(1).return_const(());
		}));
		app.world_mut().spawn(_Window(Some(default())));
		let tooltip = app
			.world_mut()
			.spawn((Tooltip::new(_T { content: "" }), Interaction::None))
			.id();

		app.update();

		let mut tooltip = app.world_mut().entity_mut(tooltip);
		let mut interaction = tooltip.get_mut::<Interaction>().unwrap();
		*interaction = Interaction::Hovered;

		app.update();
	}

	#[test]
	fn call_update_position() {
		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().return_const(());
			mock.expect_spawn().return_const(());
			mock.expect_update_position()
				.withf(|_, MouseVec2(position)| {
					assert_eq!(Vec2 { x: 33., y: 66. }, *position);
					true
				})
				.return_const(());
		}));
		app.world_mut()
			.spawn(_Window(Some(Vec2 { x: 33., y: 66. })));

		app.update();
	}

	#[test]
	fn do_not_call_update_position_when_tooltips_changed() {
		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().return_const(());
			mock.expect_spawn().return_const(());
			mock.expect_update_position().never().return_const(());
		}));
		app.world_mut().spawn(_Window(Some(default())));
		app.world_mut()
			.spawn((Tooltip::new(_T { content: "" }), Interaction::Hovered));

		app.update();
	}

	#[test]
	fn call_despawn_all_when_tooltips_changed() {
		#[derive(Resource)]
		struct _DespawnAll;

		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_outdated().return_const(());
			mock.expect_spawn().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_despawn_all()
				.returning(|_, commands| commands.insert_resource(_DespawnAll));
		}));

		app.world_mut().spawn(_Window(Some(default())));
		app.world_mut()
			.spawn((Tooltip::new(_T { content: "" }), Interaction::Hovered));

		app.update();

		assert!(app.world().get_resource::<_DespawnAll>().is_some());
	}

	#[test]
	fn do_not_call_despawn_all_when_tooltips_did_not_changed() {
		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_outdated().return_const(());
			mock.expect_spawn().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_despawn_all().never().return_const(());
		}));

		app.world_mut().spawn(_Window(Some(default())));

		app.update();
	}

	#[test]
	fn call_despawn_outdated_when_tooltips_removed() {
		#[derive(Resource, Debug, PartialEq)]
		struct _DespawnOutdated(Vec<Entity>);

		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_spawn().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_despawn_all().return_const(());
		}));

		app.world_mut().spawn(_Window(Some(default())));
		let tooltip = app
			.world_mut()
			.spawn((Tooltip::new(_T { content: "" }), Interaction::Hovered))
			.id();

		app.world_mut()
			.resource_mut::<_UIControl>()
			.mock
			.expect_despawn_outdated()
			.returning(|_, commands, mut removed_tooltips| {
				commands.insert_resource(_DespawnOutdated(removed_tooltips.read().collect()));
			});

		app.update();

		app.world_mut().entity_mut(tooltip).despawn();

		app.update();

		assert_eq!(
			Some(&_DespawnOutdated(vec![tooltip])),
			app.world().get_resource::<_DespawnOutdated>()
		);
	}

	#[test]
	fn do_not_call_despawn_outdated_when_no_tooltip_removed() {
		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_spawn().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().never().return_const(());
		}));

		app.world_mut().spawn(_Window(Some(default())));
		app.world_mut()
			.spawn((Tooltip::new(_T { content: "" }), Interaction::Hovered));

		app.update();
	}
}
