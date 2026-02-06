use crate::{
	assets::agent_config::AgentConfigAsset,
	components::agent_config::{AgentConfig, RegisterAgentAnimations},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_animations::{Animations, RegisterAnimations},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl RegisterAgentAnimations {
	pub(crate) fn execute<TAnimations>(
		mut commands: ZyheedaCommands,
		mut animations: StaticSystemParam<TAnimations>,
		agents: Query<(Entity, &AgentConfig), With<Self>>,
		configs: Res<Assets<AgentConfigAsset>>,
	) where
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: RegisterAnimations>,
	{
		for (entity, AgentConfig { config_handle }) in &agents {
			let key = Animations { entity };

			let Some(config) = configs.get(config_handle) else {
				continue;
			};
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				ctx.register_animations(&config.animations, &config.animation_mask_groups);
				e.try_remove::<Self>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_config::AgentConfigAsset;
	use common::{
		bit_mask_index,
		tools::{bone_name::BoneName, path::Path},
		traits::handles_animations::{
			AffectedAnimationBones,
			Animation,
			AnimationKey,
			AnimationMaskBits,
			AnimationPath,
			PlayMode,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, NestedMocks)]
	struct _Component {
		mock: Mock_Component,
	}

	#[automock]
	impl RegisterAnimations for _Component {
		fn register_animations(
			&mut self,
			animations: &HashMap<AnimationKey, Animation>,
			animation_mask_groups: &HashMap<AnimationMaskBits, AffectedAnimationBones>,
		) {
			self.mock
				.register_animations(animations, animation_mask_groups);
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut config_assets = Assets::default();

		for (id, asset) in configs {
			_ = config_assets.insert(id, asset);
		}

		app.insert_resource(config_assets);
		app.add_systems(
			Update,
			RegisterAgentAnimations::execute::<Query<Mut<_Component>>>,
		);

		app
	}

	#[test]
	fn set_animations_from_config() {
		let animations = HashMap::from([(
			AnimationKey::Run,
			Animation {
				path: AnimationPath::Single(Path::from("my/path")),
				play_mode: PlayMode::Replay,
				mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(42)),
			},
		)]);
		let animation_mask_groups = HashMap::from([(
			AnimationMaskBits::zero().with_set(bit_mask_index!(4)),
			AffectedAnimationBones {
				from_root: BoneName::from("root"),
				..default()
			},
		)]);
		let config_handle = new_handle();
		let asset = AgentConfigAsset {
			animations: animations.clone(),
			animation_mask_groups: animation_mask_groups.clone(),
			..default()
		};
		let mut app = setup([(&config_handle, asset)]);
		app.world_mut().spawn((
			AgentConfig { config_handle },
			_Component::new().with_mock(move |mock| {
				mock.expect_register_animations()
					.times(1)
					.with(eq(animations.clone()), eq(animation_mask_groups.clone()))
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle();
		let asset = AgentConfigAsset::default();
		let mut app = setup([(&handle, asset)]);
		app.world_mut().spawn((
			AgentConfig {
				config_handle: handle,
			},
			_Component::new().with_mock(move |mock| {
				mock.expect_register_animations().times(1).return_const(());
			}),
		));

		app.update();
		app.update();
	}

	#[test]
	fn set_animations_from_config_when_config_is_late() {
		let animations = HashMap::from([(
			AnimationKey::Run,
			Animation {
				path: AnimationPath::Single(Path::from("my/path")),
				play_mode: PlayMode::Replay,
				mask_groups: AnimationMaskBits::zero().with_set(bit_mask_index!(42)),
			},
		)]);
		let animation_mask_groups = HashMap::from([(
			AnimationMaskBits::zero().with_set(bit_mask_index!(4)),
			AffectedAnimationBones {
				from_root: BoneName::from("root"),
				..default()
			},
		)]);
		let handle = new_handle();
		let asset = AgentConfigAsset {
			animations: animations.clone(),
			animation_mask_groups: animation_mask_groups.clone(),
			..default()
		};
		let mut app = setup([]);
		app.world_mut().spawn((
			AgentConfig {
				config_handle: handle.clone(),
			},
			_Component::new().with_mock(move |mock| {
				mock.expect_register_animations()
					.times(1)
					.with(eq(animations.clone()), eq(animation_mask_groups.clone()))
					.return_const(());
			}),
		));

		app.update();
		_ = app
			.world_mut()
			.resource_mut::<Assets<AgentConfigAsset>>()
			.insert(&handle, asset);
		app.update();
	}
}
