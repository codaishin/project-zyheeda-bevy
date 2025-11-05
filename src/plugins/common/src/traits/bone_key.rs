pub trait BoneKey<TBone> {
	fn bone_key(&self, bone_name: &str) -> Option<TBone>;
}
