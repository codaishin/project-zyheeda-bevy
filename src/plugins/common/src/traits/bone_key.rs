use crate::traits::handles_animations::BoneName;

pub trait BoneKey<TBone> {
	fn bone_key(&self, bone_name: &str) -> Option<TBone>;
}

pub trait ConfiguredBones<TBone>: BoneKey<TBone> {
	fn bone_names(&self) -> impl Iterator<Item = BoneName>;
}
