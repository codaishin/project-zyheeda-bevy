use crate::components::map::Map;
use bevy::prelude::*;
use common::prelude::*;
use macros::serde_model;

#[serde_model]
#[derive(Component, Debug, PartialEq, Default, Clone)]
#[require(Map, Name = Self, Model = Self)]
pub(crate) struct Level<const L: i8>;

impl<const L: i8> SavableComponent for Level<L> {
	type TDto = Self;

	const ID: UniqueComponentId = UniqueComponentId::from_lazy(|| match L {
		l if l < 0 => format!("level_neg_{}", l.abs()),
		l => format!("level_{}", l),
	});
}

impl<const L: i8> From<Level<L>> for Name {
	fn from(_: Level<L>) -> Self {
		match L {
			l if l < 0 => Name::from(format!("LevelNeg{}", l.abs())),
			l => Name::from(format!("Level{}", l)),
		}
	}
}

impl<const L: i8> From<Level<L>> for Model {
	fn from(_: Level<L>) -> Self {
		match L {
			l if l < 0 => Model::scene(format!("maps/levels/neg_{}/model.glb", l.abs())),
			l => Model::scene(format!("maps/levels/{}/model.glb", l)),
		}
	}
}
