use crate::{
	components::skill_prefabs::{skill_contact::SkillContact, skill_projection::SkillProjection},
	system_params::skill_spawner::SpawnNewSkillContextMut,
};
use common::traits::handles_skill_behaviors::{Contact, Projection, SkillEntities, SpawnNewSkill};

impl SpawnNewSkill for SpawnNewSkillContextMut<'_> {
	fn spawn_new_skill(&mut self, contact: Contact, projection: Projection) -> SkillEntities {
		(self.spawn)(
			SkillContact::from(contact),
			SkillProjection::from(projection),
		)
	}
}
