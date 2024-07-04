use crate::items::ItemType;
use common::traits::load_asset::Path;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
enum Animate {
	ShootHandGun,
}

#[derive(Serialize, Deserialize, Debug)]
enum ProjectileType {
	Plasma,
}

#[derive(Serialize, Deserialize, Debug)]
enum Behavior {
	Projectile(ProjectileType),
	ForceShield,
	GravityWell,
}

#[derive(Serialize, Deserialize, Debug)]
struct SkillData {
	name: String,
	active: Duration,
	animate: Animate,
	behavior: Behavior,
	is_usable_with: HashSet<ItemType>,
	icon: Path,
}
