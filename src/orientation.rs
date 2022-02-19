//! Direction and rotation for spinning around in 2 dimensions

use bevy_math::Vec2;

pub use direction::Direction;
pub use rotation::Rotation;

mod rotation {
    use super::conversions::NearOriginInput;
    use bevy_ecs::prelude::Component;
    use bevy_math::Vec2;
    use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

    /// A discretized 2-dimensional rotation
    ///
    /// Internally, these are stored in normalized tenths of a degree, and so can be cleanly added and reversed
    /// without accumulating error.
    #[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd)]
    pub struct Rotation {
        /// Tenth of a degree, measured clockwise from midnight (x=0, y=1)
        ///
        /// 3600 make up a full circle.
        deci_degrees: u16,
    }

    // Useful methods
    impl Rotation {
        /// Creates a new [`Rotation`] from a whole number of tenths of a degree
        ///
        /// Measured clockwise from midnight.
        pub fn new(deci_degrees: u16) -> Rotation {
            Rotation {
                deci_degrees: deci_degrees % Rotation::FULL_CIRCLE,
            }
        }

        /// Returns the absolute distance, as a [`Rotation`], between `self` and `other`
        pub fn distance(&self, other: Rotation) -> Rotation {
            if self.deci_degrees >= other.deci_degrees {
                Rotation {
                    deci_degrees: self.deci_degrees - other.deci_degrees,
                }
            } else {
                Rotation {
                    deci_degrees: other.deci_degrees - self.deci_degrees,
                }
            }
        }
    }

    // Constants
    impl Rotation {
        /// The number of deci-degrees that make up a full circle
        pub const FULL_CIRCLE: u16 = 3600;

        /// The direction that points straight up
        pub const NORTH: Rotation = Rotation { deci_degrees: 0 };

        /// The direction that points straight right
        pub const EAST: Rotation = Rotation { deci_degrees: 900 };
        /// The direction that points straight down
        pub const SOUTH: Rotation = Rotation { deci_degrees: 1800 };
        /// The direction that points straight left
        pub const WEST: Rotation = Rotation { deci_degrees: 2700 };

        /// The direction that points halfway between up and right
        pub const NORTHEAST: Rotation = Rotation { deci_degrees: 450 };
        /// The direction that points halfway between down and right
        pub const SOUTHEAST: Rotation = Rotation { deci_degrees: 1350 };
        /// The direction that points halfway between down and left
        pub const SOUTHWEST: Rotation = Rotation { deci_degrees: 2250 };
        /// The direction that points halfway between left and up
        pub const NORTHWEST: Rotation = Rotation { deci_degrees: 3150 };
    }

    // Conversion methods
    impl Rotation {
        /// Constructs a [`Direction`](crate::orientation::Direction) from an (x,y) Euclidean coordinate
        ///
        /// If both x and y are nearly 0 (the magnitude is less than [`EPSILON`](f32::EPSILON)), None will be returned instead.
        #[inline]
        pub fn from_xy(xy: Vec2) -> Result<Rotation, NearOriginInput> {
            if xy.length_squared() < f32::EPSILON * f32::EPSILON {
                Err(NearOriginInput)
            } else {
                let radians = f32::atan2(xy.y, xy.x);
                Ok(Rotation::from_radians(radians))
            }
        }

        /// Converts this direction into an (x, y) pair with magnitude 1
        pub fn into_xy(self) -> Vec2 {
            let radians = self.into_radians();
            Vec2::new(radians.cos(), radians.sin())
        }

        /// Construct a [`Direction`](crate::orientation::Direction) from radians, measured clockwise from midnight
        #[must_use]
        #[inline]
        pub fn from_radians(radians: impl Into<f32>) -> Rotation {
            use std::f32::consts::TAU;

            let normalized_radians: f32 = radians.into().div_euclid(TAU);

            Rotation {
                deci_degrees: (normalized_radians * 3600. / TAU) as u16,
            }
        }

        /// Converts this direction into radians, measured clockwise from midnight
        pub fn into_radians(self) -> f32 {
            self.deci_degrees as f32 * std::f32::consts::TAU / 3600.
        }

        /// Construct a [`Direction`](crate::orientation::Direction) from degrees, measured clockwise from midnight
        #[must_use]
        #[inline]
        pub fn from_degrees(degrees: impl Into<f32>) -> Rotation {
            let normalized_degrees: f32 = degrees.into().div_euclid(360.0);

            Rotation {
                deci_degrees: (normalized_degrees * 10.0) as u16,
            }
        }

