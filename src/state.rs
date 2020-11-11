use {
    std::{
        collections::HashMap,
        hash::Hash,
        mem,
    },
    hashbag::HashBag,
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
            Condition
        }
    },
};

#[derive(SmartDefault)]
pub(crate) struct State<P: Eq + Hash> {
    points: HashMap<P, u32>,
    pub(crate) players: Vec<Player<P>>,
    pub(crate) phase: Role,
    /// whether somebody died this round (relevant for queen)
    pub(crate) mourning: bool,
    target: Option<Seat>,
    shielded: Option<Seat>,
    /// `true` if the angel shield is from this cycle, `false` if it's from last
    shield_is_current: bool,
    pending_action: Option<Action>,
}

impl<P: Eq + Hash> State<P> {
    fn next_input(&self) -> InputRequest {
        unimplemented!() //TODO
    }

    /// # Panics
    ///
    /// Panics on invalid inputs (e.g. `AngelVeto` with a player ID that doesn't have a pending action).
    fn advance_game(&mut self, input: Input) {
        match input {
            Input::Action(action) => {
                let phase = self.phase;
                self.players.iter_mut()
                    .filter(|player| player.role == phase)
                    .for_each(|player| if let Some(mut cond) = player.condition { cond.ready = false });
                if unimplemented!() /*TODO Engel kann diese Aktion engeln */ {
                    self.pending_action = Some(action);
                } else {
                    self.resolve_action(action);
                }
            }
            Input::AngelPass => {
                let pending = self.pending_action.take().expect("no pending action to pass");
                self.resolve_action(pending)
            }
            Input::AngelVeto(seat) => {
                //TODO validate seat?
                self.pending_action = None; //TODO Werwölfe?
                self.shielded = Some(seat);
                self.set_next_phase();
            },
            Input::WinAttempt() => unimplemented!() //TODO
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
        loop {
            self.phase = self.phase.succ();
            self.check_wincons();
            if self.phase == Role::Angel {
                // angel phase used for end-of-cycle cleanup
                if self.shield_is_current {
                    self.shield_is_current = false;
                } else {
                    self.shielded = None;
                }
                self.mourning = false;
                self.target = None;
            } else if !self.players.iter().any(|player| player.role == self.phase && player.can_act() /*TODO and has choices to make */) { break }
        }
    }

    fn check_wincons(&mut self) {
        let vs = self.players.iter()
            .enumerate()
            .filter(|(_, player)| if let Wincon::Static(check) = player.identity.wincon() { check(self) } else { false })
            .map(|(seat, _)| seat)
            .collect::<Vec<_>>();
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
            .collect::<Vec<_>>();
        victors.extend(with_victors);
        let State { points, players, .. } = mem::replace(self, State::default());
        self.points = points.into_iter().map(|(p, points)| {
            let new_points = if victors.iter().any(|&seat| self.players[seat].id == p) { points + 1 /*TODO*/ } else { points };
            (p, new_points)
        }).collect();
        self.players = players.into_iter().map(|mut player| {
            player.condition = Some(Condition::default());
            //TODO reset role/identity if dead/won
            player
        }).collect();
        //TODO allow players to join/leave
    }

    pub(crate) fn player_by_id(&self, pid: &P) -> &Player<P> {
        self.players.iter().find(|player| player.id == *pid).expect("no such player")
    }
}

pub enum InputRequest {} //TODO

pub enum Input {
    Action(Action),
    AngelPass,
    AngelVeto(Seat),
    WinAttempt(),
}

pub enum Action {
    VoteResult(Option<Seat>), //individual votes are front-end issue
    Shoot,
    Aim(Seat),
    Bite(Seat),
    Swap(Seat, Seat),
    Eat(HashMap<Seat, Seat>), //TODO decide whether werewolves can vote not to eat anyone (HashMap<Seat, Option<Seat>>)
}

pub(crate) type Seat = usize;
