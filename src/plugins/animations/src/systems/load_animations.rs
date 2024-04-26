use crate::resource::AnimationClips;
use bevy::{
	animation::AnimationClip,
	asset::Handle,
	ecs::system::{Commands, Res, Resource},
};
use common::traits::{
	iteration::IterKey,
	load_asset::{LoadAsset, Path},
};
use std::{collections::HashMap, hash::Hash};

pub(crate) fn load_animations<
	TAnimationKey: IterKey + Copy + Send + Sync + Eq + Hash + 'static,
	TLoadAnimation: LoadAsset<AnimationClip> + Resource,
>(
	mut commands: Commands,
	source: Res<TLoadAnimation>,
) where
	Path: From<TAnimationKey>,
{
	let animations = TAnimationKey::iterator().map(load_asset_from(source));
	commands.insert_resource(AnimationClips(HashMap::from_iter(animations)));
}

fn load_asset_from<TAnimationKey: Copy, TLoadAnimation: LoadAsset<AnimationClip> + Resource>(
	source: Res<TLoadAnimation>,
) -> impl Fn(TAnimationKey) -> (TAnimationKey, Handle<AnimationClip>) + '_
where
	Path: From<TAnimationKey>,
{
	move |key| (key, source.load_asset(key.into()))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		animation::AnimationClip,
		app::{App, Update},
		asset::{AssetId, Handle},
		utils::Uuid,
	};
	use common::traits::{iteration::Iter, load_asset::Path};
	use mockall::{automock, predicate::eq};

	#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
	enum _Key {
		A,
		B,
	}

	impl IterKey for _Key {
		fn iterator() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_Key::A => Some(_Key::B),
				_Key::B => None,
			}
		}
	}

	impl From<_Key> for Path {
		fn from(value: _Key) -> Self {
			match value {
				_Key::A => "A".into(),
				_Key::B => "B".into(),
			}
		}
	}

	#[derive(Resource, Default)]
	struct _LoadAsset {
		mock: Mock_LoadAsset,
	}

	#[automock]
	impl LoadAsset<AnimationClip> for _LoadAsset {
		fn load_asset(&self, path: Path) -> Handle<AnimationClip> {
			self.mock.load_asset(path)
		}
	}

	#[test]
	fn add_animations() {
		let mut app = App::new();
		let mut load_asset = _LoadAsset::default();
		load_asset
			.mock
			.expect_load_asset()
			.return_const(Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}));

		app.insert_resource(load_asset);
		app.add_systems(Update, load_animations::<_Key, _LoadAsset>);
		app.update();

		assert!(app.world.get_resource::<AnimationClips<_Key>>().is_some());
	}

	#[test]
	fn load_asset() {
		let mut app = App::new();
		let handle_a = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let handle_b = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut load_asset = _LoadAsset::default();
		load_asset
			.mock
			.expect_load_asset()
			.with(eq(Path::from("A")))
			.return_const(handle_a.clone());
		load_asset
			.mock
			.expect_load_asset()
			.with(eq(Path::from("B")))
			.return_const(handle_b.clone());

		app.insert_resource(load_asset);
		app.add_systems(Update, load_animations::<_Key, _LoadAsset>);
		app.update();

		let animations = app.world.resource::<AnimationClips<_Key>>();

		assert_eq!(
			HashMap::from([(_Key::A, handle_a), (_Key::B, handle_b)]),
			animations.0
		)
	}
}
