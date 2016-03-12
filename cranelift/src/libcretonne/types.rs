
//! Common types for the Cretonne code generator.

use std::fmt::{self, Display, Formatter, Write};

/// The type of an SSA value.
///
/// The `VOID` type is only used for instructions that produce no value. It can't be part of a SIMD
/// vector.
///
/// Basic integer types: `I8`, `I16`, `I32`, and `I64`. These types are sign-agnostic.
///
/// Basic floating point types: `F32` and `F64`. IEEE single and double precision.
///
/// Boolean types: `B1`, `B8`, `B16`, `B32`, and `B64`. These all encode 'true' or 'false'. The
/// larger types use redundant bits.
///
/// SIMD vector types have power-of-two lanes, up to 256. Lanes can be any int/float/bool type.
///
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Type(u8);

/// No type. Used for functions without a return value. Can't be loaded or stored. Can't be part of
/// a SIMD vector.
pub const VOID: Type = Type(0);

/// Integer type with 8 bits.
pub const I8: Type = Type(1);

/// Integer type with 16 bits.
pub const I16: Type = Type(2);

/// Integer type with 32 bits.
pub const I32: Type = Type(3);

/// Integer type with 64 bits.
pub const I64: Type = Type(4);

/// IEEE single precision floating point type.
pub const F32: Type = Type(5);

/// IEEE double precision floating point type.
pub const F64: Type = Type(6);

/// Boolean type. Can't be loaded or stored, but can be used to form SIMD vectors.
pub const B1: Type = Type(7);

/// Boolean type using 8 bits to represent true/false.
pub const B8: Type = Type(8);

/// Boolean type using 16 bits to represent true/false.
pub const B16: Type = Type(9);

/// Boolean type using 32 bits to represent true/false.
pub const B32: Type = Type(10);

/// Boolean type using 64 bits to represent true/false.
pub const B64: Type = Type(11);

impl Type {
    /// Get the lane type of this SIMD vector type.
    /// 
    /// A scalar type is the same as a SIMD vector type with one lane, so it returns itself.
    pub fn lane_type(self) -> Type {
        Type(self.0 & 0x0f)
    }

    /// Get the number of bits in a lane.
    pub fn lane_bits(self) -> u8 {
        match self.lane_type() {
            B1 => 1,
            B8 | I8 => 8,
            B16 | I16 => 16,
            B32 | I32 | F32 => 32,
            B64 | I64 | F64 => 64,
            _ => 0,
        }
    }

    /// Is this the VOID type?
    pub fn is_void(self) -> bool {
        self == VOID
    }

    /// Is this a scalar boolean type?
    pub fn is_bool(self) -> bool {
        match self {
            B1 | B8 | B16 | B32 | B64 => true,
            _ => false,
        }
    }

    /// Is this a scalar integer type?
    pub fn is_int(self) -> bool {
        match self {
            I8 | I16 | I32 | I64 => true,
            _ => false,
        }
    }

    /// Is this a scalar floating point type?
    pub fn is_float(self) -> bool {
        match self {
            F32 | F64 => true,
            _ => false,
        }
    }

    /// Get log2 of the number of lanes in this SIMD vector type.
    ///
    /// All SIMD types have a lane count that is a power of two and no larger than 256, so this
    /// will be a number in the range 0-8.
    ///
    /// A scalar type is the same as a SIMD vector type with one lane, so it return 0.
    pub fn log2_lane_count(self) -> u8 {
        self.0 >> 4
    }

    /// Is this a scalar type? (That is, not a SIMD vector type).
    ///
    /// A scalar type is the same as a SIMD vector type with one lane.
    pub fn is_scalar(self) -> bool {
        self.log2_lane_count() == 0
    }

    /// Get the number of lanes in this SIMD vector type.
    ///
    /// A scalar type is the same as a SIMD vector type with one lane, so it returns 1.
    pub fn lane_count(self) -> u16 {
        1 << self.log2_lane_count()
    }

    /// Get the total number of bits used to represent this type.
    pub fn bits(self) -> u16 {
        self.lane_bits() as u16 * self.lane_count()
    }

