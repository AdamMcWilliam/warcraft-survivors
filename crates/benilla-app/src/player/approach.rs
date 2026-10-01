//! Click to Move's approach (`AutoInteract`, "Toggles auto-move to interact target"): with the
//! option on, a right-click on an NPC, a body to loot or skin, or a GameObject beyond the verb's
//! reach walks the player there, and the verb runs on arrival. It is [`super::follow`]'s auto-move:
//! one cell holds either (mode `0xc4d888`, guid `0xc4d980`), so arming one ends the other
//! (`0x611130` calls the canceller `0x60fb60` first), the same input cancels both, and nothing goes
//! on the wire but the movement. The terrain and sky walks (modes 4, 8) and the attack approach
//! (mode 10) are not built.

use std::f32::consts::PI;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use benilla_protocol::EntityKind;

use crate::net::{Embodied, Guid, GuidIndex, NetEntity, ObjectStore, SelfPlayer, UnitSpeeds};

use super::follow::{self, FollowInput, FollowState};
use super::state::Player;

/// The verb an approach owes, by its mode in `0xc4d888`; the pending dispatcher `0x60fa20` runs it
/// at the stop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ApproachVerb {
    /// Mode 5: the NPC-service dispatch `0x5f0130`.
    Talk,
    /// Mode 6: the unit loot `0x5df2a0`.
    Loot,
    /// Mode 7: the GameObject use `0x5f86b0`.
    Use,
    /// Mode 9: the skin cast `0x5f05e0`.
    Skin,
}

/// What the start gate (`0x60fed0`) checks the target for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Subject {
    Unit { dead: bool },
    Corpse,
    GameObject,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Refused {
    Silent,
    /// `ERR_AUTOFOLLOW_TOO_FAR` (`0x126`, `0x61110c`).
    TooFar,
}

/// Talk stops at half the 5.5556 yd service reach (`0x60e680`).
pub(crate) const TALK_STOP: f32 = 5.555_555_3 * 0.5;
/// Use and Skin stop at this fraction of the verb's range (`0x804588`).
pub(crate) const RANGE_STOP_FRACTION: f32 = 0.9;
/// No approach starts or lasts at 80 yd or more: modes 5-10's row in `0x860a58` (`0x60e5d1`).
const LEASH_SQ: f32 = 6400.0;
/// Within this many yards on the ground the goal's height is ignored (`0x80c4c8`).
const FLAT_SNAP: f32 = 0.5;
/// `automoveturnspeednarrow` and `automoveturnspeedwide`, deg/s (`0x6039e9`, `0x6039ca`).
const TURN_NARROW: f32 = 800.0;
const TURN_WIDE: f32 = 1200.0;
/// Less ground than this in a frame while lined up and holding forward is stuck (`0x80c4dc`).
const STUCK: f32 = 1.0 / 360.0;

/// The approach in flight, and the verb an arrival owes.
#[derive(Resource, Default)]
pub(crate) struct Approach {
    /// `AutoInteract`, `CanAutoInteract`'s last term (`0x60f925`).
    pub(crate) enabled: bool,
    goal: Option<Goal>,
    /// The pending cells `0x60f940` fills (`0xc4da28`, `0xc4da78`, `0xc4d950`, `0xc4da20`).
    pub(crate) arrived: Option<(ApproachVerb, u64)>,
}

struct Goal {
    verb: ApproachVerb,
    guid: u64,
    /// `0xc4d910`.
    stop: f32,
    /// Latched by the first turn (`0x61042c`).
    turn_rate: Option<f32>,
    /// Bit 1 of `0xc4da48`: the facing has lined up once.
    aligned: bool,
    /// Last frame's position (`0xc4da84`).
    last_pos: Vec3,
    held: bool,
}

impl Approach {
    /// The reference's `0x6103a0` for this mode family: an approach is in flight.
    pub(crate) fn active(&self) -> bool {
        self.goal.is_some()
    }

    pub(crate) fn stop(&mut self) {
        self.goal = None;
    }

    #[cfg(test)]
    pub(crate) fn stop_distance(&self) -> Option<f32> {
        self.goal.as_ref().map(|g| g.stop)
    }
}

