/// An rxp point.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    /// The x coordinate.
    pub x: f32,

    /// The y coordinate.
    pub y: f32,

    /// The z coordinate.
    pub z: f32,

    /// The relative amplitude in dB.
    pub amplitude: f32,

    /// The relative reflectance in dB.
    pub reflectance: f32,

    /// A measure of pulse shape distortion.
    pub deviation: u16,

    /// The type of echo.
    pub echo_type: EchoType,

    /// Is a waveform available?
    pub is_waveform_available: bool,

    /// Is this a pseudo echo with fixed range 0.1m?
    pub is_pseudo_echo: bool,

    /// Is this a sw calcualted target?
    pub is_sw_target: bool,

    /// Is the pps not older than 1.5s?
    pub with_fresh_pps: bool,

    /// Is the time value in the pps timeframe?
    pub is_time_in_pps_timeframe: bool,

    /// The facet or segment number.
    pub facet_number: u8,

    /// The time the point was recorded in seconds.
    pub time: f64,
}

/// The type of echo.
#[derive(Clone, Copy, Debug)]
pub enum EchoType {
    /// The only echo from this pulse.
    Single,

    /// The first of multiple echos from this pulse.
    First,

    /// The second through n-1 echo from this pulse.
    Interior,

    /// The last echo from this pulse.
    Last,
}

impl From<u16> for EchoType {
    fn from(n: u16) -> EchoType {
        match n & 3 {
            0 => EchoType::Single,
            1 => EchoType::First,
            2 => EchoType::Interior,
            3 => EchoType::Last,
            _ => unreachable!(),
        }
    }
}
