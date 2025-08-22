use bevy::prelude::*;
use common::traits::handles_enemies::EnemyType;
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AgentsColorLookup {
	pub(crate) player: Color,
	pub(crate) enemies: ColorEnemyMap,
}

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct AgentsColorLookupImages<TImage = Image>
where
	TImage: Asset,
{
	pub(crate) player: Handle<TImage>,
	pub(crate) enemies: HashMap<EnemyType, Handle<TImage>>,
}

impl AgentsColorLookupImages {
	pub(crate) const ROOT_PATH: &str = "maps/lookup";

	pub(crate) const PLAYER_FILE: &str = "player.png";

	pub(crate) fn get_enemy_file(enemy_type: &EnemyType) -> &'static str {
		match enemy_type {
			EnemyType::VoidSphere => "void_sphere.png",
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct ColorEnemyMap(Vec<(Color, EnemyType)>);

impl ColorEnemyMap {
	pub(crate) fn get(&self, color: &Color) -> Option<&EnemyType> {
		self.0.iter().find(|(c, _)| c == color).map(|(_, e)| e)
	}
}

impl<T> From<T> for ColorEnemyMap
where
	T: IntoIterator<Item = (Color, EnemyType)>,
{
	fn from(colors: T) -> Self {
		Self(colors.into_iter().collect())
	}
}