/// `CanAutoInteract` (`0x60f900`): alive, driving our own body, and the option on.
pub(crate) fn can_auto_interact(enabled: bool, me: Option<&ObjectStore>, player: &Player) -> bool {
    enabled
        && me.is_some_and(|s| s.0.unit_health().unwrap_or(0) > 0)
        && player.foreign_mover.is_none()
        && !player.control_lost
}

/// The start: the gate (`0x60fed0`, `0x610300`) and the arm `0x611130`.
#[derive(SystemParam)]
pub(crate) struct AutoMove<'w, 's> {
    pub(crate) approach: ResMut<'w, Approach>,
    follow: ResMut<'w, FollowState>,
    player: Res<'w, Player>,
    me: Query<'w, 's, (&'static Guid, &'static ObjectStore), With<SelfPlayer>>,
    stand: MessageWriter<'w, super::StandStateRequest>,
}

impl AutoMove<'_, '_> {
    pub(crate) fn can_auto_interact(&self) -> bool {
        can_auto_interact(
            self.approach.enabled,
            self.me.single().ok().map(|(_, s)| s),
            &self.player,
        )
    }

    pub(crate) fn start(
        &mut self,
        verb: ApproachVerb,
        guid: u64,
        subject: Subject,
        at: Vec3,
        stop: f32,
    ) -> Result<(), Refused> {
        let Ok((me, store)) = self.me.single() else {
            return Err(Refused::Silent);
        };
        let admitted = match (verb, subject) {
            (ApproachVerb::Talk, Subject::Unit { .. }) => true,
            (ApproachVerb::Loot | ApproachVerb::Skin, Subject::Corpse) => true,
            (ApproachVerb::Loot | ApproachVerb::Skin, Subject::Unit { dead }) => dead,
            (ApproachVerb::Use, Subject::GameObject) => true,
            _ => false,
        };
        if !admitted
            || guid == me.0
            || self.player.foreign_mover.is_some()
            || self.player.control_lost
            || store.0.unit_health().unwrap_or(0) == 0
        {
            return Err(Refused::Silent);
        }
        let standing = store.0.unit_stand_state() == 0;
        self.follow.stop();
        self.approach.stop();
        if self.player.pos.distance_squared(at) >= LEASH_SQ {
            return Err(Refused::TooFar);
        }
        info!("approach: {verb:?} {guid:#x}, stopping {stop:.2} yd short");
        self.approach.goal = Some(Goal {
            verb,
            guid,
            stop,
            turn_rate: None,
            aligned: false,
            last_pos: self.player.pos,
            held: false,
        });
        if !standing {
            self.stand.write(super::StandStateRequest { state: 0 });
        }
        Ok(())
    }
}

fn speed_scale(speed: f32) -> f32 {
    (speed / follow::SPEED_NORM).max(1.0)
}

/// The goal distance, squared (`0x610e40`'s tail): on the ground alone once inside the snap
/// radius, unless swimming. Talk's radius never scales with speed.
fn goal_distance_sq(verb: ApproachVerb, delta: Vec3, speed: f32, swimming: bool) -> f32 {
    let snap = if verb == ApproachVerb::Talk {
        FLAT_SNAP
    } else {
        FLAT_SNAP * speed_scale(speed)
    };
    let flat = delta.x * delta.x + delta.z * delta.z;
    if !swimming && flat <= snap * snap {
        flat
    } else {
        delta.length_squared()
    }
}

/// The arrive distance (`0x610add`-`0x610b07`): the stop distance scaled by the mover's current
/// speed over 7 (`0x7c4c90`, 0 at rest), never below 1×; Talk's is never scaled.
fn arrive_distance(verb: ApproachVerb, stop: f32, speed: f32) -> f32 {
    if verb == ApproachVerb::Talk {
        stop
    } else {
        stop * speed_scale(speed)
    }
}

/// The rate the first turn latches, rad/s: wide for a turn of more than 120° (`0x61043f`).
fn turn_rate(remaining: f32) -> f32 {
    let deg = if remaining.abs() > 2.0 * PI / 3.0 {
        TURN_WIDE
    } else {
        TURN_NARROW
    };
    deg.to_radians()
}

