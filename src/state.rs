use {
    std::{
        collections::HashMap,
        hash::Hash,
        mem,
    },
    hashbag::HashBag,
    itertools::{
        EitherOrBoth,
        Itertools as _,
    },
    multiset::HashMultiSet,
    rand::prelude::*,
    smart_default::SmartDefault,
    crate::{
        identities::{
            Identity,
            Wincon,
        },
        roles::Role,
        util::Cycle as _,
        player::{
            Player,
            Condition,
        }
    },
};

pub trait PlayerId: Eq + Hash + Clone {}

impl<T: Eq + Hash + Clone> PlayerId for T {}

#[derive(Debug, SmartDefault)]
pub struct State<P: PlayerId> {
    points: HashMap<P, u32>,
    pub(crate) players: Vec<Player<P>>,
    /// `None` means between segments (i.e. players can join/leave)
    pub(crate) phase: Option<Role>,
    /// whether somebody died this round (relevant for queen)
    pub(crate) mourning: bool,
    /// the target chosen by the hunter
    target: Option<Seat>,
    /// the person shielded from being vetoed _again_ by the angel
    shielded: Option<Seat>,
    /// `true` if the angel shield is from this cycle, `false` if it's from last
    shield_is_current: bool,
    /// the action pending approval by the angel (within frame set by front-end)
    pending_action: Option<Action>,
}

impl<P: PlayerId> State<P> {
    pub fn points(&self) -> &HashMap<P, u32> { &self.points }
    pub fn players(&self) -> &[Player<P>] { &self.players }

    pub fn next_input(&self) -> InputRequest {
        if let Some(phase) = self.phase {
            if let Some(ref p_a) = self.pending_action {
                InputRequest::Angel(phase, p_a)
            } else {
                InputRequest::Action(phase)
            }
        } else {
            InputRequest::Meta
        }
    }

    /// # Panics
    ///
    /// Panics on invalid inputs (e.g. `AngelVeto` with a player ID that doesn't have a pending action).
    pub fn advance_game(&mut self, input: Input<P>) {
        match input {
            Input::Meta(meta_input) => {
                assert!(self.phase.is_none());
                match meta_input {
                    MetaInput::Join(p, seat) => {
                        assert!(!self.players.iter().any(|player| player.id == p));
                        let seat = seat.unwrap_or_else(|| thread_rng().gen_range(0, self.players.len().max(1)));
                        self.players.insert(seat, Player::new(p));
                    }
                    MetaInput::Quit(p) => {
                        self.players.retain(|player| player.id != p);
                    }
                    MetaInput::Go => {
                        fn free_attributes<P, T: Copy + Eq + Hash>(players: &[Player<P>], list: impl Iterator<Item = T>, player_attr: impl Fn(&Player<P>) -> T) -> Vec<T> {
                            let attrs = list.collect::<HashMultiSet<_>>()
                                - players.iter()
                                    .filter(|player| player.condition.is_some())
                                    .map(player_attr)
                                    .collect();
                            let mut attrs = attrs.iter().copied().collect_vec();
                            attrs.shuffle(&mut thread_rng());
                            attrs
                        }
                        
                        let free_roles = free_attributes(&self.players, Role::list(), |player| player.role);
                        let free_identities = free_attributes(&self.players, Identity::list(), |player| player.identity);
                        self.players.iter_mut()
                            .filter(|player| player.condition.is_none())
                            .zip_longest(free_roles.into_iter().zip(free_identities))
                            .for_each(|pair| match pair {
                                EitherOrBoth::Left(_) => panic!("not enough roles and/or identities"),
                                EitherOrBoth::Right(_) => {}
                                EitherOrBoth::Both(dead_player, (free_role, free_identity)) => {
                                    dead_player.condition = Some(Condition::default());
                                    dead_player.role = free_role; //TODO adjust props accordingly
                                    dead_player.identity = free_identity;
                                }
                            });
                            self.players.iter_mut().for_each(|player| player.condition = Some(Condition::default()));
                        self.phase = Some(Role::default());
                    }
                }
            }
            Input::Ingame(ingame_input) => {
                let phase = self.phase.expect("can't use actions in between segments");
                match ingame_input {
                    IngameInput::Action(action) => {
                        self.players.iter_mut()
                            .filter(|player| player.role == phase)
                            .for_each(|player| if let Some(mut cond) = player.condition { cond.ready = false });
                        if unimplemented!() /*TODO Engel kann diese Aktion engeln */ {
                            self.pending_action = Some(action);
                        } else {
                            self.resolve_action(action);
                        }
                    }
                    IngameInput::AngelPass => {
                        let pending = self.pending_action.take().expect("no pending action to pass");
                        self.resolve_action(pending)
                    }
                    IngameInput::AngelVeto(seat) => {
                        //TODO validate seat?
                        self.pending_action = None; //TODO Werwölfe?
                        self.shielded = Some(seat);
                        self.shield_is_current = true;
                        self.set_next_phase();
                    },
                    IngameInput::WinAttempt() => unimplemented!() //TODO
                }
            }
        }
    }

