use common::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum OnSkillStop {
	Ignore,
	Stop(PersistentEntity),
}
