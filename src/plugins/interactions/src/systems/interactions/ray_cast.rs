use crate::{events::RayCastEvent, traits::ActOn};
use bevy::ecs::{
	component::Component,
	entity::Entity,
	event::EventReader,
	system::{Commands, Query},
	world::Mut,
};
use common::components::ColliderRoot;

pub(crate) fn ray_cast_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	mut commands: Commands,
	mut ray_casts: EventReader<RayCastEvent>,
	mut actors: Query<&mut TActor>,
	mut targets: Query<&mut TTarget>,
	roots: Query<&ColliderRoot>,
) {
	let target_root_entity = |event: &RayCastEvent| {
		event
			.target
			.entity
			.and_then(|entity| Some((event.source, roots.get(entity).ok()?.0)))
	};

	for (source, target) in ray_casts.read().filter_map(target_root_entity) {
		handle_collision_interaction(source, target, &mut actors, &mut targets, &mut commands);
	}
}

fn handle_collision_interaction<TActor: ActOn<TTarget> + Component, TTarget: Component>(
	src: Entity,
	tgt: Entity,
	actors: &mut Query<&mut TActor>,
	targets: &mut Query<&mut TTarget>,
	commands: &mut Commands,
) {
	let Some((mut actor, mut target)) = get_actor_and_target(src, tgt, actors, targets) else {
		return;
	};
	actor.act_on(&mut target);
	commands.entity(src).remove::<TActor>();
}

fn get_actor_and_target<'a, TActor: Component, TTarget: Component>(
	src: Entity,
	tgt: Entity,
	actors: &'a mut Query<&mut TActor>,
	targets: &'a mut Query<&mut TTarget>,
) -> Option<(Mut<'a, TActor>, Mut<'a, TTarget>)> {
	let actor = actors.get_mut(src).ok()?;
	let target = targets.get_mut(tgt).ok()?;

	Some((actor, target))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::RayCastTarget;
	use bevy::{
		app::{App, Update},
		math::{Ray3d, Vec3},
	};
	use common::traits::cast_ray::TimeOfImpact;
	use mockall::{automock, predicate::eq};

	#[derive(Component, Default)]
	pub struct _Actor {
		mock: Mock_Actor,
	}

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	pub struct _Target;

	#[automock]
	impl ActOn<_Target> for _Actor {
		fn act_on(&mut self, target: &mut _Target) {
			self.mock.act_on(target)
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.add_event::<RayCastEvent>();
		app.add_systems(Update, ray_cast_interaction::<_Actor, _Target>);

		app
	}

	#[test]
	fn act_on_target() {
		let mut app = setup();
		let mut actor = _Actor::default();
		let target = _Target;
		actor
			.mock
			.expect_act_on()
			.times(1)
			.with(eq(target))
			.return_const(());

		let actor = app.world.spawn(actor).id();
		let target = app.world.spawn(target).id();
		let coll_target = app.world.spawn(ColliderRoot(target)).id();

		app.world.send_event(RayCastEvent {
			source: actor,
			target: RayCastTarget {
				entity: Some(coll_target),
				ray: Ray3d::new(Vec3::ZERO, Vec3::NEG_Z),
				toi: TimeOfImpact::default(),
			},
		});

		app.update();
	}

	#[test]
	fn remove_actor() {
		let mut app = setup();
		let mut actor = _Actor::default();
		let target = _Target;
		actor.mock.expect_act_on().return_const(());

		let actor = app.world.spawn(actor).id();
		let target = app.world.spawn(target).id();
		let coll_target = app.world.spawn(ColliderRoot(target)).id();

		app.world.send_event(RayCastEvent {
			source: actor,
			target: RayCastTarget {
				entity: Some(coll_target),
				ray: Ray3d::new(Vec3::ZERO, Vec3::NEG_Z),
				toi: TimeOfImpact::default(),
			},
		});

		app.update();

		let actor = app.world.entity(actor);

		assert!(!actor.contains::<_Actor>());
	}
}