/// Steer toward the goal and hold forward; on arrival face it and owe its verb. After
/// [`follow::steer_follow`], which clears [`Player::auto_forward`].
pub(super) fn steer_approach(
    time: Res<Time>,
    mut approach: ResMut<Approach>,
    mut player: ResMut<Player>,
    mover: Query<&UnitSpeeds, With<Embodied>>,
    index: Res<GuidIndex>,
    targets: Query<(&Transform, Option<&ObjectStore>, Option<&NetEntity>)>,
    input: FollowInput,
) {
    let Some(goal) = approach.goal.as_mut() else {
        return;
    };
    // `GetCurrentSpeed` on last frame's word, as the reference reads the mover's own.
    let speed = mover
        .single()
        .map_or(0.0, |s| crate::net::current_speed(&s.0, player.move_flags));
    if input.cancels(&player) {
        info!("approach: cancelled by the player's own movement input");
        approach.stop();
        return;
    }
    let Some((tf, store, kind)) = index.0.get(&goal.guid).and_then(|e| targets.get(*e).ok()) else {
        info!("approach: the target is gone");
        approach.stop();
        return;
    };
    // `0x610f13`-`0x610f40`: a unit on a taxi ends it, a dead one too unless the verb wants a body.
    if kind.is_some_and(|k| matches!(k.kind, EntityKind::Unit | EntityKind::Player)) {
        let store = store.map(|s| &s.0);
        let taxi = store.is_some_and(|s| s.unit_flags() & super::UNIT_FLAG_TAXI_FLIGHT != 0);
        let dead = store.is_none_or(|s| s.unit_health().unwrap_or(0) == 0);
        if taxi || (dead && !matches!(goal.verb, ApproachVerb::Loot | ApproachVerb::Skin)) {
            info!("approach: the target left reach (taxi {taxi}, dead {dead})");
            approach.stop();
            return;
        }
    }
    let target = tf.translation;
    let delta = target - player.pos;
    let d2 = goal_distance_sq(goal.verb, delta, speed, player.swimming);
    if d2 >= LEASH_SQ {
        info!("approach: beyond 80 yd");
        approach.stop();
        return;
    }
    // `0x61073c`: once lined up, a frame that ends farther off than the last aborts, but not Talk.
    if goal.aligned
        && goal.verb != ApproachVerb::Talk
        && goal.last_pos.distance_squared(target) < player.pos.distance_squared(target)
    {
        info!("approach: no longer closing");
        approach.stop();
        return;
    }
    let flat = Vec3::new(delta.x, 0.0, delta.z);
    let bearing = follow::bearing_to(flat);
    let remaining = follow::wrap_pi(bearing - player.face_yaw);
    let turning = remaining.abs() > follow::TURN_DEADZONE;
    if turning {
        let rate = *goal.turn_rate.get_or_insert_with(|| turn_rate(remaining));
        player.face_yaw = follow::steer(player.face_yaw, bearing, rate, time.delta_secs());
    } else {
        goal.aligned = true;
    }
    let arrive = arrive_distance(goal.verb, goal.stop, speed);
    if d2 <= arrive * arrive {
        info!("approach: arrived at {:.2} yd", d2.sqrt());
        // The arrival's canceller faces the target (`0x60fbe8`).
        if flat.length_squared() > f32::EPSILON {
            player.face_yaw = bearing;
        }
        approach.arrived = Some((goal.verb, goal.guid));
        approach.goal = None;
        return;
    }
    // Rooted, the reference lets go of forward (`0x610b4a`), which also spares the stuck test.
    let hold = !player.modes.rooted;
    if !turning && goal.held && player.pos.distance_squared(goal.last_pos) < STUCK * STUCK {
        info!("approach: stuck");
        approach.stop();
        return;
    }
    if follow::vertically_degenerate(delta) {
        info!("approach: the target is straight overhead");
        approach.stop();
        return;
    }
    goal.last_pos = player.pos;
    goal.held = hold;
    player.auto_forward = hold;
}