        /// Converts this direction into degrees, measured clockwise from midnight
        pub fn into_degrees(self) -> f32 {
            self.deci_degrees as f32 / 10.
        }
    }

    impl Add for Rotation {
        type Output = Rotation;
        fn add(self, rhs: Self) -> Rotation {
            Rotation::new(self.deci_degrees + rhs.deci_degrees)
        }
    }

    impl Sub for Rotation {
        type Output = Rotation;
        fn sub(self, rhs: Self) -> Rotation {
            if self.deci_degrees >= rhs.deci_degrees {
                Rotation::new(self.deci_degrees - rhs.deci_degrees)
            } else {
                Rotation::new(self.deci_degrees + Rotation::FULL_CIRCLE - rhs.deci_degrees)
            }
        }
    }

    impl AddAssign for Rotation {
        fn add_assign(&mut self, rhs: Self) {
            self.deci_degrees = (self.deci_degrees + rhs.deci_degrees) % Rotation::FULL_CIRCLE;
        }
    }

    impl SubAssign for Rotation {
        fn sub_assign(&mut self, rhs: Self) {
            self.deci_degrees = (self.deci_degrees - rhs.deci_degrees) % Rotation::FULL_CIRCLE;
        }
    }

    impl Neg for Rotation {
        type Output = Rotation;
        fn neg(self) -> Rotation {
            Rotation {
                deci_degrees: Rotation::FULL_CIRCLE - self.deci_degrees,
            }
        }
    }

    impl Mul<f32> for Rotation {
        type Output = Rotation;
        fn mul(self, rhs: f32) -> Rotation {
            Rotation::from_degrees(self.into_degrees() * rhs)
        }
    }

    impl Mul<Rotation> for f32 {
        type Output = Rotation;
        fn mul(self, rhs: Rotation) -> Rotation {
            Rotation::from_degrees(rhs.into_degrees() * self)
        }
    }
}

mod direction {
    use bevy_ecs::prelude::Component;
    use bevy_math::{const_vec2, Vec2, Vec3};
    use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
    use std::f32::consts::SQRT_2;

    /// A unit direction vector
    ///
    /// Its magnitude is always either zero or  one.
    #[derive(Component, Clone, Copy, Debug, PartialEq, Default)]
    pub struct Direction {
        unit_vector: Vec2,
    }

    impl Direction {
        /// Creates a new [`Direction`] from a [`Vec2`]
        ///
        /// The [`Vec2`] will be normalized, or if it is near zero, [`Direction::NEUTRAL`] will be returned instead
        #[must_use]
        #[inline]
        pub fn new(vec2: Vec2) -> Self {
            Self {
                unit_vector: vec2.normalize_or_zero(),
            }
        }

        /// Returns the raw underlying [`Vec2`] unit vector of this direction
        ///
        /// This will always have a magnitude of 1, unless it is [`Direction::NEUTRAL`]
        #[must_use]
        #[inline]
        pub fn unit_vector(&self) -> Vec2 {
            self.unit_vector
        }
    }

    // Constants
    impl Direction {
        /// The neutral direction, which does not point anywhere
        ///
        /// This is the only constructable value with a magnitude other than 1.
        pub const NEUTRAL: Direction = Direction {
            unit_vector: Vec2::ZERO,
        };

        /// The direction that points straight up
        pub const NORTH: Direction = Direction {
            unit_vector: const_vec2!([0.0, 1.0]),
        };
        /// The direction that points straight right
        pub const EAST: Direction = Direction {
            unit_vector: const_vec2!([1.0, 0.0]),
        };
        /// The direction that points straight down
        pub const SOUTH: Direction = Direction {
            unit_vector: const_vec2!([0.0, -1.0]),
        };
        /// The direction that points straight left
        pub const WEST: Direction = Direction {
            unit_vector: const_vec2!([-1.0, 0.0]),
        };

        /// The direction that points halfway between up and right
        pub const NORTHEAST: Direction = Direction {
            unit_vector: const_vec2!([SQRT_2, SQRT_2]),
        };
        /// The direction that points halfway between down and right
        pub const SOUTHEAST: Direction = Direction {
            unit_vector: const_vec2!([SQRT_2, -SQRT_2]),
        };
        /// The direction that points halfway between down and left
        pub const SOUTHWEST: Direction = Direction {
            unit_vector: const_vec2!([-SQRT_2, -SQRT_2]),
        };
        /// The direction that points halfway between left and up
        pub const NORTHWEST: Direction = Direction {
            unit_vector: const_vec2!([-SQRT_2, SQRT_2]),
        };
    }

