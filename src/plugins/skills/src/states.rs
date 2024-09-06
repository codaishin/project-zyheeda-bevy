use bevy::prelude::States;

#[derive(States, Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum SkillAssets {
	Loading,
	Loaded,
}
