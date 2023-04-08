// let x = vec2(32.0, 32.0);
// print(x);

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

let head_position = -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -TORSO_LENGTH, 0.0) - vec3(0.0, -(HEAD_SIZE / 2.0), 0.0);
let head_rotation = quat_identity();
let head_size = vec3(HEAD_SIZE, HEAD_SIZE, HEAD_SIZE);

let arm_left_position = -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -TORSO_LENGTH, 0.0) - vec3(-TORSO_WIDTH, SHOULDER_OFFSET, 0.0);
let arm_left_rotation = from_rotation_z(PI / 2.0);
let arm_left_size = vec3(LIMB_WIDTH, ARM_LENGTH, LIMB_WIDTH);

let arm_right_position = -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -TORSO_LENGTH, 0.0) - vec3(TORSO_WIDTH, SHOULDER_OFFSET, 0.0);
let arm_right_rotation = from_rotation_z(PI / 2.0);
let arm_right_size = vec3(LIMB_WIDTH, ARM_LENGTH, LIMB_WIDTH);

let torso_position = -vec3(0.0, -LEG_LENGTH, 0.0) - vec3(0.0, -(TORSO_LENGTH / 2.0), 0.0);
let torso_rotation = quat_identity();
let torso_size = vec3(TORSO_WIDTH, TORSO_LENGTH, TORSO_DEPTH);

let leg_left_position = -vec3(LIMB_WIDTH, -(LEG_LENGTH / 2.0), 0.0);
let leg_left_rotation = quat_identity();
let leg_left_size = vec3(LIMB_WIDTH, LEG_LENGTH, LIMB_WIDTH);

let leg_right_position = -vec3(-LIMB_WIDTH, -(LEG_LENGTH / 2.0), 0.0);
let leg_right_rotation = quat_identity();
let leg_right_size = vec3(LIMB_WIDTH, LEG_LENGTH, LIMB_WIDTH);

let current_animation_state = "walking";

// should be a function, but constant propagation doesn't work right now so can't make it one
if current_animation_state == "walking" {
// def play_walking_animation(transform) {

    let normalized_time = get_time() % (2.0 * PI);
    let current_t = (normalized_time * transform.velocity.length() * 2.0).sin();

    let current_left_arm_angle = normalize(current_t, 0.0, 0.75, 1.0);
    let current_right_arm_angle = -current_left_arm_angle;

    // draw torso
    draw_with_transformation(torso_position, from_rotation_z(0.25 * current_t * 0.5), || {

        // draw head
        draw_part(
            vec3(0.0, (TORSO_LENGTH / 2.0) + head_size.y / 2.0, 0.0),
            head_rotation,
            head_size
        );

        // draw left arm
        draw_with_transformation(vec3(-TORSO_WIDTH / 2.0, TORSO_LENGTH / 2.0 - SHOULDER_OFFSET / 2.0, 0.0), from_rotation_x(current_left_arm_angle) * from_rotation_z(4.25), || {

            draw_part(
                vec3(TORSO_WIDTH / 2.0, 0.0, 0.0),
                arm_left_rotation,
                arm_left_size
            );

        });

        // draw right arm
        draw_with_transformation(vec3(TORSO_WIDTH / 2.0, TORSO_LENGTH / 2.0 - SHOULDER_OFFSET / 2.0, 0.0), from_rotation_x(current_right_arm_angle) * from_rotation_z(-4.25), || {

            draw_part(
                vec3(-TORSO_WIDTH / 2.0, 0.0, 0.0),
                arm_right_rotation,
                arm_right_size
            );

        });

        draw_part(
            vec3_zero(),
            torso_rotation,
            torso_size
        );

        // draw backpack
        draw_with_transformation(vec3_zero(), from_rotation_x(0.0), || {
            draw_part(
                vec3(0.0, 0.0, TORSO_DEPTH*0.9),
                torso_rotation,
                torso_size * 0.75
            );
        });

    });

    // draw legs

    // left leg
    let current_left_leg_angle = normalize(current_t, 0.0, PI/4.0, 1.0);
    let current_right_leg_angle = -current_left_leg_angle;

    draw_with_transformation(vec3(0.0, leg_left_position.y * 2.0, 0.0), from_rotation_x(current_left_leg_angle), || {
        draw_part(
            vec3(leg_left_position.x, -leg_left_position.y, leg_left_position.z),
            leg_left_rotation,
            leg_left_size
        );
    });

    // right leg
    draw_with_transformation(vec3(0.0, leg_right_position.y * 2.0, 0.0), from_rotation_x(current_right_leg_angle), || {
        draw_part(
            vec3(leg_right_position.x, -leg_right_position.y, leg_right_position.z),
            leg_right_rotation,
            leg_right_size
        );
    });

}

// play_walking_animation(transform);