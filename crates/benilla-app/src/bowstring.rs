//! The engine-drawn bowstring: bow M2s have no string geometry. The reference's per-frame callback
//! (`0x611ff0`) draws a two-segment line between the `$WTT`/`$WTB` limb-tip markers, its middle
//! vertex at the HandArrow attach while the nock latch (`[+0xd58] & 0x4000`) holds, else the tip
//! midpoint. Drawn here as gizmo lines; the color and width are inferred, since the reference's
//! packed vertex color is not decoded.
//!
//! The tips are posed: `$BWP` plays BowPull (160) on the prop and `$BWR` returns it to Stand (0),
//! so the markers ride limb bones that bend through the draw.

use bevy::prelude::*;

use crate::creature_anim::NockLatch;
use crate::entities::BoneAttach;

/// The HandArrow attach id (35, `0x6121b8`): the string's middle point while nocked.
const HAND_ARROW: u16 = 0x23;

/// Marks a bow prop root whose model authors the `$WTT`/`$WTB` anchors; despawned with the prop.
#[derive(Component)]
pub(crate) struct Bowstring {
    /// The unit wearing the bow, which carries the HandArrow attach and the nock latch.
    pub(crate) owner: Entity,
    /// The `$WTT` top and `$WTB` bottom anchors as `(bone, model-local offset)`.
    pub(crate) top: (u16, Vec3),
    pub(crate) bottom: (u16, Vec3),
    /// The two anchor bones' rest pivots, model-local: a posed bone frame takes the point from its
    /// pivot, as [`benilla_assets::ClipEvent::offset`] does.
    pub(crate) pivots: [Vec3; 2],
}

/// A joint's rest pivot, model-local: the rest translations summed up its parent chain.
pub(crate) fn rest_pivot(skeleton: &benilla_assets::ModelSkeleton, bone: u16) -> Vec3 {
    let mut pivot = Vec3::ZERO;
    let mut at = Some(bone as usize);
    // Bounded by the joint count, so a malformed parent loop cannot hang.
    for _ in 0..skeleton.joints.len() {
        let Some(joint) = at.and_then(|i| skeleton.joints.get(i)) else {
            break;
        };
        pivot += joint.local_translation;
        at = usize::try_from(joint.parent).ok();
    }
    pivot
}

/// Draws every visible bow's string, tip to middle to tip, from this frame's joint frames.
fn draw_bowstrings(
    bows: Query<(
        &Bowstring,
        &GlobalTransform,
        &InheritedVisibility,
        Option<&benilla_world::rig_anim::RigPose>,
    )>,
    owners: Query<(
        &BoneAttach,
        &benilla_world::rig_anim::RigPose,
        Has<NockLatch>,
    )>,
    joints: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    // Inferred: the reference's packed vertex color is not decoded.
    const STRING_COLOR: Color = Color::srgb(0.12, 0.10, 0.08);
    for (bs, prop, vis, flex) in &bows {
        if !vis.get() {
            continue;
        }
        // The prop's own pose when it flexes (`joints_root` is this entity); its rigid frame
        // otherwise.
        let tip = |(bone, offset): (u16, Vec3), pivot: Vec3| {
            flex.and_then(|p| p.posed_point(prop, bone, offset - pivot))
                .unwrap_or_else(|| prop.transform_point(offset))
        };
        let top = tip(bs.top, bs.pivots[0]);
        let bottom = tip(bs.bottom, bs.pivots[1]);
        let middle = owners
            .get(bs.owner)
            .ok()
            .filter(|(_, _, latched)| *latched)
            .and_then(|(bones, pose, _)| {
                let &(bone, offset) = bones.points.get(&HAND_ARROW)?;
                pose.posed_point(joints.get(pose.joints_root).ok()?, bone, offset)
            })
            .unwrap_or_else(|| (top + bottom) / 2.0);
        gizmos.line(top, middle, STRING_COLOR);
        gizmos.line(middle, bottom, STRING_COLOR);
    }
}

/// Registers the string drawer after the palette pass, beside the other joint-frame readers.
pub(crate) struct BowstringPlugin;

impl Plugin for BowstringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            draw_bowstrings.in_set(benilla_world::billboard::BillboardPlace),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_assets::{ModelJoint, ModelSkeleton};

    fn joint(parent: i16, local: Vec3) -> ModelJoint {
        ModelJoint {
            parent,
            local_translation: local,
            billboard: None,
            parent_arm: None,
        }
    }

    /// A limb-tip bone's frame sits at its pivot, so a posed tip is the anchor less that pivot:
    /// taking the model-local anchor whole lands the string a full limb past the tip.
    #[test]
    fn a_tip_pivot_sums_the_rest_chain() {
        let skeleton = ModelSkeleton {
            joints: vec![
                joint(-1, Vec3::new(0.0, 0.1, 0.0)),
                joint(0, Vec3::new(0.0, 0.0, -0.5)),
                joint(1, Vec3::new(0.0, 0.05, -0.4)),
            ],
            ..default()
        };
        assert_eq!(rest_pivot(&skeleton, 2), Vec3::new(0.0, 0.15, -0.9));
        assert_eq!(rest_pivot(&skeleton, 0), Vec3::new(0.0, 0.1, 0.0));
        assert_eq!(
            rest_pivot(&skeleton, 9),
            Vec3::ZERO,
            "an unknown bone has no pivot"
        );
    }
}
