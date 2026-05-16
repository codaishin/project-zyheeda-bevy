use common::components::persistent_entity::PersistentEntity;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum OnSkillStop {
	Ignore,
	Stop(PersistentEntity),
}
