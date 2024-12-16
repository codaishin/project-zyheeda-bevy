use bevy::prelude::*;

pub trait HandlesSkills {
	type SkillContact: Component;
	type SkillProjection: Component;
}