    fn resolve_action(&mut self, action: Action) {
        match action {
            Action::Shoot => self.kill(self.target.expect("no target to shoot")),
            Action::Aim(seat) => {
                assert!(self.players[seat].condition.is_some()); // can't aim at dead players
                self.target = Some(seat);
            }
            Action::Bite(seat) => {
                if let Some(ref mut cond) = self.players[seat].condition {
                    cond.bitten = true;
                } else {
                    panic!("can't bite dead player")
                }
            }
            Action::Swap(seat0, seat1) => {
                //TODO
            }
            Action::Eat(map) => {
                let counts = map.into_iter().map(|(_, target)| target).collect::<HashBag<_>>();
                if let Some((seat, _)) = counts.into_iter().max_by_key(|(_, count)| *count) {
                    self.kill(seat);
                }
            },
            Action::VoteResult(opt_p) => if let Some(p) = opt_p { self.kill(p) },
        }
        self.set_next_phase();
    }

    fn set_next_phase(&mut self) {
        let mut phase = self.phase.expect("can't advance phase in between segments");
        loop {
            phase = phase.succ();
            self.check_wincons();
            if phase == Role::Angel {
                // angel phase used for end-of-cycle cleanup
                if self.shield_is_current {
                    self.shield_is_current = false;
                } else {
                    self.shielded = None;
                }
                self.mourning = false;
                self.target = None;
            } else if !self.players.iter().any(|player| player.role == phase && player.can_act() /*TODO and has choices to make */) { break }
        }
        self.phase = Some(phase);
    }

    fn check_wincons(&mut self) {
        let vs = self.players.iter()
            .enumerate()
            .filter(|(_, player)| if let Wincon::Static(check) = player.identity.wincon() { check(self) } else { false })
            .map(|(seat, _)| seat)
            .collect_vec();
        if !vs.is_empty() { self.victory(vs) }
    }

    fn kill(&mut self, seat: Seat) { //TODO day/night? method?
        assert!(self.players[seat].condition.is_some()); // make sure player was alive
        self.players[seat].condition = None;
        self.mourning = true;
        if self.target == Some(seat) { self.target = None }
    }

    fn victory(&mut self, mut victors: Vec<Seat>) {
        let with_victors = self.players.iter()
            .enumerate()
            .filter(|(_, player)| player.identity.wincon().is_with(self, &victors))
            .map(|(seat, _)| seat)
            .collect_vec();
        victors.extend(with_victors);
        let State { mut points, mut players, .. } = mem::replace(self, State::default());
        victors.iter().for_each(|&seat| players[seat].condition = None); // “kill” all victors so they get a new role/identity and count for the number of points
        let gain = players.iter().filter(|player| player.condition.is_none()).count() as u32;
        victors.iter().for_each(|&seat| *points.entry(players[seat].id.clone()).or_default() += gain);
        self.points = points;
        self.players = players;
    }

    pub(crate) fn player_by_id(&self, pid: &P) -> &Player<P> {
        self.players.iter().find(|player| player.id == *pid).expect("no such player")
    }
}

pub enum InputRequest<'a> {
    /// Accepting all [`Input::Meta`].
    Meta,
    /// Accepting actions or win attempts.
    Action(Role),
    /// Accepting angel vetos, angel passes, or win attempts.
    Angel(Role, &'a Action),
}

pub enum Input<P: Eq + Hash> {
    Meta(MetaInput<P>),
    Ingame(IngameInput),
}
pub enum MetaInput<P: Eq + Hash> {
    Join(P, Option<Seat>),
    Quit(P),
    Go,
}

pub enum IngameInput {
    Action(Action),
    AngelPass,
    AngelVeto(Seat),
    WinAttempt(),
}

#[derive(Debug)]
pub enum Action {
    VoteResult(Option<Seat>), //individual votes are front-end issue
    Shoot,
    Aim(Seat),
    Bite(Seat),
    Swap(Seat, Seat),
    Eat(HashMap<Seat, Seat>),
}

pub(crate) type Seat = usize;
