use std::f32::consts::TAU;
use bevy::math::Vec2;
use rand::Rng;

pub fn rotate_vector(vec: Vec2, angle_deg: f32) -> Vec2 {
    let angle_rad = angle_deg.to_radians();
    let cos_angle = angle_rad.cos();
    let sin_angle = angle_rad.sin();

    Vec2::new(
        vec.x * cos_angle - vec.y * sin_angle,
        vec.x * sin_angle + vec.y * cos_angle,
    )
}

pub fn random_normalized_direction() -> Vec2 {
    let mut rng = rand::thread_rng();
    let random_angle = rng.gen_range(0.0..TAU); // Random angle in radians
    Vec2::new(random_angle.cos(), random_angle.sin()).normalize()
}

