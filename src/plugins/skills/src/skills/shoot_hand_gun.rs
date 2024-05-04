use super::{Animate, Cast, Skill, SkillAnimation};
use crate::{
	components::ItemType,
	traits::{AnimationChainIf, GetAnimationSetup, GetExecution, GetSkillAnimation, SkillTemplate},
};
use animations::animation::{Animation, PlayMode};
use behaviors::components::Projectile;
use bevy::utils::default;
use common::{tools::player_animation_path, traits::load_asset::Path};
use std::{collections::HashSet, marker::PhantomData, time::Duration};

pub(crate) struct ShootHandGun<T>(PhantomData<T>);

fn shoot_right() -> Path {
	player_animation_path("Animation4")
}
fn shoot_right_dual() -> Path {
	player_animation_path("Animation6")
}
fn shoot_left() -> Path {
	player_animation_path("Animation5")
}
fn shoot_left_dual() -> Path {
	player_animation_path("Animation7")
}

impl<T> GetAnimationSetup for ShootHandGun<T> {
	fn get_animation() -> SkillAnimation {
		SkillAnimation {
			right: Animation::new(shoot_right(), PlayMode::Repeat),
			left: Animation::new(shoot_left(), PlayMode::Repeat),
		}
	}

	fn get_chains() -> Vec<AnimationChainIf> {
		vec![
			AnimationChainIf {
				last: shoot_right,
				this: shoot_left,
				then: shoot_left_dual,
			},
			AnimationChainIf {
				last: shoot_left,
				this: shoot_right,
				then: shoot_right_dual,
			},
			AnimationChainIf {
				last: shoot_right_dual,
				this: shoot_left,
				then: shoot_left_dual,
			},
			AnimationChainIf {
				last: shoot_left_dual,
				this: shoot_right,
				then: shoot_right_dual,
			},
		]
	}
}

impl<T: Sync + Send + 'static> SkillTemplate for ShootHandGun<T> {
	fn skill() -> Skill {
		Skill {
			name: "Shoot Hand Gun",
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::ZERO,
				after: Duration::from_millis(100),
			},
			animate: Animate::Some(ShootHandGun::<T>::animation()),
			execution: Projectile::<T>::execution(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			icon: Some(|| Path::from("icons/pistol.png")),
			..default()
		}
	}
}
