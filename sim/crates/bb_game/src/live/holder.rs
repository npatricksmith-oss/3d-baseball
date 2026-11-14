//! A4: one comparison of legal carries and throws. Only current public motion,
//! known rules/outs and the holder's own timing models enter these estimates.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct HolderOption {
    pub runner: usize,
    pub base: u8,
    pub carry: bool,
    pub force: bool,
    pub through: bool,
    pub receiver: usize,
    pub runner_at: f64,
    pub ball_at: f64,
    pub set_ball_at: f64,
    pub direct_at: f64,
    pub via_at: f64,
    pub receiver_at: f64,
    pub distance: f64,
    pub p_out: f64,
    pub weight: f64,
    pub value: f64,
    pub continuation: Option<(f64, f64)>, // (p(second force), completion time)
}

#[derive(Clone, Copy)]
pub(super) struct ThrowRace {
    pub direct: f64,
    pub via: f64,
    pub relay: Option<usize>,
    pub receiver: usize,
    pub receiver_at: f64,
    pub ball_at: f64,
    pub set_ball_at: f64,
    pub through: bool,
}

impl Sim<'_> {
    /// Existing direct/relay timing, evaluated only AFTER destination admission.
    /// Also used for the unchanged no-play return to second.
    pub(super) fn throw_race(&self, by: usize, base: u8) -> ThrowRace {
        let fd = &self.f[by];
        let relay = (0..9).find(|&i| i != by && matches!(self.f[i].role, Role::Relay(_)) && (self.f[i].pos - fd.pos).norm() > 10.0);
        let release = |target, at: f64| {
            let su = self.set_up_for(by, target);
            let t = at.max(fd.up_at + su.earliest_s);
            (t, THROW_EFFORT * fd.a.arm * su.speed_mult(su.hurry(t - fd.up_at)))
        };
        let direct = |at| {
            let (t, v) = release(base_pos(base), at);
            t + throw_time(v, (base_pos(base) - fd.pos).norm())
        };
        let via = |at| {
            let Some(i) = relay else { return f64::INFINITY };
            let rp = self.f[i].pos;
            if (base_pos(base) - rp).norm() >= (base_pos(base) - fd.pos).norm() {
                return f64::INFINITY;
            }
            let (t, v) = release(rp, at);
            t + throw_time(v, (rp - fd.pos).norm()) + RELAY_TRANSFER_FRAC * self.f[i].a.transfer + throw_time(THROW_EFFORT * self.f[i].a.arm, (base_pos(base) - rp).norm())
        };
        let d = direct(self.t);
        let v = via(self.t);
        let through = v < d;
        let leg = if through { self.f[relay.unwrap()].pos } else { base_pos(base) };
        let at_set = (fd.up_at + self.set_up_for(by, leg).set_s).max(self.t);
        let receiver = self.receiver_for(by, base);
        let receiver_at = self.receiver_ready(receiver, base);
        ThrowRace {
            direct: d,
            via: v,
            relay,
            receiver,
            receiver_at,
            through,
            ball_at: d.min(v).max(receiver_at) + 0.05,
            set_ball_at: (if through { via(at_set) } else { direct(at_set) }).max(receiver_at) + 0.05,
        }
    }

    /// One subsequent force at first. It survives a force elsewhere; retiring the
    /// batter first removes it. No private intention or future physical path is read.
    fn force_continuation(&self, by: usize, option: &HolderOption) -> Option<(f64, f64)> {
        if self.st.outs >= 2 || !option.force || option.base <= 1 {
            return None;
        }
        let fd = &self.f[by];
        let batter = self.r.iter().find(|r| r.start == 0 && !r.out && !r.scored && r.overrun == OverrunPhase::None)?;
        if visible_target(batter, fd.a.throw_choice) != Some(1) {
            return None;
        }
        let runner_at = self.t + runner_eta(BASE_L - batter.s, batter.v);
        let (next_holder, release_at, speed) = if option.carry {
            // The carrier still has the same setup clock after touching the bag.
            let to = base_pos(1) - base_pos(option.base);
            let turn = fd.face.map(|face| throwing::turn_extra_s(face, to, fd.a.throws)).unwrap_or(0.0);
            let su = SetUp::new(fd.set_base, turn, throwing::footwork_s(fd.catch_vel, to, fd.took_diving));
            let release_at = option.ball_at.max(fd.up_at + su.earliest_s);
            (by, release_at, THROW_EFFORT * fd.a.arm * su.speed_mult(su.hurry(release_at - fd.up_at)))
        } else {
            let next = &self.f[option.receiver];
            (option.receiver, option.ball_at + RELAY_TRANSFER_FRAC * next.a.transfer, THROW_EFFORT * next.a.arm)
        };
        let receiver = self.receiver_for(next_holder, 1);
        let ball_at = (release_at + throw_time(speed, (base_pos(1) - base_pos(option.base)).norm())).max(self.receiver_ready(receiver, 1)) + 0.05;
        Some((phi((runner_at - ball_at) / 0.12), ball_at))
    }

    pub(super) fn holder_options(&self, by: usize) -> Vec<HolderOption> {
        let fd = &self.f[by];
        let params = fd.a.throw_choice;
        let mut options = Vec::new();
        for (k, r) in self.r.iter().enumerate() {
            if r.out || r.scored || r.overrun != OverrunPhase::None {
                continue;
            }
            // Crucially, route/receiver optimization cannot invent a destination.
            let Some(base) = visible_target(r, params) else { continue };
            let runner_at = self.t + runner_eta((base as f64 * BASE_L - r.s).abs(), r.v);
            let force = self.forced(k) && base == r.start + 1 && r.s < base as f64 * BASE_L;
            let target = base_pos(base);
            let distance = (target - fd.pos).norm();
            let race = self.throw_race(by, base);
            let mut add = |carry: bool, ball_at, set_ball_at| {
                let p_out = phi((runner_at - ball_at) / 0.12);
                let weight = self.out_weight(k, base);
                let mut option = HolderOption {
                    runner: k,
                    base,
                    carry,
                    force,
                    through: !carry && race.through,
                    receiver: if carry { by } else { race.receiver },
                    runner_at,
                    ball_at,
                    set_ball_at,
                    direct_at: race.direct,
                    via_at: race.via,
                    receiver_at: race.receiver_at,
                    distance,
                    p_out,
                    weight,
                    value: p_out * p_out * weight,
                    continuation: None,
                };
                option.continuation = self.force_continuation(by, &option);
                if let Some((p_second, _)) = option.continuation {
                    let w_second = if self.st.outs + 1 >= 2 { 1.0 } else { 0.9 };
                    option.value += params.continuation_weight * p_out * p_out * p_second * p_second * w_second;
                }
                options.push(option);
            };
            // A carry to a forced runner's bag is a force touch. To an unforced runner's bag
            // it is a tag plan: be on the bag before him and tag him as he comes in (the tag
            // itself is glove/runner contact within `tag_reach_m` in holder_act). It is
            // weighed as a tag (`out_weight`, no force continuation), never as a touch.
            if distance < params.carry_max_m {
                let ready = if fd.dove { fd.ready_at } else { fd.up_at };
                // A tag (not a force) also needs the time to put it on (§6.4).
                let tag = if force { 0.0 } else { fd.a.tag_apply_s };
                let ball_at = self.t.max(ready) + run_time_to(fd, target, 0.6) + tag;
                add(true, ball_at, ball_at);
            }
            let tag = if force { 0.0 } else { self.f[race.receiver].a.tag_apply_s };
            add(false, race.ball_at + tag, race.set_ball_at + tag);
        }
        options
    }
}

impl HolderOption {
    pub(super) fn evidence(self, sim: &Sim<'_>) -> choice_trace::OptionEstimate {
        choice_trace::OptionEstimate {
            runner: sim.r[self.runner].id,
            base: self.base,
            kind: if self.carry { "carry" } else { "throw" },
            force: self.force,
            runner_at: self.runner_at,
            ball_at: self.ball_at,
            direct_at: (!self.carry).then_some(self.direct_at),
            via_at: (!self.carry).then_some(self.via_at),
            set_at: Some(self.set_ball_at),
            receiver: Some(sim.f[self.receiver].id),
            receiver_at: (!self.carry).then_some(self.receiver_at),
            p_out: Some(self.p_out),
            weight: Some(self.weight),
            value: Some(self.value),
            through: self.through,
            distance: self.distance,
            second_out_p: self.continuation.map(|c| c.0),
            second_ball_at: self.continuation.map(|c| c.1),
        }
    }
}
