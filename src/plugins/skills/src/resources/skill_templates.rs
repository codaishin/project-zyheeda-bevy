use crate::skill::Skill;
use bevy::ecs::system::Resource;
use common::errors::{Error, Level};
use std::collections::{hash_map::Entry::Vacant, HashMap};

#[derive(Resource)]
pub(crate) struct SkillTemplates(HashMap<&'static str, Skill>);

impl SkillTemplates {
	pub fn new<const N: usize>(skills: &[Skill; N]) -> (Self, Vec<Error>) {
		let mut templates = HashMap::<&'static str, Skill>::new();
		let mut errors: Vec<Error> = vec![];

		for skill in skills {
			if let Vacant(entry) = templates.entry(skill.name) {
				entry.insert(skill.clone());
			} else {
				errors.push(duplication_error(skill.name));
			}
		}

		(Self(templates), errors)
	}

	pub fn get(&self, key: &'static str) -> Option<&Skill> {
		self.0.get(&key)
	}
}

fn duplication_error(key: &'static str) -> Error {
	Error {
		msg: format!(
			"A skill template with key '{}' already exists, ignoring",
			key
		),
		lvl: Level::Warning,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn skill_name_as_key() {
		let skills = [
			Skill {
				name: "1",
				..default()
			},
			Skill {
				name: "2",
				..default()
			},
		];
		let (templates, errors) = SkillTemplates::new(&skills);
		let map: HashMap<&'static str, Skill> = skills.map(|s| (s.name, s)).into();

		assert_eq!((map, vec![]), (templates.0, errors),);
	}

	#[test]
	fn duplicate_key_errors() {
		let skills = [
			Skill {
				name: "1",
				..default()
			},
			Skill {
				name: "2",
				..default()
			},
			Skill {
				name: "1",
				..default()
			},
			Skill {
				name: "2",
				..default()
			},
		];
		let (templates, errors) = SkillTemplates::new(&skills);
		let map: HashMap<&'static str, Skill> = skills.map(|s| (s.name, s)).into();

		assert_eq!(
			(map, vec![duplication_error("1"), duplication_error("2")]),
			(templates.0, errors),
		);
	}

	#[test]
	fn get_skill_at_key() {
		let skills = [
			Skill {
				name: "1",
				..default()
			},
			Skill {
				name: "2",
				..default()
			},
		];
		let (templates, ..) = SkillTemplates::new(&skills);

		assert_eq!(Some(&skills[1]), templates.get("2"));
	}
}
