use crate::{
	components::Unmovable,
	traits::{push::Push, try_remove_from::TryRemoveFrom},
};
use bevy::prelude::*;

type Components<'a, TComponent, TTarget> = (Entity, &'a TComponent, Mut<'a, TTarget>);

pub trait MoveInto<TComponent>
where
	TComponent: Component + Clone,
{
	fn move_into<TTarget>(
		mut commands: Commands,
		mut entities: Query<Components<TComponent, TTarget>, Without<Unmovable<TComponent>>>,
	) where
		TTarget: Component + Push<TComponent>,
	{
		for (entity, component, mut target) in &mut entities {
			target.push(component.clone());
			commands.try_remove_from::<TComponent>(entity);
		}
	}
}

impl<TComponent> MoveInto<TComponent> for TComponent where TComponent: Component + Clone {}

#[cfg(test)]
mod tests {
	use crate::components::Unmovable;

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
	fn add_source_to_target() {
		let mut app = setup();
		let target = _Target::new().with_mock(assert);
		app.world_mut().spawn((_Source, target));

		app.world_mut()
			.run_system_once(_Source::move_into::<_Target>);

		fn assert(mock: &mut Mock_Target) {
			mock.expect_push()
				.times(1)
				.with(eq(_Source))
				.return_const(());
		}
	}

	#[test]
	fn remove_source() {
		let mut app = setup();
		let target = _Target::new().with_mock(|mock: &mut Mock_Target| {
			mock.expect_push().return_const(());
		});
		let entity = app.world_mut().spawn((_Source, target)).id();

		app.world_mut()
			.run_system_once(_Source::move_into::<_Target>);

		assert_eq!(None, app.world().entity(entity).get::<_Source>());
	}

	#[test]
	fn do_nothing_when_source_unmovable() {
		let mut app = setup();
		let target = _Target::new().with_mock(assert);
		app.world_mut()
			.spawn((_Source, target, Unmovable::<_Source>::default()));

		app.world_mut()
			.run_system_once(_Source::move_into::<_Target>);

		fn assert(mock: &mut Mock_Target) {
			mock.expect_push().never().return_const(());
		}
	}
}
