#![feature(associated_type_defaults)]

use std::{collections::{HashMap, hash_map}, sync::{Arc, RwLock}, f32::consts::PI, ops::Neg};
use macroquad::{prelude::{*}, rand::gen_range};

use rapier3d::{prelude::{CCDSolver, MultibodyJointSet, ImpulseJointSet, ColliderSet, RigidBodySet, BroadPhase, IslandManager, PhysicsPipeline, IntegrationParameters, Vector, Real, NarrowPhase, vector, Aabb, Shape, MassProperties, ShapeType, TypedShape, RayIntersection, Ray, PointProjection, FeatureId, Point, Cuboid, Isometry, TOI, SimdCompositeShape, RigidBodyBuilder, ColliderBuilder, RigidBodyHandle, SharedShape, Translation, RigidBody}, parry::{bounding_volume::{BoundingSphere, BoundingVolume}, query::{PointQuery, RayCast, DefaultQueryDispatcher, QueryDispatcher, ClosestPoints, Unsupported, Contact, NonlinearRigidMotion, ContactManifoldsWorkspace, PersistentQueryDispatcher, TypedWorkspaceData, WorkspaceData, visitors::BoundingVolumeIntersectionsVisitor, ContactManifold}, utils::IsometryOpt}};
use nalgebra::{self, Point3, Quaternion, UnitQuaternion};
use utility::{GameCamera, create_camera_from_game_camera, DebugText, TextPosition, BenchmarkWithDebugText, voxel_traversal_3d, AdjustHue, draw_cube_ex, WithAlpha, draw_sphere_wires_ex, draw_cube_wires_ex};

const WORLD_UP: Vec3 = Vec3::Y;

const VOXEL_SIZE: f32 = 1.0;
const VOXEL_DIMENSIONS: Vec3 = vec3(VOXEL_SIZE, VOXEL_SIZE, VOXEL_SIZE);

#[derive(Debug, Clone, Copy, PartialEq)]
enum VoxelKind {
    Air,
    Rock,
    Grass
}

#[derive(Debug, Clone, Copy)]
struct Voxel {
    kind: VoxelKind
}

#[derive(Debug)]
struct VoxelWorldSimple {

    bounds: (IVec3, IVec3),
    blocks: HashMap<IVec3, Voxel>

}

impl VoxelWorldSimple {

    pub fn new() -> VoxelWorldSimple {
        VoxelWorldSimple {
            bounds: (IVec3::ZERO, IVec3::ZERO),
            blocks: HashMap::new()
        }
    }

    pub fn generate_world(&mut self, width: i32, height: i32, depth: i32) {

        self.bounds = (IVec3::ZERO, IVec3::ZERO);
        self.blocks.clear();

        // generate a block of.. blocks :D

        for x in 0..width {
            for y in 0..height {
                for z in 0..depth {

                    let rand_value = gen_range(0, 100);
                    let should_create_voxel = rand_value > 50;

                    if should_create_voxel {
                        let current_voxel_kind = if y > height / 2 { VoxelKind::Grass } else { VoxelKind::Rock };
                        self.set_block(ivec3(x, y, z), current_voxel_kind);
                    }

                }
            }
        }

        self.update_world_bounds();

    }

    pub fn get_block(&self, position: IVec3) -> VoxelKind {
        self.blocks.get(&position).unwrap_or(&Voxel { kind: VoxelKind::Air }).kind
    }

    pub fn set_block(&mut self, position: IVec3, kind: VoxelKind) {
        
        self.blocks.insert(position, Voxel { kind: kind });

        // # HACK: This is terrifically inefficient innit :D
        self.update_world_bounds();

    }

    /// Extremely naive function that just updates the current calculated world bounds, this is fine with a small world size!
    pub fn update_world_bounds(&mut self) {

        let mut min_x = 0;
        let mut min_y = 0;
        let mut min_z = 0;

        let mut max_x = 0;
        let mut max_y = 0;
        let mut max_z = 0;

        for (pos, _kind) in &self.blocks {

            min_x = pos.x.min(min_x);
            min_y = pos.y.min(min_y);
            min_z = pos.z.min(min_z);

            max_x = pos.x.max(max_x);
            max_y = pos.y.max(max_y);
            max_z = pos.z.max(max_z);

        }

        let min_bounds = ivec3(min_x, min_y, min_z);
        let max_bounds = ivec3(max_x + 1, max_y + 1, max_z + 1);
        self.bounds = (min_bounds, max_bounds);

    }

}

impl VoxelWorld for VoxelWorldSimple {

    fn blocks<'a>(&'a self) -> Box<dyn Iterator<Item=(&IVec3, &Voxel)> + 'a> {
        Box::new(self.blocks.iter())
    }

    fn set_block(&mut self, position: IVec3, kind: VoxelKind) {
        self.set_block(position, kind)
    }

    fn get_block(&self, position: IVec3) -> VoxelKind {
        self.get_block(position)
    }

    fn get_bounds(&self) -> Aabb {
        let (min, max) = self.bounds;
        Aabb {
            mins: Point3::new(min.x as Real, min.y as Real, min.z as Real),
            maxs: Point3::new(max.x as Real, max.y as Real, max.z as Real)
        }
    }

    fn get_block_bounds(&self) -> (IVec3, IVec3) {
        self.bounds
    }

    fn get_world_bounds(&self) -> (Vec3, Vec3) {
        let (min, max) = self.bounds;
        let min_world = vec3(min.x as f32, min.y as f32, min.z as f32) * VOXEL_SIZE;
        let max_world = vec3(max.x as f32, max.y as f32, max.z as f32) * VOXEL_SIZE;
        (min_world, max_world)
    }

    fn try_pick_block_in_world(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<(Vec3, VoxelKind)> {

        // #FIXME: this hardcoding is kinda funky but lol
        let ray_target = ray_origin + ray_direction * 1000.0;
        let mut picked_position_and_voxel: Option<(Vec3, VoxelKind)> = None;
    
        let _hit_any_voxel = voxel_traversal_3d(
            ray_origin,
            ray_target,
            |pos| {
                let block_position = ivec3(pos.x.floor() as i32, pos.y.floor() as i32, pos.z.floor() as i32);
                let block_at_position = self.get_block(block_position);
    
                if block_at_position != VoxelKind::Air {
                    let view_position = vec3(block_position.x as f32, block_position.y as f32, block_position.z as f32);
                    picked_position_and_voxel = Some((view_position, block_at_position));
                    return true;
                }
    
                return false;
            }
        );
    
        picked_position_and_voxel
    }

}

