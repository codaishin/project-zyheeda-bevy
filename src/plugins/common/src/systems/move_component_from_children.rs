use crate::{
	components::Unmovable,
	traits::{push_component::PushComponent, try_remove_from::TryRemoveFrom},
};
use bevy::prelude::*;

pub trait MoveFromChildrenInto<TComponent>
where
	TComponent: Component + Clone,
{
	fn move_from_children_into<TTarget>(
		mut commands: Commands,
		mut targets: Query<(Entity, Mut<TTarget>)>,
		sources: Query<&TComponent, Without<Unmovable<TComponent>>>,
		children: Query<&Children>,
	) where
		TTarget: Component + PushComponent<TComponent>,
	{
		for (entity, target) in &mut targets {
			get_components(&mut commands, entity, target, &children, &sources);
		}
	}
}

fn get_components<TComponent, TTarget>(
	commands: &mut Commands,
	entity: Entity,
	mut target: Mut<TTarget>,
	children: &Query<&Children>,
	sources: &Query<&TComponent, Without<Unmovable<TComponent>>>,
) where
	TComponent: Component + Clone,
	TTarget: PushComponent<TComponent>,
{
	for child in children.iter_descendants(entity) {
		let Ok(source) = sources.get(child) else {
			continue;
		};
		target.push_component(child, source.clone());
		commands.try_remove_from::<TComponent>(child);
	}
}

impl<TComponent> MoveFromChildrenInto<TComponent> for TComponent where TComponent: Component + Clone {}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Unmovable;
	use bevy::ecs::system::RunSystemOnce;
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, Debug, Default, NestedMocks)]
	struct _Target {
		mock: Mock_Target,
	}

	#[automock]
	impl PushComponent<_Source> for _Target {
		fn push_component(&mut self, entity: Entity, component: _Source) {
			self.mock.push_component(entity, component);
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Source;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn clone_source_from_child_into_target() {
		let mut app = setup();
		let target = app.world_mut().spawn_empty().id();
		let source = app.world_mut().spawn(_Source).set_parent(target).id();

		app.world_mut()
			.entity_mut(target)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_push_component()
					.times(1)
					.with(eq(source), eq(_Source))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::move_from_children_into::<_Target>);
	}

	#[test]
	fn remove_source_from_child() {
		let mut app = setup();
		let target = _Target::new().with_mock(|mock: &mut Mock_Target| {
			mock.expect_push_component().return_const(());
		});
		let target = app.world_mut().spawn(target).id();
		let child = app.world_mut().spawn(_Source).set_parent(target).id();

		app.world_mut()
			.run_system_once(_Source::move_from_children_into::<_Target>);

		assert_eq!(None, app.world().entity(child).get::<_Source>());
	}

	#[test]
	fn clone_source_from_deep_child_into_target() {
		let mut app = setup();
		let target = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn_empty().set_parent(target).id();
		let deep_child = app.world_mut().spawn(_Source).set_parent(child).id();

		app.world_mut()
			.entity_mut(target)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_push_component()
					.times(1)
					.with(eq(deep_child), eq(_Source))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::move_from_children_into::<_Target>);
	}

	#[test]
	fn do_nothing_when_source_unmovable() {
		let mut app = setup();
		let target = app.world_mut().spawn_empty().id();
		app.world_mut()
			.spawn((_Source, Unmovable::<_Source>::default()))
			.set_parent(target);

		app.world_mut()
			.entity_mut(target)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_push_component().never().return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::move_from_children_into::<_Target>);
	}
}
