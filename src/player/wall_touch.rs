//! Wall run hand IK: a two-bone solve over the animated pose plants the wall-side hand on the wall.
use super::*;
use bevy::app::AnimationSystems;
use bevy::ecs::system::ParamSet;
use bevy::transform::{TransformSystems, helper::TransformHelper};

mod knobs {
    pub const MAX_REACH: f32 = 1.5;
    /// Keeps the palm just off the surface
    pub const WALL_OFFSET: f32 = 0.05;
    pub const BLEND_RATE: f32 = 12.0;
}

const LEFT_ARM: [&str; 3] = ["DEF-upper_armL", "DEF-forearmL", "DEF-handL"];
const RIGHT_ARM: [&str; 3] = ["DEF-upper_armR", "DEF-forearmR", "DEF-handR"];

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        touch_wall
            .after(AnimationSystems)
            .before(TransformSystems::Propagate)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Upper arm, forearm and hand bones per side, plus the IK blend state.
#[derive(Component)]
pub struct ArmBones {
    arms: [[Entity; 3]; 2],
    weight: f32,
    /// Arm index and world-space hand target, kept while blending out
    target: Option<(usize, Vec3)>,
}

impl ArmBones {
    pub fn find(root: Entity, children_q: Query<&Children>, name_q: Query<&Name>) -> Option<Self> {
        let mut arms = [[Entity::PLACEHOLDER; 3]; 2];
        for (arm, names) in arms.iter_mut().zip([LEFT_ARM, RIGHT_ARM]) {
            for (bone, name) in arm.iter_mut().zip(names) {
                *bone = root.get_by_name(children_q, name_q, name)?;
            }
        }
        Some(Self {
            arms,
            weight: 0.0,
            target: None,
        })
    }
}

fn touch_wall(
    time: Res<Time>,
    spatial: SpatialQuery,
    mut players: Query<(Entity, &WallRunState, &mut ArmBones)>,
    mut transforms: ParamSet<(TransformHelper, Query<&mut Transform>)>,
) {
    for (player, wall_run, mut bones) in &mut players {
        let reach = wall_run.progress.as_ref().and_then(|progress| {
            wall_target(player, *progress.normal, &bones, &spatial, &transforms.p0())
        });
        let goal = if reach.is_some() { 1.0 } else { 0.0 };
        if reach.is_some() {
            bones.target = reach;
        }
        bones.weight +=
            (goal - bones.weight) * (1.0 - (-knobs::BLEND_RATE * time.delta_secs()).exp());

        let Some((side, target)) = bones.target else {
            continue;
        };
        if bones.weight < 0.001 {
            bones.target = None;
            continue;
        }

        let [upper, fore, hand] = bones.arms[side];
        let helper = transforms.p0();
        let (Ok(a), Ok(b), Ok(c)) = (
            helper.compute_global_transform(upper),
            helper.compute_global_transform(fore),
            helper.compute_global_transform(hand),
        ) else {
            continue;
        };
        let (upper_delta, fore_delta) = solve_two_bone(&a, &b, c.translation(), target);

        let mut locals = transforms.p1();
        for (bone, delta) in [(upper, upper_delta), (fore, fore_delta)] {
            if let Ok(mut local) = locals.get_mut(bone) {
                local.rotation *= Quat::IDENTITY.slerp(delta, bones.weight);
            }
        }
    }
}

/// Raycasts from the shoulder nearest the wall; `None` when the wall is out of reach.
fn wall_target(
    player: Entity,
    normal: Vec3,
    bones: &ArmBones,
    spatial: &SpatialQuery,
    helper: &TransformHelper,
) -> Option<(usize, Vec3)> {
    let shoulders = bones.arms.map(|[upper, ..]| {
        helper
            .compute_global_transform(upper)
            .ok()
            .map(|g| g.translation())
    });
    let (side, shoulder) = shoulders
        .into_iter()
        .enumerate()
        .filter_map(|(i, s)| Some((i, s?)))
        .max_by(|(_, x), (_, y)| x.dot(-normal).total_cmp(&y.dot(-normal)))?;

    let filter = SpatialQueryFilter::from_excluded_entities([player]);
    let hit = spatial.cast_ray(
        shoulder,
        Dir3::new(-normal).ok()?,
        knobs::MAX_REACH,
        true,
        &filter,
    )?;
    Some((
        side,
        shoulder - normal * hit.distance + normal * knobs::WALL_OFFSET,
    ))
}

/// Analytic two-bone IK; returns local rotation deltas for the upper and lower bone.
fn solve_two_bone(a: &GlobalTransform, b: &GlobalTransform, c: Vec3, t: Vec3) -> (Quat, Quat) {
    let (a_pos, a_rot) = (a.translation(), a.compute_transform().rotation);
    let (b_pos, b_rot) = (b.translation(), b.compute_transform().rotation);
    let angle = |u: Vec3, v: Vec3| {
        u.normalize_or_zero()
            .dot(v.normalize_or_zero())
            .clamp(-1.0, 1.0)
            .acos()
    };

    let lab = a_pos.distance(b_pos);
    let lcb = b_pos.distance(c);
    let lat = a_pos.distance(t).clamp(0.01, lab + lcb - 0.01);

    let ac_ab_0 = angle(c - a_pos, b_pos - a_pos);
    let ba_bc_0 = angle(a_pos - b_pos, c - b_pos);
    let ac_at_0 = angle(c - a_pos, t - a_pos);
    let ac_ab_1 = ((lcb * lcb - lab * lab - lat * lat) / (-2.0 * lab * lat))
        .clamp(-1.0, 1.0)
        .acos();
    let ba_bc_1 = ((lat * lat - lab * lab - lcb * lcb) / (-2.0 * lab * lcb))
        .clamp(-1.0, 1.0)
        .acos();

    // a straight arm has no bend plane, fall back to bending around a horizontal axis
    let axis0 = (c - a_pos)
        .cross(b_pos - a_pos)
        .try_normalize()
        .unwrap_or_else(|| (c - a_pos).cross(Vec3::Y).normalize_or(Vec3::X));
    let r0 = Quat::from_axis_angle(axis0, ac_ab_1 - ac_ab_0);
    let r1 = Quat::from_axis_angle(axis0, ba_bc_1 - ba_bc_0);
    let r2 = (c - a_pos)
        .cross(t - a_pos)
        .try_normalize()
        .map_or(Quat::IDENTITY, |axis1| {
            Quat::from_axis_angle(axis1, ac_at_0)
        });

    (
        a_rot.inverse() * r2 * r0 * a_rot,
        b_rot.inverse() * r1 * b_rot,
    )
}
