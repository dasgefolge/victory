use enum_iterator::IntoEnumIterator;

#[derive(IntoEnumIterator)]
pub(crate) enum Role {
    Angel,
    Hunter,
    Vampire,
    Jester,
    Seer,
    Wolf,
    Witch,
    Mayor,
}
