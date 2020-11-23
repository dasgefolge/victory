use {
    smart_default::SmartDefault,
    crate::{
        identities::Identity,
        roles::Role,
    },
};

#[derive(Debug)]
pub struct Player<P> {
    pub id: P,
    pub(crate) role: Role,
    pub(crate) identity: Identity,
    /// `None` = dead
    pub(crate) condition: Option<Condition>,
    /// for example which potions remain
    pub(crate) props: Props,
}

impl<P> Player<P> {
    pub(crate) fn new(id: P) -> Player<P> {
        Player {
            id,
            // rest overwritten by Go
            role: Role::default(),
            identity: Identity::Single,
            condition: None,
            props: Props::default(),
        }
    }

    pub(crate) fn can_act(&self) -> bool {
        self.condition.map_or(false, |cond| cond.ready && !cond.bitten)
    }

    fn ready(&self) -> bool {
        self.condition.map_or(false, |cond| cond.ready)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Condition {
    /// has action token (relevant for angel and active vampire)
    pub(crate) ready: bool,
    pub(crate) bitten: bool,
}

impl Default for Condition {
    fn default() -> Condition {
        Condition {
            ready: true,
            bitten: false,
        }
    }
}

#[derive(Debug, SmartDefault)]
pub(crate) enum Props {
    #[default]
    None,
    //TODO variants for other roles
}
