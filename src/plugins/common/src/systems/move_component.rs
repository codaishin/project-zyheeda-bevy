use crate::{
	components::Unmovable,
	traits::{push_component::PushComponent, try_remove_from::TryRemoveFrom},
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
		TTarget: Component + PushComponent<TComponent>,
	{
		for (entity, component, mut target) in &mut entities {
			target.push_component(entity, component.clone());
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
	fn add_source_to_target() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Source).id();

		app.world_mut()
			.entity_mut(entity)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_push_component()
					.times(1)
					.with(eq(entity), eq(_Source))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::move_into::<_Target>);
	}

	#[test]
	fn remove_source() {
		let mut app = setup();
		let target = _Target::new().with_mock(|mock: &mut Mock_Target| {
			mock.expect_push_component().return_const(());
		});
		let entity = app.world_mut().spawn((_Source, target)).id();

		app.world_mut()
			.run_system_once(_Source::move_into::<_Target>);

		assert_eq!(None, app.world().entity(entity).get::<_Source>());
	}

	#[test]
	fn do_nothing_when_source_unmovable() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Source, Unmovable::<_Source>::default()))
			.id();

		app.world_mut()
			.entity_mut(entity)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_push_component().never().return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::move_into::<_Target>);
	}
}