trait VoxelWorld : Send + Sync + 'static {

    // type BlockIterator = dyn Iterator<Item=(IVec3, VoxelKind)>;

    fn try_pick_block_in_world(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<(Vec3, VoxelKind)>;
    fn blocks<'a>(&'a self) -> Box<dyn Iterator<Item=(&IVec3, &Voxel)> + 'a>;

    fn set_block(&mut self, position: IVec3, kind: VoxelKind);
    fn get_block(&self, position: IVec3) -> VoxelKind;
    fn get_world_bounds(&self) -> (Vec3, Vec3);
    fn get_block_bounds(&self) -> (IVec3, IVec3);
    fn get_bounds(&self) -> Aabb;

}

#[derive(Clone)]
pub struct VoxelWorldShape {
    world: Arc<RwLock<dyn VoxelWorld>>
}

impl VoxelWorldShape {

    #[inline]
    pub fn cast_ray_to_toi(&self, ray: &Ray, max_toi: Real) -> Option<f32>
    {

        let origin = vec3(ray.origin.x, ray.origin.y, ray.origin.z);
        let direction = vec3(ray.dir.x, ray.dir.y, ray.dir.z);

        let voxel_world_reader = self.world.read().unwrap();
        let (pos, _kind) = voxel_world_reader.try_pick_block_in_world(origin, direction)?;
        let toi = (pos - origin).length();

        if toi > max_toi {
            return None
        };

        Some(toi)

    }

    pub fn map_elements_in_local_sphere(
        &self,
        bounds: &BoundingSphere,
        mut f: impl FnMut(&IVec3, u32, &Cuboid) -> bool,
    ) {

        let center = vec3(bounds.center.x, bounds.center.y, bounds.center.z);
        let start = center - bounds.radius;
        let end = center + bounds.radius;

        let start_x = start.x.max(0.0).floor() as i32;
        let start_y = start.y.max(0.0).floor() as i32;
        let start_z = start.z.max(0.0).floor() as i32;

        let end_x = end.x.max(0.0).ceil() as i32;
        let end_y = end.y.max(0.0).ceil() as i32;
        let end_z = end.z.max(0.0).ceil() as i32;

        let voxel_world_reader = self.world.read().unwrap();

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                for z in start_z..=end_z {

                    let world_block_pos = ivec3(x, y, z);
                    let is_within_bounds = voxel_world_reader.get_bounds().contains_local_point(&Point::new(x as f32, y as f32, z as f32));
                    if !is_within_bounds {
                        continue;
                    }

                    if voxel_world_reader.get_block(world_block_pos) != VoxelKind::Air {

                        // #FIXME: fill me in!
                        let feature_id = 0u32;
                        
                        let result = f(&world_block_pos, feature_id, &self.cube());
                        if !result {
                            return;
                        }

                    }

                }
            }
        }

    }

    pub fn coords_to_index(&self, coords: IVec3) -> i32 {
        let (_min, max) = self.world.read().unwrap().get_block_bounds();
        (coords.z * max.x * max.y) + (coords.y * max.x) + coords.x
    }

    pub fn index_to_coords(&self, idx: i32) -> IVec3 {
        let (_min, max) = self.world.read().unwrap().get_block_bounds();
        let z = idx / (max.x * max.y);
        let y = (idx - (z * max.x * max.y)) / max.y;
        let x = (idx - (z * max.x * max.y)) % max.x;
        ivec3(x, y, z)
    }

    fn feature_id(&self, coords: IVec3, feature_id: u32)  -> u32 {
        (self.coords_to_index(coords) * feature_id as i32) as u32
    }

    pub fn cube(&self) -> Cuboid {
        Cuboid::new(
            Vector::new(
                VOXEL_SIZE * 0.5,
                VOXEL_SIZE * 0.5,
                VOXEL_SIZE * 0.5
            )
        )
    }

    pub fn infer_face_normal_from_ray(ray: Ray) -> Vector<Real> {

        let up_vector = Vector::new(0.0, 1.0, 0.0);
        let down_vector = -up_vector;

        let left_vector = Vector::new(1.0, 0.0, 0.0);
        let right_vector = -left_vector;

        let forward_vector = Vector::new(0.0, 0.0, 1.0);
        let backward_vector = -forward_vector;

        let mut smallest_vector = up_vector;
        let mut smallest_value = ray.dir.dot(&up_vector);

        for v in &[up_vector, down_vector, left_vector, right_vector, forward_vector, backward_vector] {
            let d = ray.dir.dot(v);
            if d < smallest_value {
                smallest_vector = *v;
                smallest_value = d;
            }
        }

        smallest_vector

    }

}

impl Shape for VoxelWorldShape {

    fn compute_local_aabb(&self) -> Aabb {
        self.world.read().unwrap().get_bounds()
    }

    fn compute_local_bounding_sphere(&self) -> BoundingSphere {
        self.world.read().unwrap().get_bounds().bounding_sphere()
    }

    fn clone_box(&self) -> Box<dyn Shape> {
        todo!()
    }

    fn mass_properties(&self, density: Real) -> MassProperties {
        Cuboid {
            half_extents: self.world.read().unwrap().get_bounds().extents() / 2.0
        }.mass_properties(density)
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Custom
    }

    fn as_typed_shape(&self) -> TypedShape {
        TypedShape::Custom(0)
    }

    fn ccd_thickness(&self) -> Real {
        0.0
    }

    fn ccd_angular_thickness(&self) -> Real {
        0.0
    }

}

impl PointQuery for VoxelWorldShape {
    
    fn project_local_point(&self, pt: &Point<Real>, solid: bool) -> PointProjection {

        let contains_point = self.world.read().unwrap().get_bounds().contains_local_point(pt);

        if solid && contains_point {
            return PointProjection {
                is_inside: true,
                point: *pt
            }
        }

        self.project_local_point_and_get_feature(pt).0

    }

