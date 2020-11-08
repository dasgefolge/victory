use crate::{
    identities::Identity,
    roles::Role,
};

pub(crate) struct Player<P> {
    pub(crate) id: P,
    pub(crate) role: Role,
    pub(crate) identity: Identity,
    /// `None` = dead
    pub(crate) condition: Option<Condition>,
    /// for example which potions remain
    props: Props,
}

impl<P> Player<P> {
    pub(crate) fn can_act(&self) -> bool {
        self.condition.map_or(false, |cond| cond.ready && !cond.bitten)
    }

    fn ready(&self) -> bool {
        self.condition.map_or(false, |cond| cond.ready)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Condition {
    /// has action token (relevant for angel and active vampire)
    ready: bool,
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

enum Props {} //TODO
