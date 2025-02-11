use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{handles_combo_menu::IsCompatible, thread_safe::ThreadSafe},
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct EquipmentInfo<TIsCompatible>(TIsCompatible);

impl<TIsCompatible> EquipmentInfo<TIsCompatible>
where
	TIsCompatible: ThreadSafe,
{
	pub(crate) fn update(In(compatible): In<Option<TIsCompatible>>, mut commands: Commands) {
		let Some(compatible) = compatible else {
			return;
		};
		commands.insert_resource(Self(compatible));
	}
}

impl<TIsCompatible, TSkill> IsCompatible<TSkill> for EquipmentInfo<TIsCompatible>
where
	TIsCompatible: IsCompatible<TSkill>,
{
	fn is_compatible(&self, key: &SlotKey, skill: &TSkill) -> bool {
		self.0.is_compatible(key, skill)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Compatible;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_instance() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once_with(Some(_Compatible), EquipmentInfo::update)?;

		assert_eq!(
			Some(&EquipmentInfo(_Compatible)),
			app.world().get_resource::<EquipmentInfo<_Compatible>>()
		);
		Ok(())
	}
}
