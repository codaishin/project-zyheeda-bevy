use crate::{
	components::tooltip::Tooltip,
	traits::{
		tooltip_ui_control::{
			DespawnAllTooltips,
			DespawnOutdatedTooltips,
			SpawnTooltips,
			UpdateTooltipPosition,
		},
		ui_components::GetUIComponents,
		update_children::UpdateChildren,
	},
};
use bevy::prelude::*;
use common::traits::mouse_position::MousePosition;

pub(crate) fn tooltip<T, TUI, TUIControl, TWindow>(
	mut commands: Commands,
	ui_control: Res<TUIControl>,
	windows: Query<&TWindow>,
	changed_tooltip_interactions: Query<(Entity, &Tooltip<T>, &Interaction), Changed<Interaction>>,
	mut tooltip_uis: Query<(Entity, &TUI, &mut Node)>,
	removed_tooltips: RemovedComponents<Tooltip<T>>,
) where
	T: Sync + Send + 'static,
	Tooltip<T>: UpdateChildren + GetUIComponents,
	TUI: Component,
	TUIControl: Resource
		+ DespawnAllTooltips<TUI>
		+ DespawnOutdatedTooltips<TUI, T>
		+ UpdateTooltipPosition<TUI>
		+ SpawnTooltips<T>,
	TWindow: Component + MousePosition,
{
	let Ok(window) = windows.get_single() else {
		return;
	};
	let Some(position) = window.mouse_position() else {
		return;
	};

	if !changed_tooltip_interactions.is_empty() {
		ui_control.despawn_all(&tooltip_uis, &mut commands);
	} else {
		ui_control.update_position(&mut tooltip_uis, position);
	}

	if !removed_tooltips.is_empty() {
		ui_control.despawn_outdated(&tooltip_uis, &mut commands, removed_tooltips);
	}

	for (entity, tooltip, _) in changed_tooltip_interactions.iter().filter(is_hovering) {
		ui_control.spawn(&mut commands, entity, tooltip, position);
	}
}

fn is_hovering<T: Sync + Send + 'static>(
	(.., interaction): &(Entity, &Tooltip<T>, &Interaction),
) -> bool {
	interaction == &&Interaction::Hovered
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::update_children::UpdateChildren;
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::mock;

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

	impl GetUIComponents for Tooltip<_T> {
		fn ui_components(&self) -> (Node, BackgroundColor) {
			todo!();
		}
	}

	impl UpdateChildren for Tooltip<_T> {
		fn update_children(&self, _: &mut ChildBuilder) {
			todo!();
		}
	}

	#[derive(Component)]
	struct _UI;

	#[derive(Resource, NestedMocks)]
	struct _UIControl {
		mock: Mock_UIControl,
	}

	impl DespawnAllTooltips<_UI> for _UIControl {
		fn despawn_all(&self, uis: &Query<(Entity, &_UI, &mut Node)>, commands: &mut Commands)
		where
			_UI: Component + Sized,
		{
			self.mock.despawn_all(uis, commands)
		}
	}

	impl DespawnOutdatedTooltips<_UI, _T> for _UIControl {
		fn despawn_outdated(
			&self,
			uis: &Query<(Entity, &_UI, &mut Node)>,
			commands: &mut Commands,
			outdated_tooltips: RemovedComponents<Tooltip<_T>>,
		) where
			_UI: Component + Sized,
		{
			self.mock.despawn_outdated(uis, commands, outdated_tooltips)
		}
	}

	impl UpdateTooltipPosition<_UI> for _UIControl {
		fn update_position(&self, uis: &mut Query<(Entity, &_UI, &mut Node)>, position: Vec2)
		where
			_UI: Component + Sized,
		{
			self.mock.update_position(uis, position)
		}
	}

	impl SpawnTooltips<_T> for _UIControl {
		fn spawn(
			&self,
			commands: &mut Commands,
			tooltip_entity: Entity,
			tooltip: &Tooltip<_T>,
			position: Vec2,
		) where
			Tooltip<_T>: UpdateChildren + GetUIComponents,
		{
			self.mock.spawn(commands, tooltip_entity, tooltip, position)
		}
	}

	mock! {
		_UIControl {}
		impl DespawnAllTooltips<_UI> for _UIControl {
			fn despawn_all<'a, 'b, 'c, 'd, 'e, 'f>(
				&self,
				uis: &Query<'a, 'b, (Entity, &'c _UI, &'d  mut Node)>,
				commands: & mut Commands<'e, 'f>
			) where
				Self: Component + Sized;
		}
		impl DespawnOutdatedTooltips<_UI, _T> for _UIControl {
			fn despawn_outdated<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h>(
				&self,
				uis: &Query<'a, 'b, (Entity, &'c  _UI, &'d  mut Node)>,
				commands: &mut Commands<'e, 'f>,
				outdated_tooltips: RemovedComponents<'g, 'h, Tooltip<_T>>,
			) where
				Self: Component + Sized;
		}
		impl UpdateTooltipPosition<_UI> for _UIControl {
			fn update_position<'a, 'b, 'c, 'd>(
				&self,
				uis: &mut Query<'a, 'b, (Entity, &'c _UI, &'d mut Node)>,
				position: Vec2
			) where
				Self: Component + Sized;
		}
		impl SpawnTooltips<_T> for _UIControl {
			fn spawn<'a, 'b>(
				&self,
				commands: &mut Commands<'a, 'b>,
				entity: Entity,
				tooltip: &Tooltip<_T>,
				position: Vec2
			) where
				Tooltip<_T>: UpdateChildren + GetUIComponents;
		}
	}

	fn setup(ui_control: _UIControl) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(ui_control);
		app.add_systems(Update, tooltip::<_T, _UI, _UIControl, _Window>);

		app
	}

	#[test]
	fn call_spawn() {
		#[derive(Resource, Debug, PartialEq)]
		struct _Spawn {
			entity: Entity,
			tooltip: Tooltip<_T>,
			position: Vec2,
		}

		let mut app = setup(_UIControl::new().with_mock(|mock| {
			mock.expect_despawn_all().return_const(());
			mock.expect_despawn_outdated().return_const(());
			mock.expect_update_position().return_const(());
			mock.expect_spawn()
				.returning(|commands, entity, tooltip, position| {
					commands.insert_resource(_Spawn {
						entity,
						tooltip: tooltip.clone(),
						position,
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
				position: Vec2 { x: 33., y: 66. }
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
				.withf(|_, position| {
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
