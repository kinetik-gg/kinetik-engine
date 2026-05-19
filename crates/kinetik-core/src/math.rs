/// Two-dimensional floating-point vector.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Vec2 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
}

impl Vec2 {
    /// Zero vector.
    pub const ZERO: Self = Self::splat(0.0);

    /// One vector.
    pub const ONE: Self = Self::splat(1.0);

    /// Unit vector on the positive X axis.
    pub const X: Self = Self::new(1.0, 0.0);

    /// Unit vector on the positive Y axis.
    pub const Y: Self = Self::new(0.0, 1.0);

    /// Creates a vector from components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Creates a vector with every component set to `value`.
    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value)
    }
}

/// Three-dimensional floating-point vector.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Vec3 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

impl Vec3 {
    /// Zero vector.
    pub const ZERO: Self = Self::splat(0.0);

    /// One vector.
    pub const ONE: Self = Self::splat(1.0);

    /// Unit vector on the positive X axis.
    pub const X: Self = Self::new(1.0, 0.0, 0.0);

    /// Unit vector on the positive Y axis.
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);

    /// Unit vector on the positive Z axis.
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    /// Creates a vector from components.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a vector with every component set to `value`.
    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }
}

/// Four-dimensional floating-point vector.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Vec4 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
    /// W component.
    pub w: f32,
}

impl Vec4 {
    /// Zero vector.
    pub const ZERO: Self = Self::splat(0.0);

    /// One vector.
    pub const ONE: Self = Self::splat(1.0);

    /// Creates a vector from components.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Creates a vector with every component set to `value`.
    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value, value)
    }
}

/// Quaternion rotation value.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Quat {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
    /// W component.
    pub w: f32,
}

impl Quat {
    /// Identity rotation.
    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Creates a quaternion from components.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Linear RGBA color.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    /// Red channel.
    pub r: f32,
    /// Green channel.
    pub g: f32,
    /// Blue channel.
    pub b: f32,
    /// Alpha channel.
    pub a: f32,
}

impl Color {
    /// Opaque white.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// Opaque black.
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a color from red, green, blue, and alpha channels.
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an opaque color from red, green, and blue channels.
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Position, rotation, and scale transform.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform {
    /// World or local position, depending on the owning system.
    pub position: Vec3,
    /// World or local rotation, depending on the owning system.
    pub rotation: Quat,
    /// World or local scale, depending on the owning system.
    pub scale: Vec3,
}

impl Transform {
    /// Identity transform.
    pub const IDENTITY: Self = Self::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

    /// Creates a transform from position, rotation, and scale.
    #[must_use]
    pub const fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Creates an identity transform translated to `position`.
    #[must_use]
    pub const fn from_position(position: Vec3) -> Self {
        Self::new(position, Quat::IDENTITY, Vec3::ONE)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Two-dimensional rectangle represented by minimum corner and size.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Rect {
    /// Minimum corner.
    pub min: Vec2,
    /// Rectangle size.
    pub size: Vec2,
}

impl Rect {
    /// Zero-sized rectangle at the origin.
    pub const ZERO: Self = Self::new(Vec2::ZERO, Vec2::ZERO);

    /// Creates a rectangle from minimum corner and size.
    #[must_use]
    pub const fn new(min: Vec2, size: Vec2) -> Self {
        Self { min, size }
    }

    /// Returns the maximum corner.
    #[must_use]
    pub fn max(self) -> Vec2 {
        Vec2::new(self.min.x + self.size.x, self.min.y + self.size.y)
    }
}

/// Axis-aligned bounding box represented by minimum and maximum corners.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Aabb {
    /// Minimum corner.
    pub min: Vec3,
    /// Maximum corner.
    pub max: Vec3,
}

impl Aabb {
    /// Zero-sized box at the origin.
    pub const ZERO: Self = Self::new(Vec3::ZERO, Vec3::ZERO);

    /// Creates an axis-aligned bounding box from minimum and maximum corners.
    #[must_use]
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Returns the box size.
    #[must_use]
    pub fn size(self) -> Vec3 {
        Vec3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    /// Returns the box center.
    #[must_use]
    pub fn center(self) -> Vec3 {
        Vec3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }
}