    fn project_local_point_and_get_feature(&self, pt: &Point<Real>)
        -> (PointProjection, FeatureId) {

        let position = ivec3(pt.x as i32, pt.y as i32, pt.z as i32);
        let is_current_block_not_air = self.world.read().unwrap().get_block(position) != VoxelKind::Air;

        let point_projection = PointProjection {
            is_inside: is_current_block_not_air,
            point: *pt
        };

        // TODO: get the closest point on the block, not clamped to block coordinates like it is now
        let feature_id = FeatureId::Face(0);
        (point_projection, feature_id)

    }

}

impl RayCast for VoxelWorldShape {

    fn cast_local_ray_and_get_normal(
        &self,
        ray: &Ray,
        max_toi: Real,
        solid: bool,
    ) -> Option<RayIntersection> {

        let toi = self.cast_ray_to_toi(ray, max_toi)?;

         // TODO: compute real normal from block position and ray angle, should be (mostly?) trivial
        let normal = VoxelWorldShape::infer_face_normal_from_ray(*ray);

        // TODO: this dovetails with the above, once we can compute the face normals we can also figure out what kind of feature this is, edge, face or vertex(and if a face, what face, etc)
        let feature = FeatureId::Face(0);

        let intersection = RayIntersection::new(
            toi,
            normal,
            feature
        );

        Some(intersection)

    }

}

fn voxel_translation(coords: IVec3) -> Translation<Real>
{

    let block_offset = Vector::new(
        coords.x as f32 * VOXEL_SIZE,
        coords.y as f32 * VOXEL_SIZE,
        coords.z as f32 * VOXEL_SIZE
    );

    let final_offset = block_offset;
    final_offset.into()

}

fn intersects(pos12: &Isometry<Real>, world: &VoxelWorldShape, other: &dyn Shape) -> bool {
    // TODO after https://github.com/dimforge/parry/issues/8
    let dispatcher = DefaultQueryDispatcher;
    let bounds = other.compute_bounding_sphere(pos12);
    let mut intersects = false;
    world.map_elements_in_local_sphere(&bounds, |coords, _, cuboid| {
        
        let relative_pos12 = voxel_translation(*coords).inverse() * pos12;

        intersects = dispatcher
            .intersection_test(&relative_pos12, cuboid, other)
            .unwrap_or(false);

        !intersects

    });
    intersects
}

fn compute_toi(
    pos12: &Isometry<Real>,
    vel12: &Vector<Real>,
    world: &VoxelWorldShape,
    other: &dyn Shape,
    max_toi: Real,
    stop_at_time_of_impact: bool,
    flipped: bool,
) -> Option<TOI> {

    // TODO after https://github.com/dimforge/parry/issues/8
    let dispatcher = DefaultQueryDispatcher;

    // TODO: Raycast vs. minkowski sum of bounds (later chunk bounds?) and bounding sphere?
    let bounds = {
        let start = other.compute_aabb(pos12);
        let end = start.transform_by(&Isometry::from_parts((max_toi * vel12).into(), nalgebra::one()));
        start.merged(&end).bounding_sphere()
    };

    let mut closest = None::<TOI>;
    world.map_elements_in_local_sphere(&bounds, |_, _, cuboid| {
        let impact = if flipped {
            dispatcher.time_of_impact(
                &pos12.inverse(),
                &-vel12,
                other,
                cuboid,
                max_toi,
                stop_at_time_of_impact
            )
        } else {
            dispatcher.time_of_impact(
                &pos12,
                vel12,
                cuboid,
                other,
                max_toi,
                stop_at_time_of_impact
            )
        };
        if let Ok(Some(impact)) = impact {
            closest = Some(match closest {
                None => impact,
                Some(x) if impact.toi < x.toi => impact,
                Some(x) => x,
            });
        }

        true

    });

    closest

}

#[allow(clippy::too_many_arguments)] // that's just what it takes
fn compute_nonlinear_toi(
    motion_world: &NonlinearRigidMotion,
    world: &VoxelWorldShape,
    motion_other: &NonlinearRigidMotion,
    other: &dyn Shape,
    start_time: Real,
    end_time: Real,
    stop_at_penetration: bool,
    flipped: bool,
) -> Option<TOI> {

    // TODO after https://github.com/dimforge/parry/issues/8
    let dispatcher = DefaultQueryDispatcher;

    // TODO: Select blocks more conservatively, as discussed in compute_toi
    let bounds = {

        let start_pos = motion_world.position_at_time(start_time).inverse()
            * motion_other.position_at_time(start_time);
        let end_pos = motion_world.position_at_time(end_time).inverse()
            * motion_other.position_at_time(end_time);

        let start = other.compute_aabb(&start_pos);
        let end = other.compute_aabb(&end_pos);

        start.merged(&end).bounding_sphere()

    };

    let mut closest = None::<TOI>;
    world.map_elements_in_local_sphere(&bounds, |coords, _, cuboid| {
        let impact = if flipped {
            dispatcher.nonlinear_time_of_impact(
                motion_other,
                other,
                motion_world,
                cuboid,
                start_time,
                end_time,
                stop_at_penetration,
            )
        } else {
            dispatcher.nonlinear_time_of_impact(
                motion_world,
                cuboid,
                motion_other,
                other,
                start_time,
                end_time,
                stop_at_penetration,
            )
        };
        if let Ok(Some(impact)) = impact {
            closest = Some(match closest {
                None => impact,
                Some(x) if impact.toi < x.toi => impact,
                Some(x) => x,
            });
        }
        true
    });

    closest

}


impl<ManifoldData, ContactData> PersistentQueryDispatcher<ManifoldData, ContactData>
    for VoxelWorldShapeDispatcher
