use bevy::{
	self,
	color::palettes::css::{BLUE, GREEN, ORANGE},
	prelude::*,
};
use common::{components::AssetModel, traits::try_insert_on::TryInsertOn};

#[derive(Component)]
#[require(
	Visibility,
	AssetModel(|| AssetModel::path("models/test.glb#Scene0")),
	Name( ||"Tail")
)]
pub(crate) struct Tail;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) enum TailBone {
	Root,
	Node,
}

const TAIL_BONE_DIRECTIONS: &[Vec3] = &[
	Vec3::new(0., 1., 0.),
	Vec3::new(1., 1., 0.),
	Vec3::new(1., 1., 0.),
];

impl TailBone {
	pub(crate) fn discover(mut commands: Commands, bones: Query<(Entity, &Name), Added<Name>>) {
		for (entity, name) in &bones {
			let name = String::from(name);
			if name == *"tail" {
				commands.try_insert_on(entity, (TailBone::Root, Name::from(TailBone::Root)));
			} else if name.starts_with("tail.") {
				commands.try_insert_on(entity, (TailBone::Node, Name::from(TailBone::Node)));
			}
		}
	}

	pub(crate) fn set_transforms(
		mut commands: Commands,
		bones: Query<(Entity, &TailBone, &Transform, Option<&Children>), Added<TailBone>>,
	) {
		for (entity, bone, local, children) in &bones {
			if bone != &TailBone::Root {
				continue;
			}

			compute(&mut commands, &bones, entity, local, children, 0);
		}
	}

	pub(crate) fn print_axis(mut gizmos: Gizmos, bones: Query<&GlobalTransform, With<TailBone>>) {
		for transform in &bones {
			let transform = Transform::from(*transform);
			gizmos.ray(transform.translation, transform.forward().as_vec3(), BLUE);
			gizmos.ray(transform.translation, transform.right().as_vec3(), GREEN);
			gizmos.ray(transform.translation, transform.up().as_vec3(), ORANGE);
		}
	}
}

#[allow(clippy::type_complexity)]
fn compute(
	commands: &mut Commands,
	bones: &Query<(Entity, &TailBone, &Transform, Option<&Children>), Added<TailBone>>,
	parent: Entity,
	parent_local: &Transform,
	parent_children: Option<&Children>,
	index: usize,
) {
	let Some(tail_dirs) = TAIL_BONE_DIRECTIONS.get(index) else {
		return;
	};

	let old_up = Vec3::from(parent_local.up());
	let old_forward = Vec3::from(parent_local.forward());

	let up = tail_dirs.try_normalize().unwrap_or(Vec3::Y);
	let forward = match up.cross(old_up) {
		forward if forward.dot(old_forward) > 0. => forward,
		forward => -forward,
	};

	commands.try_insert_on(parent, parent_local.looking_to(forward, up));

	let Some(parent_children) = parent_children else {
		return;
	};

	for child in parent_children.iter() {
		let Ok((child, _, child_local, child_children)) = bones.get(*child) else {
			continue;
		};

		compute(
			commands,
			bones,
			child,
			child_local,
			child_children,
			index + 1,
		);
	}
}

impl From<TailBone> for Name {
	fn from(value: TailBone) -> Self {
		Name::from(format!("TailBone::{:?}", value))
	}
}