    /// Get a SIMD vector type with `n` times more lanes than this one.
    ///
    /// If this is a scalar type, this produces a SIMD type with this as a lane type and `n` lanes.
    ///
    /// If this is already a SIMD vector type, this produces a SIMD vector type with `n *
    /// self.lane_count()` lanes.
    pub fn by(self, n: u16) -> Type {
        debug_assert!(self.lane_bits() > 0,
                      "Can't make SIMD vectors with void lanes.");
        debug_assert!(n.is_power_of_two(),
                      "Number of SIMD lanes must be a power of two");
        let log2_lanes: u32 = n.trailing_zeros();
        let new_type = self.0 as u32 + (log2_lanes << 4);
        assert!(new_type < 0x90, "No more than 256 SIMD lanes supported");
        Type(new_type as u8)
    }

    /// Get a SIMD vector with half the number of lanes.
    pub fn half_vector(self) -> Type {
        assert!(!self.is_scalar(), "Expecting a proper SIMD vector type.");
        Type(self.0 - 0x10)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.is_void() {
            write!(f, "void")
        } else if self.is_bool() {
            write!(f, "b{}", self.lane_bits())
        } else if self.is_int() {
            write!(f, "i{}", self.lane_bits())
        } else if self.is_float() {
            write!(f, "f{}", self.lane_bits())
        } else if !self.is_scalar() {
            write!(f, "{}x{}", self.lane_type(), self.lane_count())
        } else {
            panic!("Invalid Type(0x{:x})", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_scalars() {
        assert_eq!(VOID, VOID.lane_type());
        assert_eq!(0, VOID.bits());
        assert_eq!(B1, B1.lane_type());
        assert_eq!(B8, B8.lane_type());
        assert_eq!(B16, B16.lane_type());
        assert_eq!(B32, B32.lane_type());
        assert_eq!(B64, B64.lane_type());
        assert_eq!(I8, I8.lane_type());
        assert_eq!(I16, I16.lane_type());
        assert_eq!(I32, I32.lane_type());
        assert_eq!(I64, I64.lane_type());
        assert_eq!(F32, F32.lane_type());
        assert_eq!(F64, F64.lane_type());

        assert_eq!(VOID.lane_bits(), 0);
        assert_eq!(B1.lane_bits(), 1);
        assert_eq!(B8.lane_bits(), 8);
        assert_eq!(B16.lane_bits(), 16);
        assert_eq!(B32.lane_bits(), 32);
        assert_eq!(B64.lane_bits(), 64);
        assert_eq!(I8.lane_bits(), 8);
        assert_eq!(I16.lane_bits(), 16);
        assert_eq!(I32.lane_bits(), 32);
        assert_eq!(I64.lane_bits(), 64);
        assert_eq!(F32.lane_bits(), 32);
        assert_eq!(F64.lane_bits(), 64);
    }

    #[test]
    fn vectors() {
        let big = F64.by(256);
        assert_eq!(big.lane_bits(), 64);
        assert_eq!(big.lane_count(), 256);
        assert_eq!(big.bits(), 64 * 256);

        assert_eq!(format!("{}", big.half_vector()), "f64x128");
        assert_eq!(format!("{}", B1.by(2).half_vector()), "b1");
    }

    #[test]
    fn format_scalars() {
        assert_eq!(format!("{}", VOID), "void");
        assert_eq!(format!("{}", B1), "b1");
        assert_eq!(format!("{}", B8), "b8");
        assert_eq!(format!("{}", B16), "b16");
        assert_eq!(format!("{}", B32), "b32");
        assert_eq!(format!("{}", B64), "b64");
        assert_eq!(format!("{}", I8), "i8");
        assert_eq!(format!("{}", I16), "i16");
        assert_eq!(format!("{}", I32), "i32");
        assert_eq!(format!("{}", I64), "i64");
        assert_eq!(format!("{}", F32), "f32");
        assert_eq!(format!("{}", F64), "f64");
    }

    #[test]
    fn format_vectors() {
        assert_eq!(format!("{}", B1.by(8)), "b1x8");
        assert_eq!(format!("{}", B8.by(1)), "b8");
        assert_eq!(format!("{}", B16.by(256)), "b16x256");
        assert_eq!(format!("{}", B32.by(4).by(2)), "b32x8");
        assert_eq!(format!("{}", B64.by(8)), "b64x8");
        assert_eq!(format!("{}", I8.by(64)), "i8x64");
        assert_eq!(format!("{}", F64.by(2)), "f64x2");
    }
}
