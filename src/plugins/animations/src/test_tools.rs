use common::{tools::path::Path, traits::animation::AnimationPath};
use std::slice::Iter;
use uuid::Uuid;

pub(crate) fn leak_iterator<T>(animations: Vec<T>) -> Iter<'static, T> {
	Box::new(animations).leak().iter()
}

pub(crate) fn unique_animation_asset() -> AnimationPath {
	AnimationPath::Single(Path::from(Uuid::new_v4().to_string()))
}
