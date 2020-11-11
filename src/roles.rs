use {
    enum_iterator::IntoEnumIterator,
    smart_default::SmartDefault,
    self::Role::*,
};

/// Roles of the game, which also are the phases of a day/night cycle
#[derive(SmartDefault, Clone, Copy, IntoEnumIterator, PartialEq, Eq)]
pub(crate) enum Role {
    /// first phase, not default role
    #[default]
    Hunter,
    Vampire,
    Jester,
    Seer,
    Wolf,
    Witch,
    Mayor,
    /// phase functions as end of cycle cleanup
    Angel,
}

impl Role {
    pub(crate) fn list() -> Vec<Role> {
        vec![
            Hunter,
            Vampire,
            Jester,
            Seer,
            Wolf,
            Wolf,
            Wolf,
            Witch,
            Mayor,
            Angel,
        ]
    }
}
