use bevy::prelude::*;
use common::{components::AssetModel, traits::try_insert_on::TryInsertOn};

#[derive(Component)]
#[require(
	Transform,
	Visibility,
	AssetModel(|| AssetModel::path("models/test.glb#Scene0")),
	Name( ||"Tail")
)]
pub(crate) struct Tail;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) enum TailBone {
	NoId,
	Id(usize),
}

impl TailBone {
	pub(crate) fn discover(mut commands: Commands, bones: Query<(Entity, &Name), Added<Name>>) {
		for (entity, name) in &bones {
			let name = name.split(".").collect::<Vec<_>>();

			let bone = match name.as_slice() {
				[name, id] if name == &"tail" => {
					if let Ok(id) = id.parse::<usize>() {
						TailBone::Id(id)
					} else {
						TailBone::NoId
					}
				}
				[name] if name == &"tail" => TailBone::Id(0),
				_ => continue,
			};

			commands.try_insert_on(entity, (bone, Name::from(bone)));
		}
	}
}

impl From<TailBone> for Name {
	fn from(value: TailBone) -> Self {
		Name::from(format!("TailBone::{:?}", value))
	}
}