fn on_cvar(ev: On<crate::cvars::CvarChanged>, mut approach: ResMut<Approach>) {
    if ev.is("AutoInteract") {
        approach.enabled = ev.flag();
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Approach>().add_observer(on_cvar);
}

#[cfg(test)]
mod tests {
    use super::super::state::MoveSpeed;
    use super::*;
    use benilla_protocol::messages::ObjectFields;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn talk_stops_inside_the_service_reach_and_never_scales() {
        assert!((TALK_STOP - 2.777_777_7).abs() < 1e-5);
        assert_eq!(
            arrive_distance(ApproachVerb::Talk, TALK_STOP, 14.0),
            TALK_STOP
        );
        assert_eq!(arrive_distance(ApproachVerb::Loot, 5.0, 7.0), 5.0);
        assert_eq!(arrive_distance(ApproachVerb::Loot, 5.0, 14.0), 10.0);
        assert_eq!(arrive_distance(ApproachVerb::Use, 5.0, 3.5), 5.0);
        assert_eq!(arrive_distance(ApproachVerb::Use, 5.0, 0.0), 5.0, "at rest");
    }

    #[test]
    fn the_goal_flattens_only_inside_the_snap_radius() {
        let above = Vec3::new(0.3, 4.0, 0.3);
        assert!((goal_distance_sq(ApproachVerb::Use, above, 7.0, false) - 0.18).abs() < 1e-5);
        assert!((goal_distance_sq(ApproachVerb::Use, above, 7.0, true) - 16.18).abs() < 1e-4);
        let far = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(goal_distance_sq(ApproachVerb::Use, far, 7.0, false), 25.0);
    }

    const ME: u64 = 0x1;
    const VENDOR: u64 = 0xF00D;
    const DT: f32 = 1.0 / 60.0;

    /// Our body at the origin facing -Z, alive (and seated when `stand_state` is set), a vendor at
    /// `at`, and both steers chained as the plugin runs them.
    fn world(at: Vec3, stand_state: u32) -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<Approach>()
            .init_resource::<FollowState>()
            .init_resource::<Player>()
            .init_resource::<GuidIndex>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<crate::bindings::BindingsState>()
            .init_resource::<super::super::camera::CameraControl>()
            .init_resource::<super::super::view_subject::ViewSubject>()
            .insert_resource(MoveSpeed {
                value: 7.0,
                env_override: false,
            })
            .add_message::<super::super::StandStateRequest>()
            .add_systems(Update, (follow::steer_follow, steer_approach).chain());
        app.world_mut().resource_mut::<Approach>().enabled = true;
        app.world_mut().spawn((
            SelfPlayer,
            Guid(ME),
            ObjectStore(ObjectFields::from_pairs(&[
                (22, 100),
                (28, 100),
                (138, stand_state),
            ])),
        ));
        let vendor = app
            .world_mut()
            .spawn((Guid(VENDOR), Transform::from_translation(at)))
            .id();
        app.world_mut()
            .resource_mut::<GuidIndex>()
            .0
            .insert(VENDOR, vendor);
        app
    }

    fn start(app: &mut App, verb: ApproachVerb, subject: Subject, at: Vec3) -> Result<(), Refused> {
        app.world_mut()
            .run_system_once(move |mut auto: AutoMove| {
                auto.start(verb, VENDOR, subject, at, TALK_STOP)
            })
            .unwrap()
    }

    /// One frame: the steers, then the controller's walk along the facing while forward is held.
    fn frame(app: &mut App, walks: bool) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(DT));
        app.update();
        let mut player = app.world_mut().resource_mut::<Player>();
        if walks && player.auto_forward {
            let forward = Quat::from_rotation_y(player.face_yaw) * Vec3::NEG_Z;
            player.pos += forward * 7.0 * DT;
        }
    }

    #[test]
    fn a_vendor_out_of_reach_is_walked_to_and_owed_the_talk() {
        let at = Vec3::new(14.0, 0.0, 0.0);
        let mut app = world(at, 0);
        assert_eq!(
            start(
                &mut app,
                ApproachVerb::Talk,
                Subject::Unit { dead: false },
                at
            ),
            Ok(())
        );
        frame(&mut app, true);
        assert!(
            app.world().resource::<Player>().auto_forward,
            "forward from the first frame"
        );
        let mut frames = 1;
        while app.world().resource::<Approach>().arrived.is_none() {
            assert!(frames < 300, "never arrived");
            frame(&mut app, true);
            frames += 1;
        }
        let player = app.world().resource::<Player>();
        let left = player.pos.distance(at);
        assert!(
            left <= TALK_STOP && left > TALK_STOP - 7.0 * DT - 1e-3,
            "stopped {left} yd off, not at the 2.78 yd stop"
        );
        assert!(!player.auto_forward, "forward let go at the stop");
        assert!(
            (follow::wrap_pi(player.face_yaw - follow::bearing_to(at - player.pos))).abs() < 1e-4,
            "facing the vendor"
        );
        let approach = app.world().resource::<Approach>();
        assert_eq!(approach.arrived, Some((ApproachVerb::Talk, VENDOR)));
        assert!(!approach.active());
    }

    #[test]
    fn the_arm_ends_a_follow_and_refuses_past_80_yards() {
        let near = Vec3::new(14.0, 0.0, 0.0);
        let mut app = world(near, 0);
        app.world_mut()
            .resource_mut::<FollowState>()
            .start(0xBEEF, "Friend".into());
        assert_eq!(
            start(
                &mut app,
                ApproachVerb::Talk,
                Subject::Unit { dead: false },
                near
            ),
            Ok(())
        );
        assert!(
            app.world().resource::<FollowState>().guid.is_none(),
            "one auto-move cell"
        );

        let far = Vec3::new(80.0, 0.0, 0.0);
        assert_eq!(
            start(
                &mut app,
                ApproachVerb::Talk,
                Subject::Unit { dead: false },
                far
            ),
            Err(Refused::TooFar)
        );
        assert!(
            !app.world().resource::<Approach>().active(),
            "the arm cancelled the last walk"
        );
    }

    #[test]
    fn the_gate_admits_each_verb_its_own_kind_of_target() {
        let at = Vec3::new(14.0, 0.0, 0.0);
        let mut app = world(at, 0);
        for (verb, subject, ok) in [
            (ApproachVerb::Talk, Subject::Unit { dead: false }, true),
            (ApproachVerb::Talk, Subject::GameObject, false),
            (ApproachVerb::Loot, Subject::Unit { dead: true }, true),
            (ApproachVerb::Loot, Subject::Unit { dead: false }, false),
            (ApproachVerb::Loot, Subject::Corpse, true),
            (ApproachVerb::Skin, Subject::Unit { dead: false }, false),
            (ApproachVerb::Use, Subject::GameObject, true),
            (ApproachVerb::Use, Subject::Unit { dead: false }, false),
        ] {
            let got = start(&mut app, verb, subject, at);
            assert_eq!(got.is_ok(), ok, "{verb:?} at {subject:?}: {got:?}");
        }
    }

    #[test]
    fn a_seated_player_stands_to_walk() {
        let at = Vec3::new(14.0, 0.0, 0.0);
        let mut app = world(at, 1);
        assert_eq!(
            start(
                &mut app,
                ApproachVerb::Talk,
                Subject::Unit { dead: false },
                at
            ),
            Ok(())
        );
        let asks: Vec<u8> = app
            .world_mut()
            .resource_mut::<Messages<super::super::StandStateRequest>>()
            .drain()
            .map(|m| m.state)
            .collect();
        assert_eq!(asks, vec![0]);
    }

    #[test]
    fn the_players_own_movement_cancels_the_walk() {
        let at = Vec3::new(14.0, 0.0, 0.0);
        let mut app = world(at, 0);
        assert_eq!(
            start(
                &mut app,
                ApproachVerb::Talk,
                Subject::Unit { dead: false },
                at
            ),
            Ok(())
        );
        frame(&mut app, true);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Forward);
        frame(&mut app, true);
        let approach = app.world().resource::<Approach>();
        assert!(!approach.active() && approach.arrived.is_none());
    }

    #[test]
    fn a_walk_that_cannot_move_gives_up_once_lined_up() {
        let at = Vec3::new(14.0, 0.0, 0.0);
        let mut app = world(at, 0);
        assert_eq!(
            start(&mut app, ApproachVerb::Use, Subject::GameObject, at),
            Ok(())
        );
        frame(&mut app, false);
        frame(&mut app, false);
        assert!(
            app.world().resource::<Approach>().active(),
            "still turning, where standing still is not stuck"
        );
        for _ in 0..60 {
            frame(&mut app, false);
        }
        let approach = app.world().resource::<Approach>();
        assert!(!approach.active(), "stuck against a wall");
        assert!(approach.arrived.is_none(), "nothing owed");
    }

    #[test]
    fn the_first_turn_picks_the_wide_rate_past_120_degrees() {
        assert_eq!(turn_rate(0.5), 800f32.to_radians());
        assert_eq!(turn_rate(-2.0 * PI / 3.0), 800f32.to_radians());
        assert_eq!(turn_rate(2.2), 1200f32.to_radians());
        assert_eq!(turn_rate(-PI), 1200f32.to_radians());
    }
}
