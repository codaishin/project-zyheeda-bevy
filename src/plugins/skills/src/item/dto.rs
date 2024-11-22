use content::SkillItemContentDto;
use items::item::dto::ItemDto;

pub(crate) mod content;

pub(crate) type SkillItemDto = ItemDto<SkillItemContentDto>;