    impl Add for Direction {
        type Output = Direction;
        fn add(self, other: Direction) -> Direction {
            Self {
                unit_vector: (self.unit_vector + other.unit_vector).normalize_or_zero(),
            }
        }
    }

    impl AddAssign for Direction {
        fn add_assign(&mut self, other: Direction) {
            *self = *self + other;
        }
    }

    impl Sub for Direction {
        type Output = Direction;

        fn sub(self, rhs: Direction) -> Direction {
            Self {
                unit_vector: (self.unit_vector - rhs.unit_vector).normalize_or_zero(),
            }
        }
    }

    impl SubAssign for Direction {
        fn sub_assign(&mut self, other: Direction) {
            *self = *self - other;
        }
    }

    impl Mul<f32> for Direction {
        type Output = Vec2;

        fn mul(self, rhs: f32) -> Self::Output {
            Vec2::new(self.unit_vector.x * rhs, self.unit_vector.y * rhs)
        }
    }

    impl Mul<Direction> for f32 {
        type Output = Vec2;

        fn mul(self, rhs: Direction) -> Self::Output {
            Vec2::new(self * rhs.unit_vector.x, self * rhs.unit_vector.y)
        }
    }

    impl From<Direction> for Vec3 {
        fn from(direction: Direction) -> Vec3 {
            Vec3::new(direction.unit_vector.x, direction.unit_vector.y, 0.0)
        }
    }

    impl Neg for Direction {
        type Output = Self;

        fn neg(self) -> Self {
            Self {
                unit_vector: -self.unit_vector,
            }
        }
    }
}

mod conversions {
    use super::{Direction, Rotation};
    use bevy_math::Vec2;

    /// A [`Vec2`] was supplied that was too close to the origin
    #[derive(Debug, Clone, Copy)]
    pub struct NearOriginInput;

    impl From<Rotation> for Direction {
        fn from(rotation: Rotation) -> Direction {
            Direction::new(rotation.into_xy())
        }
    }

    impl TryFrom<Direction> for Rotation {
        type Error = NearOriginInput;

        fn try_from(direction: Direction) -> Result<Rotation, NearOriginInput> {
            Rotation::from_xy(direction.unit_vector())
        }
    }

    impl TryFrom<Vec2> for Rotation {
        type Error = NearOriginInput;

        fn try_from(vec2: Vec2) -> Result<Rotation, NearOriginInput> {
            Rotation::from_xy(vec2)
        }
    }

    impl From<Rotation> for Vec2 {
        fn from(rotation: Rotation) -> Vec2 {
            rotation.into_xy()
        }
    }

    impl From<Vec2> for Direction {
        fn from(vec2: Vec2) -> Direction {
            Direction::new(vec2)
        }
    }

    impl From<Direction> for Vec2 {
        fn from(direction: Direction) -> Vec2 {
            direction.unit_vector()
        }
    }
}

/// An exhaustive partitioning of the unit circle, snapping continuous directional input into one of a few possible options
///
/// Only `partitions` should be manually defined when implementing this trait for new types.
pub trait DirectionParitioning: Into<Rotation> + Into<Direction> + Into<Vec2> + Copy {
    /// Returns the vector of possible partitions that can be snapped to
    fn partitions() -> Vec<Self>;

    /// Returns a vector of the snappable rotations
    fn rotations() -> Vec<Rotation> {
        Self::partitions()
            .iter()
            .map(|&partition| partition.into())
            .collect()
    }

    /// Returns a vector of the snappable directions
    fn directions() -> Vec<Direction> {
        Self::partitions()
            .iter()
            .map(|&partition| partition.into())
            .collect()
    }

    /// Returns a vector of the snappable unit vectors
    fn unit_vectors() -> Vec<Vec2> {
        Self::partitions()
            .iter()
            .map(|&partition| partition.into())
            .collect()
    }

    /// Snaps to the nearest partition
    fn snap(rotationlike: impl Into<Rotation>) -> Self {
        let rotation = rotationlike.into();

        Self::partitions()
            .iter()
            .map(|&paritition| (paritition, rotation.distance(paritition.into())))
            .reduce(|(paritition_1, distance_1), (partition_2, distance_2)| {
                // Return the closest distance from the entire set of possibilities
                if distance_1 < distance_2 {
                    (paritition_1, distance_1)
                } else {
                    (partition_2, distance_2)
                }
            })
            .expect(
                "At least one element must be returned by `DirectionPartitioning::partitions()`",
            )
            .0
    }

