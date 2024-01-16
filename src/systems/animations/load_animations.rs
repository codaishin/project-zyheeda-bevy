use crate::{
	resources::Animations,
	traits::{load_asset::LoadAsset, recourse_key::ResourceKey},
};
use bevy::{
	animation::AnimationClip,
	asset::Handle,
	ecs::system::{Commands, Res, Resource},
};
use std::{collections::HashMap, hash::Hash};

pub fn load_animations<
	TAnimationKey: ResourceKey + Copy + Send + Sync + Eq + Hash + 'static,
	TLoadAnimation: LoadAsset<AnimationClip> + Resource,
>(
	mut commands: Commands,
	source: Res<TLoadAnimation>,
) {
	let animations = TAnimationKey::resource_keys().map(load_asset_from(source));
	commands.insert_resource(Animations(HashMap::from_iter(animations)));
}

fn load_asset_from<
	TAnimationKey: ResourceKey + Copy,
	TLoadAnimation: LoadAsset<AnimationClip> + Resource,
>(
	source: Res<TLoadAnimation>,
) -> impl Fn((TAnimationKey, String)) -> (TAnimationKey, Handle<AnimationClip>) + '_ {
	move |(key, path)| (key, source.load_asset(path))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::recourse_key::Iter;
	use bevy::{
		animation::AnimationClip,
		app::{App, Update},
		asset::{AssetId, AssetPath, Handle},
		utils::Uuid,
	};

	#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
	enum _Key {
		A,
		B,
	}

	impl ResourceKey for _Key {
		fn resource_keys() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn get_next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_Key::A => Some(_Key::B),
				_Key::B => None,
			}
		}

		fn get_resource_path(value: &Self) -> String {
			match value {
				_Key::A => "A".to_owned(),
				_Key::B => "B".to_owned(),
			}
		}
	}

	#[derive(Resource)]
	struct _LoadAsset(HashMap<String, Handle<AnimationClip>>);

	impl LoadAsset<AnimationClip> for _LoadAsset {
		fn load_asset<'a, TPath: Into<AssetPath<'a>>>(&self, path: TPath) -> Handle<AnimationClip> {
			let path: AssetPath = path.into();
			self.0
				.iter()
				.find_map(|(key, value)| match AssetPath::from(key) == path {
					true => Some(value.clone()),
					false => None,
				})
				.unwrap_or(Handle::default())
		}
	}

	#[test]
	fn add_animations() {
		let mut app = App::new();

		app.insert_resource(_LoadAsset(HashMap::from([])));
		app.add_systems(Update, load_animations::<_Key, _LoadAsset>);
		app.update();

		assert!(app.world.get_resource::<Animations<_Key>>().is_some());
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

		app.insert_resource(_LoadAsset(HashMap::from([
			("A".to_owned(), handle_a.clone()),
			("B".to_owned(), handle_b.clone()),
		])));
		app.add_systems(Update, load_animations::<_Key, _LoadAsset>);
		app.update();

		let animations = app.world.resource::<Animations<_Key>>();

		assert_eq!(
			HashMap::from([(_Key::A, handle_a), (_Key::B, handle_b)]),
			animations.0
		)
	}
}
