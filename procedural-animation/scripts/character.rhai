// #TODO: inject these mathematical constants from the rust side somehow instead?
const PI = 3.14159265358979323846264338327950288;

const HEAD_SIZE = 0.2;

const LIMB_WIDTH = 0.15;
const TORSO_WIDTH = 0.5;
const TORSO_DEPTH = TORSO_WIDTH / 2.0;

const SHOULDER_OFFSET = LIMB_WIDTH*2.0;

const ARM_LENGTH = 0.6;
const TORSO_LENGTH = 0.618 + 0.3;
const LEG_LENGTH = 1.0;

// the entity position is at the entity's feet in our setup

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

let current_animation_state = "walking";

switch current_animation_state {

    "walking" => play_walking_animation(transform, character_data),
    _ => play_idle_animation(transform, character_data)

}

fn play_idle_animation(transform, data) {

}

// should be a function, but constant propagation doesn't work right now so can't make it one
fn play_walking_animation(transform, data) {

    let normalized_time = get_time() % (2.0 * PI);
    let current_t = (normalized_time * transform.velocity.length() * 2.0).sin();

    let current_left_arm_angle = normalize(current_t, 0.0, 0.75, 1.0);
    let current_right_arm_angle = -current_left_arm_angle;

    // draw torso
    draw_with_transformation(data.torso_position, from_rotation_z(0.25 * current_t * 0.5), || {

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

    // draw legs

    // left leg
    let current_left_leg_angle = normalize(current_t, 0.0, PI/4.0, 1.0);
    let current_right_leg_angle = -current_left_leg_angle;

    draw_with_transformation(vec3(0.0, data.leg_left_position.y * 2.0, 0.0), from_rotation_x(current_left_leg_angle), || {
        draw_part(
            vec3(data.leg_left_position.x, -data.leg_left_position.y, data.leg_left_position.z),
            data.leg_left_rotation,
            data.leg_left_size
        );
    });

    // right leg
    draw_with_transformation(vec3(0.0, data.leg_right_position.y * 2.0, 0.0), from_rotation_x(current_right_leg_angle), || {
        draw_part(
            vec3(data.leg_right_position.x, -data.leg_right_position.y, data.leg_right_position.z),
            data.leg_right_rotation,
            data.leg_right_size
        );
    });

}