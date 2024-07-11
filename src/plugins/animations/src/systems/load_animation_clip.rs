use crate::{
	resource::AnimationClips,
	traits::{AnimationPath, HighestPriorityAnimation},
};
use bevy::{
	animation::AnimationClip,
	ecs::{
		component::Component,
		query::Changed,
		system::{Query, ResMut, Resource},
	},
};
use common::traits::load_asset::{LoadAsset, Path};
use std::collections::hash_map::Entry;

pub(crate) fn load_animation_clip<
	TAnimation: AnimationPath + Sync + Send + 'static,
	TAnimationDispatch: Component + HighestPriorityAnimation<TAnimation>,
	TServer: Resource + LoadAsset<AnimationClip>,
>(
	mut clips: ResMut<AnimationClips<Path>>,
	mut server: ResMut<TServer>,
	dispatchers: Query<&TAnimationDispatch, Changed<TAnimationDispatch>>,
) {
	for animation in dispatchers.iter().filter_map(has_animation) {
		let path = animation.animation_path().clone();
		let Entry::Vacant(entry) = clips.0.entry(path.clone()) else {
			continue;
		};
		let clip = server.load_asset(path);
		entry.insert(clip);
	}
}

fn has_animation<TAnimation, TAnimationDispatch: HighestPriorityAnimation<TAnimation>>(
	dispatch: &TAnimationDispatch,
) -> Option<&TAnimation> {
	dispatch.highest_priority_animation()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
	};
	use common::{test_tools::utils::SingleThreadedApp, traits::load_asset::Path};
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use uuid::Uuid;

	struct _Animation(Path);

	impl AnimationPath for _Animation {
		fn animation_path(&self) -> &Path {
			&self.0
		}
	}

	#[derive(Component, Default)]
	struct _AnimationDispatch(Option<_Animation>);

	impl HighestPriorityAnimation<_Animation> for _AnimationDispatch {
		fn highest_priority_animation(&self) -> Option<&_Animation> {
			self.0.as_ref()
		}
	}

	#[derive(Resource, Default)]
	struct _LoadAnimation {
		mock: Mock_LoadAnimation,
	}

	#[automock]
	impl LoadAsset<AnimationClip> for _LoadAnimation {
		fn load_asset(&mut self, path: Path) -> Handle<AnimationClip> {
			self.mock.load_asset(path)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<AnimationClips<Path>>();
		app.add_systems(
			Update,
			load_animation_clip::<_Animation, _AnimationDispatch, _LoadAnimation>,
		);

		app
	}

	#[test]
	fn store_clip() {
		let mut app = setup();
		let mut server = _LoadAnimation::default();

		let dispatch = _AnimationDispatch(Some(_Animation(Path::from("a/path"))));
		let clip = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});

		server.mock.expect_load_asset().return_const(clip.clone());
		app.insert_resource(server);

		app.world_mut().spawn(dispatch);
		app.update();

		let clips = app.world().resource::<AnimationClips<Path>>();

		assert_eq!(Some(&clip), clips.0.get(&Path::from("a/path")));
	}

	#[test]
	fn use_correct_path() {
		let mut app = setup();
		let mut server = _LoadAnimation::default();

		let dispatch = _AnimationDispatch(Some(_Animation(Path::from("top/secret/path"))));

		server
			.mock
			.expect_load_asset()
			.times(1)
			.with(eq(Path::from("top/secret/path")))
			.return_const(Handle::default());
		app.insert_resource(server);

		app.world_mut().spawn(dispatch);
		app.update();
	}

	#[test]
	fn only_attempt_load_when_dispatch_changes() {
		let mut app = setup();
		let mut server = _LoadAnimation::default();

		let dispatch = _AnimationDispatch(Some(_Animation(Path::from("path/one"))));

		server
			.mock
			.expect_load_asset()
			.times(2)
			.return_const(Handle::default());
		app.insert_resource(server);

		let agent = app.world_mut().spawn(dispatch).id();
		app.update();
		app.update();

		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_AnimationDispatch>()
			.unwrap()
			.0 = Some(_Animation(Path::from("path/two")));
		app.update();
	}

	#[test]
	fn do_not_attempt_load_when_clip_already_stored() {
		let mut app = setup();
		let mut server = _LoadAnimation::default();

		let path = Path::from("a/path");
		let dispatch = _AnimationDispatch(Some(_Animation(path.clone())));

		server
			.mock
			.expect_load_asset()
			.never()
			.return_const(Handle::default());
		app.insert_resource(server);
		app.insert_resource(AnimationClips(HashMap::from([(path, Handle::default())])));

		app.world_mut().spawn(dispatch);
		app.update();
	}
}
