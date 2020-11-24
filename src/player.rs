use {
    smart_default::SmartDefault,
    crate::{
        identities::Identity,
        roles::Role,
        state::Seat,
    },
};

#[derive(Debug)]
pub struct Player<P> {
    pub id: P,
    /// `None` = dead
    pub(crate) character: Option<Character>,
}

impl<P> Player<P> {
    pub(crate) fn new(id: P) -> Player<P> {
        Player {
            id,
            character: None,
        }
    }

    pub(crate) fn can_act(&self) -> bool {
        self.character.as_ref().map_or(false, |c| c.ready && c.bitten_by.is_none())
    }
}

#[derive(Debug)]
pub struct Character {
    pub(crate) role: Role,
    pub(crate) identity: Identity,
    /// has action token (relevant for angel and active vampire)
    pub(crate) ready: bool,
    pub(crate) bitten_by: Option<Seat>,
    /// for example which potions remain
    pub(crate) props: Props,
}

impl Character {
    pub(crate) fn new(role: Role, identity: Identity) -> Character {
        Character {
            role, identity,
            ready: true,
            bitten_by: None,
            props: Props::for_role(role),
        }
    }
}

#[derive(Debug, SmartDefault)]
pub(crate) enum Props {
    #[default]
    None,
    Witch {
        health_potion: bool,
        poison_potion: bool,
    },
}

impl Props {
    fn for_role(role: Role) -> Props {
        if role == Role::Witch {
            Props::Witch {
                health_potion: true,
                poison_potion: true,
            }
        } else {
            Props::None
        }
    }
}
