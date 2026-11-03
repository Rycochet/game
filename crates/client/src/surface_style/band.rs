//! A colour band, and the kinds of surface a point shows.

/// A colour as hue (degrees, `0..360`) and saturation (`0..1`). Lightness is
/// worked out per pixel from height and slope.
#[derive(Clone, Copy, Debug)]
pub struct Band {
    pub hue: f64,
    pub saturation: f64,
}

/// What a surface point shows, which decides its colour band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    /// Land cold enough for snow, and anything above the snow line.
    Snow,
    /// Open land, coloured by its biome.
    Land,
    /// Land just below the water, showing where the tide has reached.
    Shore,
    Water,
}
