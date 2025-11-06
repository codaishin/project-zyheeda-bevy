use common::{
	tools::path::Path,
	traits::animation::{Animation, AnimationPath},
};
use std::slice::Iter;
use uuid::Uuid;

pub(crate) fn leak_iterator(animations: Vec<Animation>) -> Iter<'static, Animation> {
	Box::new(animations).leak().iter()
}

pub(crate) fn unique_animation_asset() -> AnimationPath {
	AnimationPath::Single(Path::from(Uuid::new_v4().to_string()))
}