where
    ManifoldData: Default + Clone,
    ContactData: Default + Copy,
{
    fn contact_manifolds(
        &self,
        pos12: &Isometry<Real>,
        g1: &dyn Shape,
        g2: &dyn Shape,
        prediction: Real,
        manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>,
        workspace: &mut Option<ContactManifoldsWorkspace>,
    ) -> Result<(), Unsupported> {
        if let Some(p1) = g1.downcast_ref::<VoxelWorldShape>() {
            if let Some(composite) = g2.as_composite_shape() {
                compute_manifolds_vs_composite(
                    pos12,
                    &pos12.inverse(),
                    p1,
                    composite,
                    prediction,
                    manifolds,
                    workspace,
                    false,
                );
            } else {
                compute_manifolds(pos12, p1, g2, prediction, manifolds, workspace, false);
            }
            return Ok(());
        }
        if let Some(p2) = g2.downcast_ref::<VoxelWorldShape>() {
            if let Some(composite) = g2.as_composite_shape() {
                compute_manifolds_vs_composite(
                    &pos12.inverse(),
                    pos12,
                    p2,
                    composite,
                    prediction,
                    manifolds,
                    workspace,
                    true,
                );
            } else {
                compute_manifolds(
                    &pos12.inverse(),
                    p2,
                    g1,
                    prediction,
                    manifolds,
                    workspace,
                    true,
                );
            }
            return Ok(());
        }
        Err(Unsupported)
    }

    fn contact_manifold_convex_convex(
        &self,
        _pos12: &Isometry<Real>,
        _g1: &dyn Shape,
        _g2: &dyn Shape,
        _prediction: Real,
        _manifold: &mut ContactManifold<ManifoldData, ContactData>,
    ) -> Result<(), Unsupported> {
        // Voxel worlds aren't guaranteed to be convex, so we have no cases to handle here
        Err(Unsupported)
    }
}

fn compute_manifolds<ManifoldData, ContactData>(
    pos12: &Isometry<Real>,
    world: &VoxelWorldShape,
    other: &dyn Shape,
    prediction: Real,
    manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>,
    workspace: &mut Option<ContactManifoldsWorkspace>,
    flipped: bool,
) where
    ManifoldData: Default + Clone,
    ContactData: Default + Copy,
{
    let workspace = workspace
        .get_or_insert_with(|| ContactManifoldsWorkspace(Box::new(Workspace::default())))
        .0
        .downcast_mut::<Workspace>()
        .unwrap();
    let dispatcher = DefaultQueryDispatcher; // TODO after https://github.com/dimforge/parry/issues/8

    workspace.phase ^= true;
    let phase = workspace.phase;

    let bounds = other.compute_bounding_sphere(pos12).loosened(prediction);
    let mut old_manifolds = std::mem::take(manifolds);
    world.map_elements_in_local_sphere(&bounds, |&coords, index, cuboid| {
        let voxel_state = match workspace.state.entry((coords, index)) {
            hash_map::Entry::Occupied(e) => {
                let voxel_state = e.into_mut();

                let manifold = old_manifolds[voxel_state.manifold_index].take();
                voxel_state.manifold_index = manifolds.len();
                voxel_state.phase = phase;
                manifolds.push(manifold);

                voxel_state
            }
            hash_map::Entry::Vacant(e) => {
                let voxel_state = VoxelState {
                    manifold_index: manifolds.len(),
                    phase,
                };

                let id = world.feature_id(coords, index) as u32;
                let (id1, id2) = if flipped { (0, id) } else { (id, 0) };
                manifolds.push(ContactManifold::with_data(
                    id1,
                    id2,
                    ManifoldData::default(),
                ));

                e.insert(voxel_state)
            }
        };

        let manifold = &mut manifolds[voxel_state.manifold_index];
    
        // translate current position to one accurate for local space given our current voxel being tested against

        let mut pos12 = *pos12;
        pos12.append_translation_mut(&voxel_translation(coords).inverse());

        // TODO: Nonconvex, postprocess contact `fid`s once parry's feature ID story is worked out
        if flipped {

            let _ = dispatcher.contact_manifold_convex_convex(
                &pos12.inverse(),
                other,
                cuboid,
                prediction,
                manifold,
            );

            // translate contacts back to positions accurate in world space to the shape

            for p in &mut manifold.points {

                let local_p1_adjusted = voxel_translation(coords).inverse() * p.local_p1;
                let local_p2_adjusted = voxel_translation(coords) * p.local_p2;

                p.local_p1 = local_p1_adjusted;
                p.local_p2 = local_p2_adjusted;

            }

        } else {

            let _ = dispatcher
                .contact_manifold_convex_convex(
                    &pos12,
                    cuboid,
                    other,
                    prediction,
                    manifold
                );

            // translate contacts back to positions accurate in world space to the shape

            for p in &mut manifold.points {

                let local_p1_adjusted = voxel_translation(coords) * p.local_p1;
                let local_p2_adjusted = voxel_translation(coords).inverse() * p.local_p2;

                p.local_p1 = local_p1_adjusted;
                p.local_p2 = local_p2_adjusted;

            }

        }
        true
    });

    workspace.state.retain(|_, x| x.phase == phase);
}

/// Narrow-phase collision detection state for `VoxelWorldShape`
#[derive(Default, Clone)]
pub struct Workspace {
    state: HashMap<(IVec3, u32), VoxelState>,
    phase: bool,
}

impl WorkspaceData for Workspace {
    fn as_typed_workspace_data(&self) -> TypedWorkspaceData {
        TypedWorkspaceData::Custom(0)
    }

