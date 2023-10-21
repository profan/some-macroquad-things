use crate::{frac0, frac1, sign};
use macroquad::prelude::Vec3;

pub fn voxel_traversal_3d<F>(p0: Vec3, p1: Vec3, mut function: F)
where
    F: FnMut(Vec3) -> bool,
{
    // vector from p0 to p1
    let v = p1 - p0;

    let d_x = sign(v.x);
    let d_y = sign(v.y);
    let d_z = sign(v.z);

    let t_delta_x = if d_x != 0.0 {
        (d_x / v.x).min(100000.0)
    } else {
        100000.0
    };
    let mut t_max_x = if d_x > 0.0 {
        t_delta_x * frac1(p0.x)
    } else {
        t_delta_x * frac0(p0.x)
    };

    let t_delta_y = if d_y != 0.0 {
        (d_y / v.y).min(100000.0)
    } else {
        100000.0
    };
    let mut t_max_y = if d_y > 0.0 {
        t_delta_y * frac1(p0.y)
    } else {
        t_delta_y * frac0(p0.y)
    };

    let t_delta_z = if d_z != 0.0 {
        (d_z / v.z).min(100000.0)
    } else {
        100000.0
    };
    let mut t_max_z = if d_z > 0.0 {
        t_delta_z * frac1(p0.z)
    } else {
        t_delta_z * frac0(p0.z)
    };

    let mut voxel = p0;

    loop {
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                voxel.x += d_x;
                t_max_x += t_delta_x;
            } else {
                voxel.z += d_z;
                t_max_z += t_delta_z;
            }
        } else {
            if t_max_y < t_max_z {
                voxel.y += d_y;
                t_max_y += t_delta_y;
            } else {
                voxel.z += d_z;
                t_max_z += t_delta_z;
            }
        }

        // process voxel here

        // uncomment to debug me lol
        // println!("voxel_traversal_3d - passed: {}", voxel);

        // visit
        let should_exit = function(voxel);
        if should_exit {
            break;
        }

        if v.x == 0.0 && v.y == 0.0 && v.z == 0.0 {
            break;
        }

        // original exit condition
        if t_max_x > 1.0 && t_max_y > 1.0 && t_max_z > 1.0 {
            break;
        }

        // we've hit our mark
        if voxel == p1 {
            break;
        }
    }
}