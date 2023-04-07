// let x = vec2(32.0, 32.0);
// print(x);

const PI = 3.14159265358979323846264338327950288;

let head_size = 0.2;

let limb_width = 0.15;
let torso_width = 0.5;
let torso_depth = torso_width / 2.0;

let shoulder_offset = limb_width;

let arm_length = 0.5;
let torso_length = 0.618 + 0.3;
let leg_length = 1.0;

// the entity position is at the entity's feet in our setup

let head_position = -vec3(0.0, -leg_length, 0.0) - vec3(0.0, -torso_length, 0.0) - vec3(0.0, -(head_size / 2.0), 0.0);
let head_rotation = quat_identity();
let head_size = vec3(head_size, head_size, head_size);

let arm_left_position = -vec3(0.0, -leg_length, 0.0) - vec3(0.0, -torso_length, 0.0) - vec3(-torso_width, shoulder_offset, 0.0);
let arm_left_rotation = from_rotation_z(PI / 2.0);
let arm_left_size = vec3(limb_width, arm_length, limb_width);

let arm_right_position = -vec3(0.0, -leg_length, 0.0) - vec3(0.0, -torso_length, 0.0) - vec3(torso_width, shoulder_offset, 0.0);
let arm_right_rotation = from_rotation_z(PI / 2.0);
let arm_right_size = vec3(limb_width, arm_length, limb_width);

let torso_position = -vec3(0.0, -leg_length, 0.0) - vec3(0.0, -(torso_length / 2.0), 0.0);
let torso_rotation = quat_identity();
let torso_size = vec3(torso_width, torso_length, torso_depth);

let leg_left_position = -vec3(limb_width, -(leg_length / 2.0), 0.0);
let leg_left_rotation = quat_identity();
let leg_left_size = vec3(limb_width, leg_length, limb_width);

let leg_right_position = -vec3(-limb_width, -(leg_length / 2.0), 0.0);
let leg_right_rotation = quat_identity();
let leg_right_size = vec3(limb_width, leg_length, limb_width);

// draw_with_transformation(vec3(0.0, 0.0, 0.0), from_rotation_z(0.0), || 
//     draw_part(vec3(0.0, 0.0, 0.0), quat_identity(), vec3(1.0, 1.0, 1.0))
// );

let normalized_time = get_time() % (2.0 * PI);
let current_t = (normalized_time * transform.velocity.length() * 2.0).sin();

// draw arms

// left arm

let current_left_arm_angle = normalize(current_t, 0.0, 0.75, 1.0);
let current_right_arm_angle = -current_left_arm_angle;

// draw_with_transformation(-vec3(0.0, -leg_length, 0.0) - vec3(0.0, -torso_length, 0.0) + vec3(0.0, -shoulder_offset, 0.0), from_rotation_x(current_left_arm_angle), || {

//     draw_with_transformation(vec3(torso_width / 2.0, 0.0, 0.0), from_rotation_z(-4.0), || {

//         draw_part(
//             vec3(-torso_width / 2.0, 0.0, 0.0),
//             arm_right_rotation,
//             arm_right_size
//         );

//     });

// });

// // right arm

// draw_with_transformation(-vec3(0.0, -leg_length, 0.0) - vec3(0.0, -torso_length, 0.0) + vec3(0.0, -shoulder_offset, 0.0), from_rotation_x(current_right_arm_angle), || {

//     draw_with_transformation(vec3(-torso_width / 2.0, 0.0, 0.0), from_rotation_z(4.0), || {

//         draw_part(
//             vec3(torso_width / 2.0, 0.0, 0.0),
//             arm_left_rotation,
//             arm_left_size
//         );

//     });

// });


// draw torso
draw_with_transformation(torso_position, from_rotation_z(0.25 * current_t * 0.5), || {

    // draw head
    draw_part(
        vec3(0.0, (torso_length / 2.0) + head_size.y / 2.0, 0.0),
        head_rotation,
        head_size
    );

    // draw left arm
    draw_with_transformation(vec3(-torso_width / 2.0, torso_length / 2.0 - shoulder_offset / 2.0, 0.0), from_rotation_x(current_left_arm_angle) * from_rotation_z(4.0), || {

        draw_part(
            vec3(torso_width / 2.0, 0.0, 0.0),
            arm_left_rotation,
            arm_left_size
        );

    });

    // draw right arm
    draw_with_transformation(vec3(torso_width / 2.0, torso_length / 2.0 - shoulder_offset / 2.0, 0.0), from_rotation_x(current_right_arm_angle) * from_rotation_z(-4.0), || {

        draw_part(
            vec3(-torso_width / 2.0, 0.0, 0.0),
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
            vec3(0.0, 0.0, torso_depth*0.9),
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