    fn clone_dyn(&self) -> Box<dyn WorkspaceData> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
struct VoxelState {
    manifold_index: usize,
    phase: bool,
}

#[allow(clippy::too_many_arguments)] // that's just what it takes
fn compute_manifolds_vs_composite<ManifoldData, ContactData>(
    pos12: &Isometry<Real>,
    pos21: &Isometry<Real>,
    world: &VoxelWorldShape,
    other: &dyn SimdCompositeShape,
    prediction: Real,
    manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>,
    workspace: &mut Option<ContactManifoldsWorkspace>,
    flipped: bool,
) where
    ManifoldData: Default + Clone,
    ContactData: Default + Copy,
{
    let workspace = workspace
        .get_or_insert_with(|| ContactManifoldsWorkspace(Box::new(WorkspaceVsComposite::default())))
        .0
        .downcast_mut::<WorkspaceVsComposite>()
        .unwrap();
    let dispatcher = DefaultQueryDispatcher; // TODO after https://github.com/dimforge/parry/issues/8

    workspace.phase ^= true;
    let phase = workspace.phase;

    let bvh = other.qbvh();

    let bounds = bvh
        .root_aabb()
        .bounding_sphere()
        .transform_by(pos12)
        .loosened(prediction);

    let mut old_manifolds = std::mem::take(manifolds);

    world.map_elements_in_local_sphere(&bounds, |&coords, index, cuboid| {

        // compute actual offset of cuboid we're testing with
        // let pos12 = &cuboid_with_translation(pos12, coords);
        // let mut pos21 = &mut cuboid_with_translation(pos21, coords);
        // pos21.translation = (-pos21.translation.vector).into();
        
        let voxel_aabb = cuboid.compute_aabb(pos21).loosened(prediction);

        let mut visit = |&composite_subshape: &u32| {
            other.map_part_at(
                composite_subshape,
                &mut |composite_part_pos, composite_part_shape| {
                    let key = CompositeKey {
                        block_coords: coords,
                        composite_subshape,
                    };
                    // TODO: Dedup wrt. convex case
                    let voxel_state = match workspace.state.entry(key) {
                        hash_map::Entry::Occupied(e) => {
                            let voxel_state = e.into_mut();

                            let manifold = old_manifolds[voxel_state.manifold_index].take();
                            voxel_state.manifold_index = manifolds.len();
                            voxel_state.phase = phase;
                            manifolds.push(manifold);

                            voxel_state
                        }
                        hash_map::Entry::Vacant(e) => {
                            let mut manifold = ContactManifold::new();
                            let id = world.feature_id(coords, index) as u32;
                            if flipped {
                                manifold.subshape1 = composite_subshape;
                                manifold.subshape2 = id;
                                manifold.subshape_pos1 = composite_part_pos.copied();
                            } else {
                                manifold.subshape1 = id;
                                manifold.subshape2 = composite_subshape;
                                manifold.subshape_pos2 = composite_part_pos.copied();
                            };

                            let voxel_state = VoxelState {
                                manifold_index: manifolds.len(),
                                phase,
                            };
                            manifolds.push(manifold);
                            e.insert(voxel_state)
                        }
                    };

                    let manifold = &mut manifolds[voxel_state.manifold_index];
                    // let pos12 = &with_translation_for_voxel(pos12, coords);
                    // let pos21 = &with_translation_for_voxel(pos21, coords);

                    if flipped {
                        let _ = dispatcher.contact_manifold_convex_convex(
                            &composite_part_pos.inv_mul(pos21),
                            composite_part_shape,
                            cuboid,
                            prediction,
                            manifold,
                        );
                    } else {
                        let _ = dispatcher.contact_manifold_convex_convex(
                            &composite_part_pos.prepend_to(pos12),
                            cuboid,
                            composite_part_shape,
                            prediction,
                            manifold,
                        );
                    }
                },
            );
            true
        };
        let mut visitor = BoundingVolumeIntersectionsVisitor::new(&voxel_aabb, &mut visit);
        bvh.traverse_depth_first(&mut visitor);

        true
    });

    workspace.state.retain(|_, x| x.phase == phase);
}

/// Narrow-phase collision detection state for `Planet`
#[derive(Default, Clone)]
pub struct WorkspaceVsComposite {
    state: HashMap<CompositeKey, VoxelState>,
    phase: bool,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct CompositeKey {
    block_coords: IVec3,
    composite_subshape: u32,
}

impl WorkspaceData for WorkspaceVsComposite {
    fn as_typed_workspace_data(&self) -> TypedWorkspaceData {
        TypedWorkspaceData::Custom(0)
    }

    fn clone_dyn(&self) -> Box<dyn WorkspaceData> {
        Box::new(self.clone())
    }
}

pub struct PhysicsState {

    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,

    ccd_solver: CCDSolver,

}

impl PhysicsState {

    pub fn new() -> PhysicsState {

        // collider sets
        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();

        // simulation parameters, etc
        let gravity = vector![0.0, -9.81, 0.0];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::with_query_dispatcher(
            VoxelWorldShapeDispatcher.chain(DefaultQueryDispatcher)
        );
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();

        PhysicsState {

            rigid_body_set,
            collider_set,

            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver

        }

    }
    
}

pub struct PhysicsWorld {
    state: PhysicsState,
}

impl PhysicsWorld {

    pub fn add_voxel_world(&mut self, world: VoxelWorldShape) -> RigidBodyHandle {

        let rigid_body = RigidBodyBuilder::fixed()
            .build();

        let world_body_handle = self.state.rigid_body_set.insert(rigid_body);
        let world_collider = ColliderBuilder::new(SharedShape::new(world.clone()))
            .sensor(false)
            .build();

        self.state.collider_set.insert_with_parent(world_collider, world_body_handle, &mut self.state.rigid_body_set);

        world_body_handle

    }

    pub fn add_box(&mut self, position: Vec3) -> RigidBodyHandle {

        let box_half_size = 0.5 - 0.1;
        let box_restitution = 0.7;

        let rigid_body = RigidBodyBuilder::dynamic()
            .ccd_enabled(true)
            .can_sleep(false)
            .translation(Vector::new(position.x, position.y, position.z))
            .rotation(Vector::new(0.0, PI, 0.0))
            .build();

        let collider = ColliderBuilder::cuboid(box_half_size, box_half_size, box_half_size).restitution(box_restitution).build();
        let ball_body_handle = self.state.rigid_body_set.insert(rigid_body);

        self.state.collider_set.insert_with_parent(collider, ball_body_handle, &mut self.state.rigid_body_set);

        ball_body_handle

    }

    pub fn add_ball(&mut self, position: Vec3) -> RigidBodyHandle {

        let ball_radius = 0.5 - 0.1;
        let ball_restitution = 0.7;

        let rigid_body = RigidBodyBuilder::dynamic()
            .ccd_enabled(true)
            .can_sleep(false)
            .translation(Vector::new(position.x, position.y, position.z))
            .rotation(Vector::new(0.0, PI, 0.0))
            .build();

        let collider = ColliderBuilder::ball(ball_radius).restitution(ball_restitution).build();
        let ball_body_handle = self.state.rigid_body_set.insert(rigid_body);

        self.state.collider_set.insert_with_parent(collider, ball_body_handle, &mut self.state.rigid_body_set);

        ball_body_handle

    }

    /// Returns a vec of all the current contact points, contact points are in world space.
    pub fn contact_points(&self) -> Vec<Vector<Real>> {
        self.state.narrow_phase.contact_pairs()
            .map(|c| &c.manifolds).flatten()
            .map(|m| &m.data.solver_contacts).flatten()
            .map(|s| s.point.coords)
            .collect()
    }

}

pub struct VoxelWorldShapeDispatcher;

impl QueryDispatcher for VoxelWorldShapeDispatcher {

