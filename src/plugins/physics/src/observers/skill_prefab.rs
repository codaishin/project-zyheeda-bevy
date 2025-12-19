use crate::components::skill::Skill;
use bevy::prelude::*;
use common::{
	components::lifetime::Lifetime,
	traits::accessors::get::GetMut,
	zyheeda_commands::ZyheedaCommands,
};

impl Skill {
	pub(crate) fn prefab(
		trigger: Trigger<OnInsert, Skill>,
		mut commands: ZyheedaCommands,
		skills: Query<&Self>,
	) {
		let root = trigger.target();
		let Ok(skill) = skills.get(root) else {
			return;
		};
		let Some(mut entity) = commands.get_mut(&root) else {
			return;
		};

		if let Some(lifetime) = skill.lifetime {
			entity.try_insert(Lifetime::from(lifetime));
		}

		skill.motion(&mut entity);
		skill.contact(&mut commands.spawn(ChildOf(root)).into(), root);
		skill.projection(&mut commands.spawn(ChildOf(root)).into(), root);
	}
}
