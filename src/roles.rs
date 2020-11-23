use {
    enum_iterator::IntoEnumIterator,
    smart_default::SmartDefault,
    self::Role::*,
};

/// Roles of the game, which also are the phases of a day/night cycle
#[derive(Debug, SmartDefault, Clone, Copy, IntoEnumIterator, PartialEq, Eq, Hash)]
pub enum Role {
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
    pub(crate) fn list() -> impl Iterator<Item = Role> {
        Role::into_enum_iter()
            .flat_map(|role| if role == Wolf { vec![role; 3] } else { vec![role] })
    }
}