    fn intersection_test(
        &self,
        pos12: &Isometry<Real>,
        g1: &dyn Shape,
        g2: &dyn Shape,
    ) -> Result<bool, Unsupported> {
        if let Some(p1) = g1.downcast_ref::<VoxelWorldShape>() {
            return Ok(intersects(pos12, p1, g2));
        }
        if let Some(p2) = g2.downcast_ref::<VoxelWorldShape>() {
            return Ok(intersects(&pos12.inverse(), p2, g1));
        }
        Err(Unsupported)
    }

    fn distance(
        &self,
        pos12: &Isometry<Real>,
        g1: &dyn Shape,
        g2: &dyn Shape,
    ) -> Result<Real, Unsupported> {
        todo!()
    }

    fn contact(
        &self,
        pos12: &Isometry<Real>,
        g1: &dyn Shape,
        g2: &dyn Shape,
        prediction: Real,
    ) -> Result<Option<Contact>, Unsupported> {
        todo!()
    }

    fn closest_points(
        &self,
        pos12: &Isometry<Real>,
        g1: &dyn Shape,
        g2: &dyn Shape,
        max_dist: Real,
    ) -> Result<ClosestPoints, Unsupported> {
        todo!()
    }

    fn time_of_impact(
        &self,
        pos12: &Isometry<Real>,
        vel12: &Vector<Real>, // #TODO: this used to be called local_vel12, is that relevant? who knows, hopefully not lol
        g1: &dyn Shape,
        g2: &dyn Shape,
        max_toi: Real,
        stop_at_penetration: bool,
    ) -> Result<Option<TOI>, Unsupported> {
        if let Some(p1) = g1.downcast_ref::<VoxelWorldShape>() {
            return Ok(compute_toi(
                pos12,
                vel12,
                p1,
                g2,
                max_toi,
                stop_at_penetration,
                false,
            ));
        }
        if let Some(p2) = g2.downcast_ref::<VoxelWorldShape>() {
            return Ok(compute_toi(
                &pos12.inverse(),
                &-vel12,
                p2,
                g1,
                max_toi,
                stop_at_penetration,
                true,
            ));
        }
        Err(Unsupported)
    }

    fn nonlinear_time_of_impact(
        &self,
        motion1: &NonlinearRigidMotion,
        g1: &dyn Shape,
        motion2: &NonlinearRigidMotion,
        g2: &dyn Shape,
        start_time: Real,
        end_time: Real,
        stop_at_penetration: bool,
    ) -> Result<Option<TOI>, Unsupported> {
        if let Some(p1) = g1.downcast_ref::<VoxelWorldShape>() {
            return Ok(compute_nonlinear_toi(
                motion1,
                p1,
                motion2,
                g2,
                start_time,
                end_time,
                stop_at_penetration,
                false,
            ));
        }
        if let Some(p2) = g2.downcast_ref::<VoxelWorldShape>() {
            return Ok(compute_nonlinear_toi(
                motion2,
                p2,
                motion1,
                g1,
                start_time,
                end_time,
                stop_at_penetration,
                false,
            ));
        }
        Err(Unsupported)
    }

}

impl PhysicsWorld {

    pub fn new() -> PhysicsWorld {
        PhysicsWorld {
            state: PhysicsState::new()
        }
    }

    pub fn step(&mut self) {

        let physics_hooks = ();
        let event_handler = ();

        self.state.physics_pipeline.step(
            &self.state.gravity,
            &self.state.integration_parameters,
            &mut self.state.island_manager,
            &mut self.state.broad_phase,
            &mut self.state.narrow_phase,
            &mut self.state.rigid_body_set,
            &mut self.state.collider_set,
            &mut self.state.impulse_joint_set,
            &mut self.state.multibody_joint_set,
            &mut self.state.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
          );

    }

}

pub struct Game {

    camera: GameCamera,
    debug_text: DebugText,
    voxel_world: Arc<RwLock<dyn VoxelWorld>>,
    physics_world: PhysicsWorld,