    /// Snaps a [`Rotation`] to the nearest matching discrete [`Rotation`]
    fn snap_rotation(rotation: Rotation) -> Rotation {
        Self::snap(rotation).into()
    }

    /// Snaps a [`Direction`] to the nearest matching discrete [`Direction`]
    fn snap_direction(direction: Direction) -> Direction {
        if let Ok(rotation) = direction.try_into() {
            Self::snap_rotation(rotation).into()
        } else {
            Direction::NEUTRAL
        }
    }

    /// Snaps a [`Vec2`] to the nearest matching discrete [`Direction`], preserving the magnitude
    fn snap_vec2(vec2: Vec2) -> Vec2 {
        if let Ok(rotation) = vec2.try_into() {
            Self::snap_rotation(rotation).into()
        } else {
            Vec2::ZERO
        }
    }
}

/// A 4-way [`DirectionParitioning`], corresponding to the four cardinal directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalQuadrant {
    /// Up
    North,
    /// Right
    East,
    /// Down
    South,
    /// Left
    West,
}

impl DirectionParitioning for CardinalQuadrant {
    fn partitions() -> Vec<Self> {
        use CardinalQuadrant::*;

        vec![North, East, South, West]
    }
}

/// A 4-way [`DirectionParitioning`], corresponding to the four cardinal directions offset by 45 degrees
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetQuadrant {
    /// Up and right
    NorthEast,
    /// Down and right
    SouthEast,
    /// Down and left
    SouthWest,
    /// Up and left
    NorthWest,
}

impl DirectionParitioning for OffsetQuadrant {
    fn partitions() -> Vec<Self> {
        use OffsetQuadrant::*;

        vec![NorthEast, SouthEast, SouthWest, NorthWest]
    }
}

/// A 8-way [`DirectionParitioning`], corresponding to the four cardinal directions and the intermediate values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalOctant {
    /// Up
    North,
    /// Up and right
    NorthEast,
    /// Right
    East,
    /// Down and right
    SouthEast,
    /// Down
    South,
    /// Down and left
    SouthWest,
    /// Left
    West,
    /// Up and left
    NorthWest,
}

impl DirectionParitioning for CardinalOctant {
    fn partitions() -> Vec<Self> {
        use CardinalOctant::*;

        vec![
            North, NorthEast, East, SouthEast, South, SouthWest, West, NorthWest,
        ]
    }
}

/// A 6-way [`DirectionParitioning`], corresponding to the 6 directions of a tip-up hexagon
///
/// For visualization purposes, these hexagons can be tiled in a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum CardinalSextant {
    /// Up
    North,
    /// Up and right
    NorthEast,
    /// Down and right
    SouthEast,
    /// Down
    South,
    /// Down and left
    SouthWest,
    /// Up and left
    NorthWest,
}

impl DirectionParitioning for CardinalSextant {
    fn partitions() -> Vec<Self> {
        use CardinalSextant::*;

        vec![North, NorthEast, SouthEast, South, SouthWest, NorthWest]
    }
}

/// A 6-way [`DirectionParitioning`], corresponding to the 6 directions of a flat-up hexagon
///
/// For visualization purposes, these hexagons can be tiled in a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum OffsetSextant {
    /// Up and right
    NorthEast,
    /// Right
    East,
    /// Down and right
    SouthEast,
    /// Down and left,
    SouthWest,
    /// Left
    West,
    /// Up and left
    NorthWest,
}

impl DirectionParitioning for OffsetSextant {
    fn partitions() -> Vec<Self> {
        use OffsetSextant::*;

        vec![NorthEast, East, SouthEast, SouthWest, West, NorthWest]
    }
}

mod parition_conversions {
    use super::*;

    // Quadrant
    impl From<CardinalQuadrant> for Rotation {
        fn from(quadrant: CardinalQuadrant) -> Rotation {
            match quadrant {
                CardinalQuadrant::North => Rotation::from_degrees(0.0),
                CardinalQuadrant::East => Rotation::from_degrees(90.0),
                CardinalQuadrant::South => Rotation::from_degrees(180.0),
                CardinalQuadrant::West => Rotation::from_degrees(270.0),
            }
        }
    }

