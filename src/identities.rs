use {
    std::{
        convert::Infallible as Never,
        hash::Hash,
        ops::BitOr,
    },
    crate::{
        roles::Role,
        state::{
            PlayerId,
            Seat,
            State,
        },
    },
    self::{
        Identity::*,
        Party::*,
        SoloIdentity::*,
        Wincon::*,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SoloIdentity {
    Sherlock,
    V,
    JackTheRipper,
    Queen,
    Macbeth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Identity {
    Solo(SoloIdentity),
    Romeo,
    Juliet,
    Single,
    Churchill,
    Doyle,
    Mozart,
}

impl Identity {
    pub(crate) fn list() -> impl Iterator<Item = Identity> {
        vec![Solo(Sherlock),Solo(V),Solo(JackTheRipper),Solo(Queen),Solo(Macbeth),Romeo,Juliet,Single,Churchill,Doyle,Mozart].into_iter() //TODO: recursive enum iter macro
    }

    pub(crate) fn party(&self) -> Party {
        match *self {
            Solo(solo) => SoloParty(solo),
            Romeo | Juliet | Single => Lovers,
            Churchill | Doyle | Mozart => Masons,
        }
    }

    pub(crate) fn wincon<P: PlayerId>(&self) -> Wincon<P> {
        (match self {
            Romeo | Juliet | Single => None,
            Churchill => With(SoloParty(Queen)),
            Doyle => With(SoloParty(Sherlock)),
            Mozart => With(Lovers),
            Solo(Queen) => Static(Box::new(|state| state.phase == Some(Role::Angel) && !state.mourning)),
            _ => unimplemented!(), //TODO
        }) | self.party().wincon()
    }
}

type Flip = Never; // "IO Bool"

pub(crate) enum Wincon<P: PlayerId> {
    Static(Box<dyn Fn(&State<P>) -> bool>),
    Flips(Flip),
    With(Party),
    /// constructed on the fly only, with `bitor`
    Either(Box<Wincon<P>>, Box<Wincon<P>>),
    None,
}

impl<P: PlayerId> Wincon<P> {
    pub(crate) fn is_with(&self, state: &State<P>, victors: &[Seat]) -> bool {
        match self {
            With(party) => victors.iter().any(|&seat| state.players[seat].character.as_ref().map_or(false, |c| c.identity.party() == *party)),
            Either(lhs, rhs) => lhs.is_with(state, victors) || rhs.is_with(state, victors),
            _ => false,
        }
    }
}

impl<P: PlayerId> BitOr for Wincon<P> {
    type Output = Wincon<P>;

    fn bitor(self, rhs: Wincon<P>) -> Wincon<P> {
        match (self, rhs) {
            (wincon, None) | (None, wincon) => wincon,
            (lhs, rhs) => Either(Box::new(lhs), Box::new(rhs)),
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum Party {
    SoloParty(SoloIdentity),
    Lovers,
    Masons,
}

impl Party {
    fn wincon<P: PlayerId>(&self) -> Wincon<P> {
        match self {
            SoloParty(_) => None,
            Lovers => Flips(unimplemented!()), //TODO
            Masons => Flips(unimplemented!()), //TODO
        }
    }
}
