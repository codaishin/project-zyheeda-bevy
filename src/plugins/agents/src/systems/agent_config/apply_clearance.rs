use crate::{
	assets::agent_meta::AgentMeta,
	components::{agent::AgentTransformDirty, agent_config::AgentConfig},
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl AgentConfig {
	pub(crate) fn apply_clearance(
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &mut Transform, &Self), With<AgentTransformDirty>>,
		metas: Res<Assets<AgentMeta>>,
	) {
		for (entity, mut transform, Self { config_handle }) in agents {
			let Some(meta) = metas.get(config_handle) else {
				continue;
			};

			transform.translation.y += *meta.required_clearance.vertical;
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<AgentTransformDirty>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::Units, traits::handles_movement::RequiredClearance};
	use testing::{SingleThreadedApp, new_handle};

	fn setup<const N: usize>(metas: [(&Handle<AgentMeta>, AgentMeta); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, asset) in metas {
			_ = assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.add_systems(Update, AgentConfig::apply_clearance);

		app
	}

	#[test]
	fn apply_clearance() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			required_clearance: RequiredClearance {
				vertical: Units::from_u8(10),
				..default()
			},
			..default()
		};
		let mut app = setup([(&config_handle, meta)]);
		let entity = app
			.world_mut()
			.spawn((
				AgentConfig { config_handle },
				Transform::from_xyz(1., 2., 3.),
				AgentTransformDirty,
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 12., 3.)),
			app.world().entity(entity).get::<Transform>(),
		);
	}

	#[test]
	fn do_nothing_if_transform_not_dirty() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			required_clearance: RequiredClearance {
				vertical: Units::from_u8(10),
				..default()
			},
			..default()
		};
		let mut app = setup([(&config_handle, meta)]);
		let entity = app
			.world_mut()
			.spawn((
				AgentConfig { config_handle },
				Transform::from_xyz(1., 2., 3.),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 3.)),
			app.world().entity(entity).get::<Transform>(),
		);
	}

	#[test]
	fn remove_transform_dirty() {
		let config_handle = new_handle();
		let meta = AgentMeta::default();
		let mut app = setup([(&config_handle, meta)]);
		let entity = app
			.world_mut()
			.spawn((
				AgentConfig { config_handle },
				Transform::from_xyz(1., 2., 3.),
				AgentTransformDirty,
			))
			.id();

		app.update();

		assert!(!app.world().entity(entity).contains::<AgentTransformDirty>());
	}
}
