pub(crate) mod player;

pub(crate) trait SubModelNames {
	fn sub_model_names() -> Vec<&'static str>;
}
