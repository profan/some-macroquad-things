// #TODO: inject these mathematical constants from the rust side somehow instead?
const PI = 3.14159265358979323846264338327950288;

const HEAD_SIZE = 0.2;

const LIMB_WIDTH = 0.15;
const TORSO_WIDTH = 0.5;
const TORSO_DEPTH = TORSO_WIDTH / 2.0;

const SHOULDER_OFFSET = LIMB_WIDTH * 2.0;

const ARM_LENGTH = 0.6;
const TORSO_LENGTH = 0.618 + 0.3;
const LEG_LENGTH = 1.0;

let character_data = #{

    head_position : -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -TORSO_LENGTH, 0.0) - vec3(0.0, -(HEAD_SIZE / 2.0), 0.0),
    head_rotation : quat_identity(),
    head_size : vec3(HEAD_SIZE, HEAD_SIZE, HEAD_SIZE),

    arm_left_position : -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -TORSO_LENGTH, 0.0) - vec3(-TORSO_WIDTH, SHOULDER_OFFSET, 0.0),
    arm_left_rotation : from_rotation_z(PI / 2.0),
    arm_left_size : vec3(LIMB_WIDTH, ARM_LENGTH, LIMB_WIDTH),

    arm_right_position : -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -TORSO_LENGTH, 0.0) - vec3(TORSO_WIDTH, SHOULDER_OFFSET, 0.0),
    arm_right_rotation : from_rotation_z(PI / 2.0),
    arm_right_size : vec3(LIMB_WIDTH, ARM_LENGTH, LIMB_WIDTH),

    torso_position : -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -(TORSO_LENGTH / 2.0), 0.0),
    torso_rotation : quat_identity(),
    torso_size : vec3(TORSO_WIDTH, TORSO_LENGTH, TORSO_DEPTH),

    leg_left_position : -vec3(LIMB_WIDTH, -(LEG_LENGTH / 2.0), 0.0),
    leg_left_rotation : quat_identity(),
    leg_left_size : vec3(LIMB_WIDTH, LEG_LENGTH, LIMB_WIDTH),

    leg_right_position : -vec3(-LIMB_WIDTH, -(LEG_LENGTH / 2.0), 0.0),
    leg_right_rotation : quat_identity(),
    leg_right_size : vec3(LIMB_WIDTH, LEG_LENGTH, LIMB_WIDTH),

};

fn create_bone(position, rotation, parent) {
    return #{
        position: position,
        rotation: rotation,
        parent: parent
    };
}

// returns a map representing the rigged character and its bones in the idle pose
fn create_idle_pose() {

    let character_rig = #{

        neck_bone: create_bone(vec3(0.0, TORSO_LENGTH / 2.0, 0.0), quat_identity(), "hip_bone"),
        hip_bone: create_bone(vec3(0.0, LEG_LENGTH * 1.5, 0.0), quat_identity(), ()),

        shoulder_left_bone: create_bone(vec3(-TORSO_WIDTH / 2.0 - LIMB_WIDTH / 2.0 - SHOULDER_OFFSET - LIMB_WIDTH / 2.0, -SHOULDER_OFFSET / 2.0, 0.0), quat_identity(), "hip_bone"),
        shoulder_right_bone: create_bone(vec3(TORSO_WIDTH / 2.0 + LIMB_WIDTH / 2.0 + SHOULDER_OFFSET + LIMB_WIDTH / 2.0, -SHOULDER_OFFSET / 2.0, 0.0), quat_identity(), "hip_bone"),
        
        leg_left_bone: create_bone(vec3(-LIMB_WIDTH, -(LEG_LENGTH / 2.0), 0.0), quat_identity(), "hip_bone"),
        leg_right_bone: create_bone(vec3(LIMB_WIDTH, -(LEG_LENGTH / 2.0), 0.0), quat_identity(), "hip_bone"),

    };

    return character_rig;

}

fn create_walking_pose(transform) {

    let normalized_time = get_time() % (2.0 * PI);
    let current_t = (normalized_time * transform.velocity.length() * 2.0).sin();

    let current_left_arm_angle = normalize(current_t, 0.0, 0.75, 1.0);
    let current_right_arm_angle = -current_left_arm_angle;

    let current_left_leg_angle = normalize(current_t, 0.0, PI/4.0, 1.0);
    let current_right_leg_angle = -current_left_leg_angle;

    let current_torso_sway = 0.25 * current_t * 0.5;

    let idle_pose = create_idle_pose();

    let walking_pose = #{

        shoulder_left_bone: idle_pose.shoulder_left_bone + #{rotation: from_rotation_x(current_left_arm_angle) * from_rotation_z(2.0)},
        shoulder_right_bone: idle_pose.shoulder_right_bone + #{rotation: from_rotation_x(current_right_arm_angle) * from_rotation_z(-2.0)},

        leg_left_bone: idle_pose.leg_left_bone + #{rotation: from_rotation_x(current_left_leg_angle)},
        leg_right_bone: idle_pose.leg_right_bone + #{rotation: from_rotation_x(current_right_leg_angle)},

        hip_bone: idle_pose.hip_bone + #{rotation: from_rotation_z(current_torso_sway)},

    };

    return idle_pose + walking_pose;

}

