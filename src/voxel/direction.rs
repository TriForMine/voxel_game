pub enum Direction {
    Left,
    Right,
    Down,
    Up,
    Back,
    Forward,
}

impl Direction {
    pub fn get_normal(&self) -> bevy::math::Vec3 {
        match self {
            Direction::Left => -bevy::math::Vec3::X,
            Direction::Right => bevy::math::Vec3::X,
            Direction::Down => -bevy::math::Vec3::Y,
            Direction::Up => bevy::math::Vec3::Y,
            Direction::Back => -bevy::math::Vec3::Z,
            Direction::Forward => bevy::math::Vec3::Z,
        }
    }
}
