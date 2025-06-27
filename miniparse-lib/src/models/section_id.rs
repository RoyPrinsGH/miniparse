#[derive(Debug, Default)]
pub enum SectionId<'content> {
    #[default]
    Global,
    Named(&'content str),
}