    debug_parameters: GameDebugParameters

}

pub struct GameDebugParameters {
    should_show_contacts: bool
}

impl GameDebugParameters {
    pub fn new() -> GameDebugParameters {
        GameDebugParameters {
            should_show_contacts: true
        }
    }
}

impl Game {
    pub fn new() -> Game {
        Game {

            camera: GameCamera::new(),
            debug_text: DebugText::new(),
            voxel_world: Arc::new(RwLock::new(VoxelWorldSimple::new())),
            physics_world: PhysicsWorld::new(),

            // purely debug specific stuff
            debug_parameters: GameDebugParameters::new()

        }
    }
}

fn handle_camera_input(active: &mut GameCamera, dt: f32) {

    let is_forwards_pressed = is_key_down(KeyCode::W);
    let is_backwards_pressed = is_key_down(KeyCode::S);
    let is_left_pressed = is_key_down(KeyCode::A);
    let is_right_pressed = is_key_down(KeyCode::D);

    let is_up_pressed = is_key_down(KeyCode::Space);
    let is_down_pressed = is_key_down(KeyCode::LeftControl);

    let mut camera_movement_delta = Vec3::ZERO;

    let forward_in_plane = vec3(active.forward().x, 0.0, active.forward().z);
    let left_in_plane = vec3(active.left().x, 0.0, active.left().z);
    let up_in_plane = WORLD_UP;

    if is_forwards_pressed {
        camera_movement_delta += forward_in_plane * active.parameters.movement_speed * dt;
    }

    if is_backwards_pressed {
        camera_movement_delta -= forward_in_plane * active.parameters.movement_speed * dt;
    }

    if is_left_pressed {
        camera_movement_delta += left_in_plane * active.parameters.movement_speed * dt;
    }

    if is_right_pressed {
        camera_movement_delta -= left_in_plane * active.parameters.movement_speed * dt;
    }

    if is_up_pressed {
        camera_movement_delta += up_in_plane * active.parameters.movement_speed * dt;
    }

    if is_down_pressed {
        camera_movement_delta -= up_in_plane * active.parameters.movement_speed * dt;
    }

    active.position += camera_movement_delta;
    active.target += camera_movement_delta;

}

fn is_voxel_occluded(voxel_world: &dyn VoxelWorld, position: IVec3) -> bool {

    let above = (voxel_world.get_block(position + ivec3(0, 1, 0)) != VoxelKind::Air) as i32;
    let below = (voxel_world.get_block(position + ivec3(0, -1, 0)) != VoxelKind::Air) as i32;
    let left = (voxel_world.get_block(position + ivec3(-1, 0, 0)) != VoxelKind::Air) as i32;
    let right = (voxel_world.get_block(position + ivec3(1, 0, 0)) != VoxelKind::Air) as i32;
    let front = (voxel_world.get_block(position + ivec3(0, 0, 1)) != VoxelKind::Air) as i32;
    let behind = (voxel_world.get_block(position + ivec3(0, 0, -1)) != VoxelKind::Air) as i32;

    let number_of_faces_occluded = above + below + left + right + front + behind;
    return number_of_faces_occluded == 6;

}

fn render_grass_block(render_pos: Vec3) {
    draw_cube(render_pos, VOXEL_DIMENSIONS, None, GREEN);
    draw_cube_wires(render_pos, VOXEL_DIMENSIONS, GREEN.darken(0.25));
}

fn render_stone_block(render_pos: Vec3) {
    draw_cube(render_pos, VOXEL_DIMENSIONS, None, GRAY);
    draw_cube_wires(render_pos, VOXEL_DIMENSIONS, GRAY.darken(0.25));
}

fn render_voxel_world(voxel_world: &dyn VoxelWorld) {

    for (&pos, &voxel) in voxel_world.blocks() {

        let render_position = vec3(pos.x as f32, pos.y as f32, pos.z as f32) * VOXEL_SIZE;
        if is_voxel_occluded(voxel_world, pos) { continue; }

        match voxel.kind {
            VoxelKind::Grass => render_grass_block(render_position),
            VoxelKind::Rock => render_stone_block(render_position),
            VoxelKind::Air => ()
        }

    }

}

fn render_physics_object(rigid_body: &RigidBody, shape: &dyn Shape) {

    let body_isometry = rigid_body.position();
    let body_rotation = rigid_body.rotation();
    let body_quat = body_rotation.coords;

    if let Some(ball) = shape.as_ball() {
        draw_sphere_wires_ex(
            vec3(body_isometry.translation.x, body_isometry.translation.y, body_isometry.translation.z),
            quat(body_quat.x, body_quat.y, body_quat.z, body_quat.w),
            ball.radius,
            BLACK.lighten(0.25),
            DrawSphereParams { rings: 8, slices: 8, ..Default::default() }
        );
    }

    if let Some(cuboid) = shape.as_cuboid() {
        let cuboid_size = vec3(cuboid.half_extents.x * 2.0, cuboid.half_extents.y * 2.0, cuboid.half_extents.z * 2.0);
        draw_cube_wires_ex(
            vec3(body_isometry.translation.x, body_isometry.translation.y, body_isometry.translation.z),
            quat(body_quat.x, body_quat.y, body_quat.z, body_quat.w),
            cuboid_size,
            BLACK.lighten(0.25)
        );
    }

}

fn render_physics_objects(physics_world: &PhysicsWorld) {

    for (_rigid_body_handle, rigid_body) in physics_world.state.rigid_body_set.iter() {

        for &collider in rigid_body.colliders() {
            let physics_shape = physics_world.state.collider_set.get(collider).unwrap().shape();
            render_physics_object(rigid_body, physics_shape);
        }

    }

}

fn render_voxel_world_bounds(voxel_world: &dyn VoxelWorld) {

    let (voxel_world_bounds_min, voxel_world_bounds_max) = voxel_world.get_world_bounds();
    let voxel_world_render_bounds_center = ((voxel_world_bounds_min + voxel_world_bounds_max) / 2.0) - Vec3::splat(VOXEL_SIZE) / 2.0;
    let voxel_world_render_bounds_size = voxel_world_bounds_max - voxel_world_bounds_min;

    draw_cube_wires(voxel_world_render_bounds_center, voxel_world_render_bounds_size, BLACK);

}

fn render_debug_text(game: &mut Game) {

    let dt = get_frame_time() * 1000.0;
    game.debug_text.draw_text(format!("frametime: {:.2} ms", dt), TextPosition::TopRight, BLACK);

}

fn try_pick_block_in_world(game: &mut Game) -> Option<(Vec3, VoxelKind)> {
    let on_voxel_face = false;
    try_pick_block_in_world_ex(game, on_voxel_face)
}

fn try_pick_block_in_world_ex(game: &mut Game, on_voxel_face: bool) -> Option<(Vec3, VoxelKind)> {

    let mouse_screen_pos: Vec2 = mouse_position().into();
    let near_target = game.camera.screen_to_world(mouse_screen_pos, 0.0).round() + VOXEL_DIMENSIONS / 2.0;
    let picking_dir = game.camera.screen_to_world_ray(mouse_screen_pos);

    let picked_block = game.voxel_world.write()
        .unwrap().try_pick_block_in_world(near_target, picking_dir);

    if on_voxel_face == false {

        picked_block

    } else {

        if let Some((pos, _kind)) = picked_block {

            let physics_ray_origin = Point::new(near_target.x, near_target.y, near_target.z);
            let physics_ray_direction = Vector::new(picking_dir.x, picking_dir.y, picking_dir.z);
            let physics_ray = Ray::new(physics_ray_origin, physics_ray_direction);
    
            let inferred_face_normal = VoxelWorldShape::infer_face_normal_from_ray(physics_ray);
            let position_offset = vec3(inferred_face_normal.x, inferred_face_normal.y, inferred_face_normal.z);
    
            let position_infront_of_picked_block = pos + position_offset;
            Some((position_infront_of_picked_block, _kind))

        } else {
            None
        }

    }

}

fn render_current_block_under_mouse(game: &mut Game) {

    game.debug_text.draw_text(format!("camera position: {}", game.camera.position), TextPosition::TopLeft, BLACK);

    if let Some((pos, _kind)) = try_pick_block_in_world(game) {
        game.debug_text.draw_text(format!("picked block position: {}", pos), TextPosition::TopLeft, BLACK);
        draw_cube_wires(pos, VOXEL_DIMENSIONS, BLACK);
    }

}

fn handle_spawn_object_on_click(game: &mut Game) {

    let was_left_mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
    let was_right_mouse_pressed = is_mouse_button_pressed(MouseButton::Right);
    let was_alt_pressed = is_key_down(KeyCode::LeftAlt);

    if (was_left_mouse_pressed || was_right_mouse_pressed) && was_alt_pressed {

        let should_spawn_ball = was_left_mouse_pressed;
        let should_spawn_box = was_right_mouse_pressed;

        if let Some((pos, _kind)) = try_pick_block_in_world(game) {

            let height_to_spawn_at = 4.0;
            let position_above_current_world_position = pos + WORLD_UP * height_to_spawn_at;

            if should_spawn_ball {
                game.physics_world.add_ball(position_above_current_world_position);
            }
            
            if should_spawn_box {
                game.physics_world.add_box(position_above_current_world_position);
            }

        }

    }

}

fn handle_modify_block_on_click(game: &mut Game) {

    let was_left_mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
    let was_right_mouse_pressed = is_mouse_button_pressed(MouseButton::Right);
    let was_alt_pressed = is_key_down(KeyCode::LeftAlt);

    if (was_left_mouse_pressed || was_right_mouse_pressed) && was_alt_pressed == false {

        let should_set_air = was_left_mouse_pressed;
        let should_set_grass = was_right_mouse_pressed;
        let should_use_face = should_set_grass;

        if let Some((pos, _kind)) = try_pick_block_in_world_ex(game, should_use_face) {

            let mut voxel_world_writer = game.voxel_world.write().unwrap();
            let voxel_position = ivec3(pos.x as i32, pos.y as i32, pos.z as i32);

            if should_set_air {
                voxel_world_writer.set_block(voxel_position, VoxelKind::Air);
            }

            if should_set_grass {
                voxel_world_writer.set_block(voxel_position, VoxelKind::Grass);
            }

        }

    }

}

fn handle_spawning_objects_and_modifying_world(game: &mut Game) {

    game.debug_text.draw_text("", TextPosition::TopLeft, BLACK);
    game.debug_text.draw_text(format!("left/right click to spawn objects (with left alt held)"), TextPosition::TopLeft, BLACK);
    game.debug_text.draw_text(format!("left/right click to remove/add blocks"), TextPosition::TopLeft, BLACK);

    handle_spawn_object_on_click(game);
    handle_modify_block_on_click(game);

}

fn handle_toggle_debug_parameters(game: &mut Game) {

    let should_reset_simulation = is_key_pressed(KeyCode::R);

    if should_reset_simulation {
        initialize_world(game);
    }

    game.debug_text.draw_text("press r to reset the world state", TextPosition::TopLeft, BLACK);

    let should_toggle_debug_parameters = is_key_pressed(KeyCode::T);

    if should_toggle_debug_parameters {
        game.debug_parameters.should_show_contacts = !game.debug_parameters.should_show_contacts;
    }

    game.debug_text.draw_text(
        format!("press t to toggle showing contacts (currently: {})", game.debug_parameters.should_show_contacts),
        TextPosition::TopLeft,
        BLACK
    );

}

/// Draws all current physics contacts, contacts are in world space.
fn render_physics_contacts(physics_world: &PhysicsWorld) {

    let contacts = physics_world.contact_points();

    for contact_world_position in contacts {
        let contact_sphere_size = 0.1;
        draw_sphere_ex(
            vec3(contact_world_position.x, contact_world_position.y, contact_world_position.z),
            contact_sphere_size,
            None,
            RED,
            DrawSphereParams { rings: 8, slices: 8, draw_mode: DrawMode::Triangles }
        );
    }

}

fn initialize_world(game: &mut Game) {

    let voxel_world_size = 8;
    let voxel_world_render_size = voxel_world_size as f32 * VOXEL_SIZE;

    // recreate the physics world
    game.physics_world = PhysicsWorld::new();

    // calculate the center of the voxel world?
    let voxel_world_center = vec3(voxel_world_render_size, voxel_world_render_size, voxel_world_render_size) / 2.0;

    // generate a very basic world
    let mut basic_voxel_world = VoxelWorldSimple::new();
    basic_voxel_world.generate_world(voxel_world_size, voxel_world_size, voxel_world_size);
    game.voxel_world = Arc::new(RwLock::new(basic_voxel_world));

    // add the physical voxel world
    game.physics_world.add_voxel_world(VoxelWorldShape { world: game.voxel_world.clone() });

    // set the camera position and target
    game.camera.position = vec3(0.0, (voxel_world_render_size * 2.0) as f32, -(voxel_world_render_size as f32 * 0.75));
    game.camera.target = voxel_world_center;

    // set camera speed
    game.camera.parameters.movement_speed = 8.0;

}

#[macroquad::main("voxel-physics")]
async fn main() {

    let mut game = Game::new();

    initialize_world(&mut game);

    loop {

        let dt = get_frame_time();
        clear_background(WHITE);

        set_camera(&create_camera_from_game_camera(&game.camera));

        // render voxel world (and its bounds)
        game.debug_text.benchmark_execution(
            || {
                let voxel_world_reader = game.voxel_world.read().unwrap();
                render_voxel_world(&*voxel_world_reader);
                render_voxel_world_bounds(&*voxel_world_reader);
            },
            "render_voxel_world",
            TextPosition::TopRight,
            BLACK
        );

        game.debug_text.benchmark_execution(
            || {
                render_physics_objects(&game.physics_world);
            },
            "render_physics_objects",
            TextPosition::TopRight,
            BLACK
        );

        handle_toggle_debug_parameters(&mut game);

        if game.debug_parameters.should_show_contacts {
            game.debug_text.benchmark_execution(
                || {
                    render_physics_contacts(&mut game.physics_world);
                },
                "render_physics_contacts",
                TextPosition::TopRight,
                BLACK
            );
        }

        // render current picked block, if any
        render_current_block_under_mouse(&mut game);

        // update camera position etc
        handle_camera_input(&mut game.camera, dt);

        // handle spawning shit, modifying
        handle_spawning_objects_and_modifying_world(&mut game);
        
        // step physics world
        game.debug_text.benchmark_execution(
            || {
                game.physics_world.step();
            },
            "physics_world_step",
            TextPosition::TopRight,
            BLACK
        );

        // ui, debug text, etc
        set_default_camera();

        game.debug_text.new_frame();
        render_debug_text(&mut game);

        next_frame().await;

    }
    
}
