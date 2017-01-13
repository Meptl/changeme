use ::cgmath::Vector3;
use ::resource::Resource;

/// X is right. Y is up. Z is into the screen.
pub struct Scene {
    entities: [Entity]
}

/// Data concerning a single entity
pub struct Entity<'a> {
    resources: [&'a Resource],
    position: Vector3,
}
