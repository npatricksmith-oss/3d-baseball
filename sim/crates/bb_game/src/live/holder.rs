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
        // The cutoff man as the holder knows him (`seen`: a job taken in the last
        // `CALL_HEARD_S` is not yet known; Codex fielder-communication recheck P1-1).
        let relay = (0..9).find(|&i| i != by && matches!(self.seen(by, i).0, Role::Relay(_)) && (self.f[i].pos - fd.pos).norm() > 10.0);
        // A cutoff man lined up between him and the base (closer to it than he is): the relay
        // route through him, as before; and, when he has the job for this base and stands on
        // its line, the direct throw must pass him low (`cutoff_out`, agent-models §5.5).
        let between = relay.filter(|&i| (base_pos(base) - self.f[i].pos).norm() < (base_pos(base) - fd.pos).norm());
        let cut = self.cut_seen(Some(by), fd.pos, base);
        let release = |target: Vec3, at: f64| {
            let su = self.set_up_for(by, target);
            let t = at.max(fd.up_at + su.earliest_s);
            (t, self.throw_speed(by, target, su.hurry(t - fd.up_at)) * su.speed_mult(su.hurry(t - fd.up_at)))
        };
        // With a cutoff man in the throw it must pass him low enough to be cut (agent-models
        // §5.5): the cuttable throw, by the throw and bounce physics.
        let direct = |at| {
            let (t, v) = release(base_pos(base), at);
            t + direct_time(v, (base_pos(base) - fd.pos).norm(), cut)
        };
        let via = |at| {
            let Some(i) = between else { return f64::INFINITY };
            let rp = self.f[i].pos;
            let (t, v) = release(rp, at);
            t + throw_time(v, (rp - fd.pos).norm()) + throwing::relay_s(self.f[i].a.transfer) + throw_time(THROW_EFFORT * self.f[i].a.arm, (base_pos(base) - rp).norm())
        };
        let d = direct(self.t);
        let v = via(self.t);
        // The route: time, and what each route's throws are likely to cost the man taking them
        // off line (agent-models §5.5): one long throw misses by more than two short ones.
        // (A man lined up for another base: by time alone, as before.)
        let through = between.is_some_and(|i| {
            if cut == Cut::No {
                return v < d;
            }
            let rp = self.f[i].pos;
            let miss_direct = miss_cost_s((base_pos(base) - fd.pos).norm(), fd.a.arm_sigma);
            let miss_via = miss_cost_s((rp - fd.pos).norm(), fd.a.arm_sigma) + miss_cost_s((base_pos(base) - rp).norm(), self.f[i].a.arm_sigma);
            v + miss_via < d + miss_direct
        });
        let leg = if through { self.f[relay.unwrap()].pos } else { base_pos(base) };
        let at_set = (fd.up_at + self.set_up_for(by, leg).set_s).max(self.t);
        let receiver = self.receiver_for(by, base);
        // He is ready for the throw the holder would make, timed to it (a man covering a force
        // base crosses it unless he is early for the throw: agent-models §5.5).
        let receiver_at = self.receiver_ready(receiver, base, if through { v } else { d });
        ThrowRace {
            direct: d,
            via: v,
            relay,
            receiver,
            receiver_at,
            through,
            ball_at: (if through { v } else { d }).max(receiver_at) + 0.05,
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
        let (next_holder, release_at, speed) = if option.base == 2 {
            // A double-play pivot at second, by the same model as the world's (agent-models
            // §5.5), from what the holder sees.
            let pivot = if option.carry { by } else { option.receiver };
            let pf = &self.f[pivot];
            let bag = base_pos(2);
            let approach = Vec3::new(bag.x - pf.pos.x, bag.y - pf.pos.y, 0.0).normalized();
            // He gets the bag at the later of the feed's arrival and his own arrival there, by
            // the movement he will make (`cover_reach`, timed to this feed); a carrier brakes
            // onto it at √(2·BRAKE·0.6 m) at the touch radius.
            let (t_bag, arriving) = if (bag - pf.pos).norm() < BAG_TOUCH_M {
                (option.ball_at, None)
            } else if option.carry {
                (option.ball_at, Some(approach * m::sqrt(2.0 * BRAKE * BAG_TOUCH_M)))
            } else {
                let (t_touch, v_touch) = self.cover_reach(pivot, 2, option.ball_at - 0.05, BAG_TOUCH_M);
                (option.ball_at.max(t_touch), (t_touch + 0.3 >= option.ball_at).then_some(v_touch))
            };
            (pivot, self.pivot_release(pivot, fd.pos, t_bag, arriving), THROW_EFFORT * pf.a.arm)
        } else if option.carry {
            // The carrier still has the same setup clock after touching the bag.
            let to = base_pos(1) - base_pos(option.base);
            let turn = fd.face.map(|face| throwing::turn_extra_s(face, to, fd.a.throws)).unwrap_or(0.0);
            let footwork = throwing::footwork_s(fd.catch_vel, to, fd.took_diving);
            let drift = if fd.took_diving { f64::INFINITY } else { throwing::off_line_mps(fd.catch_vel, to) };
            let su = if fd.relayed { SetUp::relay(fd.set_base, throwing::exchange_footwork_s(fd.catch_vel, to, fd.took_diving)) } else { SetUp::new(fd.set_base, turn, footwork).with_drift(drift) };
            let release_at = option.ball_at.max(fd.up_at + su.earliest_s);
            (by, release_at, self.throw_speed(by, base_pos(1), su.hurry(release_at - fd.up_at)) * su.speed_mult(su.hurry(release_at - fd.up_at)))
        } else {
            let next = &self.f[option.receiver];
            (option.receiver, option.ball_at + RELAY_TRANSFER_FRAC * next.a.transfer, THROW_EFFORT * next.a.arm)
        };
        let receiver = self.receiver_for(next_holder, 1);
        let arrives = release_at + throw_time(speed, (base_pos(1) - base_pos(option.base)).norm());
        let ball_at = arrives.max(self.receiver_ready(receiver, 1, arrives)) + 0.05;
        Some((phi((runner_at - ball_at) / RACE_SIGMA_S), ball_at))
    }

    /// When the pivot man at second lets the ball go for first, as a teammate judges it
    /// (agent-models §5.5): he has the bag at `t_bag`; still `arriving` at it, he takes it moving
    /// with that velocity (braking onto it, or crossing it, by the movement law: `cover_reach`)
    /// and faces the way he moves (faster than `PIVOT_FACE_MIN_MPS`), else the feed; already
    /// there, he stands facing the feed from `feed_from`. His `Transfer` is known to his
    /// teammates.
    pub(super) fn pivot_release(&self, pivot: usize, feed_from: Vec3, t_bag: f64, arriving: Option<Vec3>) -> f64 {
        let pf = &self.f[pivot];
        let bag = base_pos(2);
        let to_first = base_pos(1) - bag;
        let feed_face = Vec3::new(feed_from.x - bag.x, feed_from.y - bag.y, 0.0).normalized();
        let (face, vel) = match arriving {
            Some(v) => {
                let v = Vec3::new(v.x, v.y, 0.0);
                (if v.norm() > PIVOT_FACE_MIN_MPS { v.normalized() } else { feed_face }, v)
            }
            None => (feed_face, Vec3::ZERO),
        };
        let su = SetUp::pivot(throwing::pivot_s(pf.a.transfer), throwing::turn_extra_s(face, to_first, pf.a.throws), throwing::exchange_footwork_s(vel, to_first, false));
        t_bag + su.set_s
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
                let p_out = phi((runner_at - ball_at) / RACE_SIGMA_S);
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
                // Only a second out whose throw would be attempted (p > `RACE_MIN_P`, the holder's own
                // rule) adds to the value (Gemini A4 review P1-2).
                if let Some((p_second, _)) = option.continuation.filter(|c| c.0 > RACE_MIN_P) {
                    let w_second = if self.st.outs + 1 >= 2 { 1.0 } else { 0.9 };
                    option.value += params.continuation_weight * p_out * p_out * p_second * p_second * w_second;
                }
                options.push(option);
            };
            let carry_ready = if fd.dove { fd.ready_at } else { fd.up_at };
            let carry_tag = if force { 0.0 } else { fd.a.tag_apply_s };
            let carry_ball_at = self.t.max(carry_ready) + run_time_to(self.inp.field, fd, target, 0.6) + carry_tag;
            let throw_tag = if force { 0.0 } else { self.f[race.receiver].a.tag_apply_s };
            let throw_ball_at = race.ball_at + throw_tag;
            // A carry to a forced runner's bag is a force touch. To an unforced runner's bag
            // it is a tag plan: be on the bag before him and tag him as he comes in (the tag
            // itself is glove/runner contact within `tag_reach_m` in holder_act). It is
            // weighed as a tag (`out_weight`, no force continuation), never as a touch.
            // Admitted within `carry_max_m`, or from further away (e.g. 1B playing Standard
            // depth ~11.1 m off first) when carrying reaches the bag sooner than throwing to
            // the covering receiver.
            if params.carry_max_m > 0.0 && (distance < params.carry_max_m || carry_ball_at < throw_ball_at) {
                add(true, carry_ball_at, carry_ball_at);
            }
            add(false, throw_ball_at, race.set_ball_at + throw_tag);
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
