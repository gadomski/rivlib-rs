/// An inclination reading.
#[derive(Clone, Copy, Debug)]
pub struct Inclination {
    /// The time the inclination was recorded.
    pub time: f64,

    /// The rotation around the x-axis in degrees.
    pub roll: f32,

    /// The rotation around the y-axis in degrees.
    pub pitch: f32,
}
