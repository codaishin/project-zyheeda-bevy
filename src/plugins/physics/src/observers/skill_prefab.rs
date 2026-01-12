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
		let target = trigger.target();
		let Ok(skill) = skills.get(target) else {
			return;
		};

		let Some(mut entity) = commands.get_mut(&target) else {
			return;
		};

		if let Some(lifetime) = skill.lifetime {
			entity.try_insert(Lifetime::from(lifetime));
		}

		skill.motion(&mut entity);
		skill.contact(&mut entity);
		skill.projection(&mut commands.spawn(ChildOf(target)).into());
	}
}
