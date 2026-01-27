use crate::components::followers::{FollowStateDirty, FollowTransform, Followers};
use bevy::prelude::*;

impl Followers {
	pub(crate) fn mark_dirty(
		mut followed: Query<(Ref<Self>, Ref<Transform>, &mut FollowStateDirty)>,
		changed_follow_transforms: Query<(), Changed<FollowTransform>>,
	) {
		for (followed, transform, mut state_dirty) in &mut followed {
			if !Self::is_dirty(followed, transform, &changed_follow_transforms) {
				continue;
			}

			state_dirty.set_changed();
		}
	}

	fn is_dirty(
		followed: Ref<Followers>,
		transform: Ref<Transform>,
		changed_follow_transforms: &Query<(), Changed<FollowTransform>>,
	) -> bool {
		if followed.is_changed() || transform.is_changed() {
			return true;
		}

		for entity in followed.iter() {
			if changed_follow_transforms.contains(entity) {
				return true;
			}
		}

		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::followers::{Follow, FollowTransform};
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(Followers::mark_dirty, IsChanged::<FollowStateDirty>::detect).chain(),
		);

		app
	}

	#[test]
	fn mark_dirty_if_followed_changed() {
		let mut app = setup();
		let entity = app.world_mut().spawn(related!(Followers[()])).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Followers>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(entity)
				.get::<IsChanged::<FollowStateDirty>>(),
		);
	}

	#[test]
	fn do_not_mark_dirty_if_followed_did_not_change() {
		let mut app = setup();
		let entity = app.world_mut().spawn(related!(Followers[()])).id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged::<FollowStateDirty>>(),
		);
	}

	#[test]
	fn mark_dirty_if_followed_transform_changed() {
		let mut app = setup();
		let entity = app.world_mut().spawn(related!(Followers[()])).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Transform>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(entity)
				.get::<IsChanged::<FollowStateDirty>>(),
		);
	}

	#[test]
	fn mark_dirty_if_follower_transform_changed() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let follower = app.world_mut().spawn(Follow(entity)).id();

		app.update();
		app.world_mut()
			.entity_mut(follower)
			.get_mut::<FollowTransform>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(entity)
				.get::<IsChanged::<FollowStateDirty>>(),
		);
	}
}
