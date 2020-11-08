use {
    enum_iterator::IntoEnumIterator,
    smart_default::SmartDefault,
};

#[derive(SmartDefault, IntoEnumIterator, PartialEq, Eq)]
pub(crate) enum Role {
    #[default]
    Hunter,
    Vampire,
    Jester,
    Seer,
    Wolf,
    Witch,
    Mayor,
    Angel, // phase functions as end of cycle
}