fn create_proning_pose(transform) {

    let normalized_time = get_time() % (2.0 * PI);
    let current_t = (normalized_time * transform.velocity.length() * 2.0).sin();

    let current_left_arm_angle = normalize(current_t, 0.0, 0.75, 1.0);
    let current_right_arm_angle = -current_left_arm_angle;

    let current_left_leg_angle = normalize(current_t, 0.0, PI/6.0, 1.0);
    let current_right_leg_angle = -current_left_leg_angle;

    let current_torso_sway = 0.25 * current_t * 0.5;

    let idle_pose = create_idle_pose();

    let proning_pose = #{

        neck_bone: idle_pose.neck_bone + #{rotation: from_rotation_x(0.25) * from_rotation_y(current_torso_sway)},

        shoulder_left_bone: idle_pose.shoulder_left_bone + #{rotation: from_rotation_y(-0.25 + current_left_arm_angle)},
        shoulder_right_bone: idle_pose.shoulder_right_bone + #{rotation: from_rotation_y(0.25 - current_right_arm_angle)},

        leg_left_bone: idle_pose.leg_left_bone + #{rotation: from_rotation_x(0.125) * from_rotation_y(-current_left_leg_angle)},
        leg_right_bone: idle_pose.leg_right_bone + #{rotation: from_rotation_x(0.125) * from_rotation_y(current_right_leg_angle)},

        hip_bone: idle_pose.hip_bone + #{rotation: from_rotation_y(current_torso_sway) * from_rotation_x(-(PI / 2.0))}

    };

    return idle_pose + proning_pose;

}

fn calculate_bone(bone, rig) {

    let calculated_bone_position = vec3_zero();
    let calculated_bone_rotation = bone.rotation;

    let current_bone = bone;
    while current_bone.parent != () {

        let parent_bone = rig[current_bone.parent];
        current_bone = rig[current_bone.parent];

        calculated_bone_rotation *= parent_bone.rotation;
        calculated_bone_position += parent_bone.rotation * current_bone.position; // here we append position in parent

    }

    return #{
        position: calculated_bone_position + calculated_bone_rotation * bone.position,
        rotation: calculated_bone_rotation,
        parent: bone.parent
    };

}

fn calculate_character_pose(character_rig) {

    let calculated_pose = #{};
    
    for bone_key in character_rig.keys() {
        let bone = character_rig[bone_key];
        calculated_pose[bone_key] = calculate_bone(bone, character_rig);
    }

    return calculated_pose;

}

fn blend_character_pose(pose_one, pose_two, value) {

    let blended_pose = #{};

    for bone_key in pose_one.keys() {

        let bone_one = pose_one[bone_key];
        let bone_two = pose_two[bone_key];

        blended_pose[bone_key] = #{
            position: bone_one.position.lerp(bone_two.position, value),
            rotation: bone_one.rotation.slerp(bone_two.rotation, value),
            parent: bone_one.parent
        };

    }

    return blended_pose;

}

let normalized_time = get_time() % (2.0 * PI);
let current_t = (normalized_time * 2.0).sin();

let current_animation = "walking";

let current_pose = switch current_animation {
    "idle" => create_idle_pose(),
    "walking" => create_walking_pose(transform),
    "proning" => create_proning_pose(transform),
};

// let idle_pose = create_idle_pose();
// let walking_pose = create_walking_pose(transform);
// let proning_pose = create_proning_pose(transform);

// let final_idle_character_pose = calculate_character_pose(idle_pose);
// let final_walking_character_pose = calculate_character_pose(walking_pose);
// let final_proning_character_pose = calculate_character_pose(proning_pose);

// let calculated_character_pose = calculate_character_pose(blend_character_pose(idle_pose, walking_pose, current_t));

let current_character_pose = calculate_character_pose(current_pose);
draw_character_in_pose(character_data, current_character_pose);

fn get_bone_parent_or_identity(character_pose_data, bone) {

    if bone.parent != () {
        return character_pose_data[bone.parent];
    } else {
        return bone;
    }

}

