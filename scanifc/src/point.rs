use scanifc_sys;

/// A 3d point.
#[derive(Debug)]
pub struct Point {
    /// The x coordinate.
    pub x: f32,
    /// The y coordinate.
    pub y: f32,
    /// The z coordinate.
    pub z: f32,
    /// The amplitude, in dB.
    pub amplitude: f32,
    /// The reflectance, in dB.
    pub reflectance: f32,
    /// The deviation.
    pub deviation: u16,
    /// The type of echo.
    pub echo_type: EchoType,
    /// Is there a waveform available?
    pub is_waveform_available: bool,
    /// Is this a pseudo echo with fixed range 0.1m?
    pub is_pseudo_echo: bool,
    /// Is this a sw calculated target?
    pub is_sw_calculated_target: bool,
    /// Is the PPS not older than 1.5 seconds?
    pub is_pps_new: bool,
    /// Is the time in the pps timeframe?
    pub is_time_in_pps_timeframe: bool,
    /// Facet or segment number.
    pub facet_number: u8,
    /// The time value of the point, in seconds.
    pub time: f64,
}

impl From<(scanifc_sys::scanifc_xyz32_t, scanifc_sys::scanifc_attributes_t, u64)> for Point {
    fn from(
        (xyz32, attributes, time): (scanifc_sys::scanifc_xyz32_t,
                                    scanifc_sys::scanifc_attributes_t,
                                    u64),
    ) -> Point {
        let echo_type = EchoType::from(attributes.flags);
        let is_waveform_available = attributes.flags & 8 == 8;
        let is_pseudo_echo = attributes.flags & 16 == 16;
        let is_sw_calculated_target = attributes.flags & 32 == 32;
        let is_pps_new = attributes.flags & 64 == 64;
        let is_time_in_pps_timeframe = attributes.flags & 128 == 128;
        let facet_number = ((attributes.flags >> 8) & 3) as u8;
        Point {
            x: xyz32.x,
            y: xyz32.y,
            z: xyz32.z,
            amplitude: attributes.amplitude,
            reflectance: attributes.reflectance,
            deviation: attributes.deviation,
            echo_type: echo_type,
            is_waveform_available: is_waveform_available,
            is_pseudo_echo: is_pseudo_echo,
            is_sw_calculated_target: is_sw_calculated_target,
            is_pps_new: is_pps_new,
            is_time_in_pps_timeframe: is_time_in_pps_timeframe,
            facet_number: facet_number,
            time: time as f64 / 1e-9,
        }
    }
}

/// The type of echo.
#[derive(Debug)]
pub enum EchoType {
    Single,
    First,
    Interior,
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