    impl From<CardinalQuadrant> for Direction {
        fn from(quadrant: CardinalQuadrant) -> Direction {
            let rotation: Rotation = quadrant.into();
            rotation.into()
        }
    }

    impl From<CardinalQuadrant> for Vec2 {
        fn from(quadrant: CardinalQuadrant) -> Vec2 {
            let rotation: Rotation = quadrant.into();
            rotation.into()
        }
    }

    // Quadrant
    impl From<OffsetQuadrant> for Rotation {
        fn from(quadrant: OffsetQuadrant) -> Rotation {
            match quadrant {
                OffsetQuadrant::NorthEast => Rotation::from_degrees(45.0),
                OffsetQuadrant::SouthEast => Rotation::from_degrees(135.0),
                OffsetQuadrant::SouthWest => Rotation::from_degrees(225.0),
                OffsetQuadrant::NorthWest => Rotation::from_degrees(315.0),
            }
        }
    }

    impl From<OffsetQuadrant> for Direction {
        fn from(quadrant: OffsetQuadrant) -> Direction {
            let rotation: Rotation = quadrant.into();
            rotation.into()
        }
    }

    impl From<OffsetQuadrant> for Vec2 {
        fn from(quadrant: OffsetQuadrant) -> Vec2 {
            let rotation: Rotation = quadrant.into();
            rotation.into()
        }
    }

    // Octant
    impl From<CardinalOctant> for Rotation {
        fn from(octant: CardinalOctant) -> Rotation {
            match octant {
                CardinalOctant::North => Rotation::from_degrees(0.0),
                CardinalOctant::NorthEast => Rotation::from_degrees(45.0),
                CardinalOctant::East => Rotation::from_degrees(90.0),
                CardinalOctant::SouthEast => Rotation::from_degrees(135.0),
                CardinalOctant::South => Rotation::from_degrees(180.0),
                CardinalOctant::SouthWest => Rotation::from_degrees(225.0),
                CardinalOctant::West => Rotation::from_degrees(270.0),
                CardinalOctant::NorthWest => Rotation::from_degrees(315.0),
            }
        }
    }

    impl From<CardinalOctant> for Direction {
        fn from(octant: CardinalOctant) -> Direction {
            let rotation: Rotation = octant.into();
            rotation.into()
        }
    }

    impl From<CardinalOctant> for Vec2 {
        fn from(octant: CardinalOctant) -> Vec2 {
            let rotation: Rotation = octant.into();
            rotation.into()
        }
    }

    // Sextant
    impl From<CardinalSextant> for Rotation {
        fn from(sextant: CardinalSextant) -> Rotation {
            match sextant {
                CardinalSextant::North => Rotation::from_degrees(0.0),
                CardinalSextant::NorthEast => Rotation::from_degrees(60.0),
                CardinalSextant::SouthEast => Rotation::from_degrees(120.0),
                CardinalSextant::South => Rotation::from_degrees(180.0),
                CardinalSextant::SouthWest => Rotation::from_degrees(240.0),
                CardinalSextant::NorthWest => Rotation::from_degrees(300.0),
            }
        }
    }

    impl From<CardinalSextant> for Direction {
        fn from(sextant: CardinalSextant) -> Direction {
            let rotation: Rotation = sextant.into();
            rotation.into()
        }
    }

    impl From<CardinalSextant> for Vec2 {
        fn from(sextant: CardinalSextant) -> Vec2 {
            let rotation: Rotation = sextant.into();
            rotation.into()
        }
    }

    // OffsetSextant
    impl From<OffsetSextant> for Rotation {
        fn from(sextant: OffsetSextant) -> Rotation {
            match sextant {
                OffsetSextant::NorthEast => Rotation::from_degrees(30.0),
                OffsetSextant::East => Rotation::from_degrees(90.0),
                OffsetSextant::SouthEast => Rotation::from_degrees(150.0),
                OffsetSextant::SouthWest => Rotation::from_degrees(210.0),
                OffsetSextant::West => Rotation::from_degrees(270.0),
                OffsetSextant::NorthWest => Rotation::from_degrees(330.0),
            }
        }
    }

    impl From<OffsetSextant> for Direction {
        fn from(sextant: OffsetSextant) -> Direction {
            let rotation: Rotation = sextant.into();
            rotation.into()
        }
    }

    impl From<OffsetSextant> for Vec2 {
        fn from(sextant: OffsetSextant) -> Vec2 {
            let rotation: Rotation = sextant.into();
            rotation.into()
        }
    }
}