fn draw_character_in_pose(character_data, character_pose_data) {

    let neck_bone = character_pose_data.neck_bone;
    let hip_bone = character_pose_data.hip_bone;

    let shoulder_left_bone = character_pose_data.shoulder_left_bone; 
    let shoulder_right_bone = character_pose_data.shoulder_right_bone; 

    let leg_left_bone = character_pose_data.leg_left_bone;
    let leg_right_bone = character_pose_data.leg_right_bone;

    // head
    draw_part(
        neck_bone.position + neck_bone.rotation * vec3(0.0, HEAD_SIZE / 2.0, 0.0),
        neck_bone.rotation,
        character_data.head_size
    );

    // torso
    draw_part(
        hip_bone.position,
        hip_bone.rotation,
        character_data.torso_size
    );
    
    // backpack
    draw_part(
        hip_bone.position + hip_bone.rotation * vec3(0.0, 0.0, TORSO_DEPTH*0.9),
        hip_bone.rotation,
        character_data.torso_size * 0.75
    );

    // left arm
    draw_part(
        shoulder_left_bone.position + shoulder_left_bone.rotation * vec3(LIMB_WIDTH, 0.0, 0.0) + get_bone_parent_or_identity(character_pose_data, shoulder_left_bone).rotation * vec3(0.0, TORSO_LENGTH / 2.0, 0.0),
        (shoulder_left_bone.rotation * character_data.arm_left_rotation),
        character_data.arm_left_size
    );

    // right arm
    draw_part(
        shoulder_right_bone.position + shoulder_right_bone.rotation * vec3(-LIMB_WIDTH, 0.0, 0.0) + get_bone_parent_or_identity(character_pose_data, shoulder_right_bone).rotation * vec3(0.0, TORSO_LENGTH / 2.0, 0.0),
        shoulder_right_bone.rotation * character_data.arm_right_rotation,
        character_data.arm_right_size
    );

    // left leg
    draw_part(
        leg_left_bone.position - character_pose_data[leg_left_bone.parent].rotation * vec3(0.0, LEG_LENGTH / 2.0, 0.0),
        leg_left_bone.rotation,
        character_data.leg_left_size
    );

    // right leg
    draw_part(
        leg_right_bone.position - character_pose_data[leg_right_bone.parent].rotation * vec3(0.0, LEG_LENGTH / 2.0, 0.0),
        leg_right_bone.rotation,
        character_data.leg_right_size
    );

}

// the entity position is at the entity's feet in our setup

fn play_walking_animation(transform, data) {

    let normalized_time = get_time() % (2.0 * PI);
    let current_t = (normalized_time * transform.velocity.length() * 2.0).sin();

    let current_left_arm_angle = normalize(current_t, 0.0, 0.75, 1.0);
    let current_right_arm_angle = -current_left_arm_angle;

    draw_with_transformation(vec3(0.0, data.leg_right_position.y * 2.0, 0.0), quat_identity(), || {

        // draw legs

        // left leg
        let current_left_leg_angle = normalize(current_t, 0.0, PI/4.0, 1.0);
        let current_right_leg_angle = -current_left_leg_angle;

        draw_with_transformation(vec3(0.0, 0.0, 0.0), from_rotation_x(current_left_leg_angle), || {
            draw_part(
                vec3(data.leg_left_position.x, -data.leg_left_position.y, data.leg_left_position.z),
                data.leg_left_rotation,
                data.leg_left_size
            );
        });

        // right leg
        draw_with_transformation(vec3(0.0, 0.0, 0.0), from_rotation_x(current_right_leg_angle), || {
            draw_part(
                vec3(data.leg_right_position.x, -data.leg_right_position.y, data.leg_right_position.z),
                data.leg_right_rotation,
                data.leg_right_size
            );
        });

        // draw torso
        draw_with_transformation(vec3(0.0, TORSO_LENGTH / 2.0, 0.0), from_rotation_z(0.25 * current_t * 0.5), || {

            // draw head
            draw_part(
                vec3(0.0, (TORSO_LENGTH / 2.0) + data.head_size.y / 2.0, 0.0),
                data.head_rotation,
                data.head_size
            );

            // draw left arm
            draw_with_transformation(vec3(-TORSO_WIDTH / 2.0, TORSO_LENGTH / 2.0 - SHOULDER_OFFSET / 2.0, 0.0), from_rotation_x(current_left_arm_angle) * from_rotation_z(4.25), || {

                draw_part(
                    vec3(TORSO_WIDTH / 2.0, 0.0, 0.0),
                    data.arm_left_rotation,
                    data.arm_left_size
                );

            });

            // draw right arm
            draw_with_transformation(vec3(TORSO_WIDTH / 2.0, TORSO_LENGTH / 2.0 - SHOULDER_OFFSET / 2.0, 0.0), from_rotation_x(current_right_arm_angle) * from_rotation_z(-4.25), || {

                draw_part(
                    vec3(-TORSO_WIDTH / 2.0, 0.0, 0.0),
                    data.arm_right_rotation,
                    data.arm_right_size
                );

            });

            draw_part(
                vec3_zero(),
                data.torso_rotation,
                data.torso_size
            );

            // draw backpack
            draw_with_transformation(vec3_zero(), from_rotation_x(0.0), || {
                draw_part(
                    vec3(0.0, 0.0, TORSO_DEPTH*0.9),
                    data.torso_rotation,
                    data.torso_size * 0.75
                );
            });

        });

    });

}