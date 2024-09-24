use crate::traits::{push::Push, try_remove_from::TryRemoveFrom};
use bevy::prelude::*;

pub trait MoveFromChildrenInto<TComponent>
where
	TComponent: Component + Clone,
{
	fn move_from_children_into<TTarget>(
		mut commands: Commands,
		mut targets: Query<(Entity, Mut<TTarget>)>,
		sources: Query<&TComponent>,
		children: Query<&Children>,
	) where
		TTarget: Component + Push<TComponent>,
	{
		for (entity, target) in &mut targets {
			get_components(&mut commands, entity, target, &children, &sources);
		}
	}
}

fn get_components<TComponent, TTarget: Push<TComponent>>(
	commands: &mut Commands,
	entity: Entity,
	mut target: Mut<TTarget>,
	children: &Query<&Children>,
	sources: &Query<&TComponent>,
) where
	TComponent: Component + Clone,
{
	for child in children.iter_descendants(entity) {
		let Ok(source) = sources.get(child) else {
			continue;
		};
		target.push(source.clone());
		commands.try_remove_from::<TComponent>(child);
	}
}

impl<TComponent> MoveFromChildrenInto<TComponent> for TComponent where TComponent: Component + Clone {}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, Debug, Default, NestedMocks)]
	struct _Target {
		mock: Mock_Target,
	}

	#[automock]
	impl Push<_Source> for _Target {
		fn push(&mut self, value: _Source) {
			self.mock.push(value);
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
		let target = _Target::new().with_mock(assert);
		let target = app.world_mut().spawn(target).id();
		app.world_mut().spawn(_Source).set_parent(target);

		app.world_mut()
			.run_system_once(_Source::move_from_children_into::<_Target>);

		fn assert(mock: &mut Mock_Target) {
			mock.expect_push()
				.times(1)
				.with(eq(_Source))
				.return_const(());
		}
	}

	#[test]
	fn remove_source_from_child() {
		let mut app = setup();
		let target = _Target::new().with_mock(|mock: &mut Mock_Target| {
			mock.expect_push().return_const(());
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
		let target = _Target::new().with_mock(assert);
		let target = app.world_mut().spawn(target).id();
		let child = app.world_mut().spawn_empty().set_parent(target).id();
		app.world_mut().spawn(_Source).set_parent(child);

		app.world_mut()
			.run_system_once(_Source::move_from_children_into::<_Target>);

		fn assert(mock: &mut Mock_Target) {
			mock.expect_push()
				.times(1)
				.with(eq(_Source))
				.return_const(());
		}
	}
}
