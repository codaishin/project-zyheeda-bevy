use crate::{
	components::tooltip::Tooltip,
	traits::{
		get_node::GetNode,
		instantiate_content_on::InstantiateContentOn,
		tooltip_ui_control::{
			DespawnAllTooltips,
			DespawnOutdatedTooltips,
			SpawnTooltips,
			UpdateTooltipPosition,
		},
	},
};
use bevy::{
	ecs::system::Res,
	prelude::{Changed, Commands, Component, Entity, Query, RemovedComponents, Resource},
	ui::{Interaction, Style},
};
use common::traits::mouse_position::MousePosition;

pub(crate) fn tooltip<T, TUI, TUIControl, TWindow>(
	mut commands: Commands,
	ui_control: Res<TUIControl>,
	windows: Query<&TWindow>,
	changed_tooltip_interactions: Query<(Entity, &Tooltip<T>, &Interaction), Changed<Interaction>>,
	mut tooltip_uis: Query<(Entity, &TUI, &mut Style)>,
	removed_tooltips: RemovedComponents<Tooltip<T>>,
) where
	T: Sync + Send + 'static,
	Tooltip<T>: InstantiateContentOn + GetNode,
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
	use crate::traits::instantiate_content_on::InstantiateContentOn;
	use bevy::{
		app::{App, Update},
		hierarchy::ChildBuilder,
		math::Vec2,
		prelude::{default, Resource},
		ui::{node_bundles::NodeBundle, Style},
	};
	use common::test_tools::utils::SingleThreadedApp;
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

	#[derive(Clone, Copy)]
	struct _T {
		content: &'static str,
	}

	impl GetNode for Tooltip<_T> {
		fn node(&self) -> NodeBundle {
			todo!();
		}
	}

	impl InstantiateContentOn for Tooltip<_T> {
		fn instantiate_content_on(&self, _: &mut ChildBuilder) {
			todo!();
		}
	}

	#[derive(Component)]
	struct _UI;

	#[derive(Resource, Default)]
	struct _UIControl {
		mock: Mock_UIControl,
	}

	impl DespawnAllTooltips<_UI> for _UIControl {
		fn despawn_all(&self, uis: &Query<(Entity, &_UI, &mut Style)>, commands: &mut Commands)
		where
			_UI: Component + Sized,
		{
			self.mock.despawn_all(uis, commands)
		}
	}

	impl DespawnOutdatedTooltips<_UI, _T> for _UIControl {
		fn despawn_outdated(
			&self,
			uis: &Query<(Entity, &_UI, &mut Style)>,
			commands: &mut Commands,
			outdated_tooltips: RemovedComponents<Tooltip<_T>>,
		) where
			_UI: Component + Sized,
		{
			self.mock.despawn_outdated(uis, commands, outdated_tooltips)
		}
	}

	impl UpdateTooltipPosition<_UI> for _UIControl {
		fn update_position(&self, uis: &mut Query<(Entity, &_UI, &mut Style)>, position: Vec2)
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
			Tooltip<_T>: InstantiateContentOn + GetNode,
		{
			self.mock.spawn(commands, tooltip_entity, tooltip, position)
		}
	}

	mock! {
		_UIControl {}
		impl DespawnAllTooltips<_UI> for _UIControl {
			fn despawn_all<'a, 'b, 'c, 'd, 'e, 'f>(
				&self,
				uis: &Query<'a, 'b, (Entity, &'c _UI, &'d  mut Style)>,
				commands: & mut Commands<'e, 'f>
			) where
				Self: Component + Sized;
		}
		impl DespawnOutdatedTooltips<_UI, _T> for _UIControl {
			fn despawn_outdated<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h>(
				&self,
				uis: &Query<'a, 'b, (Entity, &'c  _UI, &'d  mut Style)>,
				commands: &mut Commands<'e, 'f>,
				outdated_tooltips: RemovedComponents<'g, 'h, Tooltip<_T>>,
			) where
				Self: Component + Sized;
		}
		impl UpdateTooltipPosition<_UI> for _UIControl {
			fn update_position<'a, 'b, 'c, 'd>(
				&self,
				uis: &mut Query<'a, 'b, (Entity, &'c _UI, &'d mut Style)>,
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
				Tooltip<_T>: InstantiateContentOn + GetNode;
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
			tooltip: &'static str,
			position: Vec2,
		}

		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_all().return_const(());
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control
			.mock
			.expect_spawn()
			.returning(|commands, entity, tooltip, position| {
				commands.insert_resource(_Spawn {
					entity,
					tooltip: tooltip.0.content,
					position,
				});
			});

		let mut app = setup(ui_control);
		app.world.spawn(_Window(Some(Vec2 { x: 33., y: 66. })));
		let tooltip_id = app
			.world
			.spawn((
				Tooltip(_T {
					content: "My Content",
				}),
				Interaction::Hovered,
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Spawn {
				entity: tooltip_id,
				tooltip: "My Content",
				position: Vec2 { x: 33., y: 66. }
			}),
			app.world.get_resource::<_Spawn>()
		);
	}

	#[test]
	fn do_not_call_spawn_when_not_hovering() {
		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_all().return_const(());
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control.mock.expect_spawn().never().return_const(());

		let mut app = setup(ui_control);
		app.world.spawn(_Window(Some(default())));
		app.world
			.spawn((Tooltip(_T { content: "" }), Interaction::None));

		app.update();
	}

	#[test]
	fn call_spawn_only_once() {
		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_all().return_const(());
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control.mock.expect_spawn().times(1).return_const(());

		let mut app = setup(ui_control);
		app.world.spawn(_Window(Some(default())));
		app.world
			.spawn((Tooltip(_T { content: "" }), Interaction::Hovered));

		app.update();
		app.update();
	}

	#[test]
	fn call_spawn_again_when_interaction_changed_to_hovered() {
		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_all().return_const(());
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control.mock.expect_spawn().times(1).return_const(());

		let mut app = setup(ui_control);
		app.world.spawn(_Window(Some(default())));
		let tooltip = app
			.world
			.spawn((Tooltip(_T { content: "" }), Interaction::None))
			.id();

		app.update();

		let mut tooltip = app.world.entity_mut(tooltip);
		let mut interaction = tooltip.get_mut::<Interaction>().unwrap();
		*interaction = Interaction::Hovered;

		app.update();
	}

	#[test]
	fn call_update_position() {
		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_all().return_const(());
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_spawn().return_const(());
		ui_control
			.mock
			.expect_update_position()
			.withf(|_, position| {
				assert_eq!(Vec2 { x: 33., y: 66. }, *position);
				true
			})
			.return_const(());

		let mut app = setup(ui_control);
		app.world.spawn(_Window(Some(Vec2 { x: 33., y: 66. })));

		app.update();
	}

	#[test]
	fn do_not_call_update_position_when_tooltips_changed() {
		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_all().return_const(());
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_spawn().return_const(());
		ui_control
			.mock
			.expect_update_position()
			.never()
			.return_const(());

		let mut app = setup(ui_control);
		app.world.spawn(_Window(Some(default())));
		app.world
			.spawn((Tooltip(_T { content: "" }), Interaction::Hovered));

		app.update();
	}

	#[test]
	fn call_despawn_all_when_tooltips_changed() {
		#[derive(Resource)]
		struct _DespawnAll;

		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_spawn().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control
			.mock
			.expect_despawn_all()
			.returning(|_, commands| commands.insert_resource(_DespawnAll));

		let mut app = setup(ui_control);

		app.world.spawn(_Window(Some(default())));
		app.world
			.spawn((Tooltip(_T { content: "" }), Interaction::Hovered));

		app.update();

		assert!(app.world.get_resource::<_DespawnAll>().is_some());
	}

	#[test]
	fn do_not_call_despawn_all_when_tooltips_did_not_changed() {
		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_despawn_outdated().return_const(());
		ui_control.mock.expect_spawn().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control
			.mock
			.expect_despawn_all()
			.never()
			.return_const(());

		let mut app = setup(ui_control);

		app.world.spawn(_Window(Some(default())));

		app.update();
	}

	#[test]
	fn call_despawn_outdated_when_tooltips_removed() {
		#[derive(Resource, Debug, PartialEq)]
		struct _DespawnOutdated(Vec<Entity>);

		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_spawn().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control.mock.expect_despawn_all().return_const(());

		let mut app = setup(ui_control);

		app.world.spawn(_Window(Some(default())));
		let tooltip = app
			.world
			.spawn((Tooltip(_T { content: "" }), Interaction::Hovered))
			.id();

		app.world
			.resource_mut::<_UIControl>()
			.mock
			.expect_despawn_outdated()
			.returning(|_, commands, mut removed_tooltips| {
				commands.insert_resource(_DespawnOutdated(removed_tooltips.read().collect()));
			});

		app.update();

		app.world.entity_mut(tooltip).despawn();

		app.update();

		assert_eq!(
			Some(&_DespawnOutdated(vec![tooltip])),
			app.world.get_resource::<_DespawnOutdated>()
		);
	}

	#[test]
	fn do_not_call_despawn_outdated_when_no_tooltip_removed() {
		let mut ui_control = _UIControl::default();
		ui_control.mock.expect_spawn().return_const(());
		ui_control.mock.expect_update_position().return_const(());
		ui_control.mock.expect_despawn_all().return_const(());
		ui_control
			.mock
			.expect_despawn_outdated()
			.never()
			.return_const(());

		let mut app = setup(ui_control);

		app.world.spawn(_Window(Some(default())));
		app.world
			.spawn((Tooltip(_T { content: "" }), Interaction::Hovered));

		app.update();
	}
}
