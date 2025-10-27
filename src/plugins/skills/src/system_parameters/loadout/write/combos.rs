use crate::{
	components::combos::Combos,
	skills::{Skill, SkillId},
	system_parameters::loadout::LoadoutWriter,
	traits::combos::UpdateComboSkills,
};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::EntityContextMut,
		handles_loadout::combos::{Combo, Combos as CombosMarker, UpdateCombos},
	},
};

impl EntityContextMut<CombosMarker> for LoadoutWriter<'_, '_> {
	type TContext<'ctx> = CombosMut<'ctx>;

	fn get_entity_context_mut<'ctx>(
		param: &'ctx mut LoadoutWriter,
		entity: Entity,
		_: CombosMarker,
	) -> Option<Self::TContext<'ctx>> {
		let (.., combos) = param.agents.get_mut(entity).ok()?;

		Some(CombosMut {
			combos,
			skills: &param.skills,
		})
	}
}

pub struct CombosMut<'ctx> {
	combos: Mut<'ctx, Combos>,
	skills: &'ctx Assets<Skill>,
}

impl UpdateCombos<SkillId> for CombosMut<'_> {
	fn update_combos(&mut self, combos: Combo<SlotKey, Option<SkillId>>) {
		let combos = combos
			.into_iter()
			.map(|(key, id)| (key, id.and_then(find_skill(self.skills))));

		self.combos.update_combo_skills(combos);
	}
}

fn find_skill<'a>(skills: &'a Assets<Skill>) -> impl Fn(SkillId) -> Option<&'a Skill> {
	move |id| {
		let (_, skill) = skills.iter().find(|(_, skill)| skill.id == id)?;

		Some(skill)
	}
}
