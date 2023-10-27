pub use c_kzg::{BYTES_PER_G1_POINT, BYTES_PER_G2_POINT};
use core::fmt::Display;
use core::slice;
use core::str;
use derive_more::{AsMut, AsRef, Deref, DerefMut};

/// Number of G1 Points.
pub const NUM_G1_POINTS: usize = 4096;

/// Number of G2 Points.
pub const NUM_G2_POINTS: usize = 65;

/// A newtype over list of G1 point from kzg trusted setup.
#[derive(Debug, Clone, PartialEq, AsRef, AsMut, Deref, DerefMut)]
#[repr(transparent)]
pub struct G1Points(pub [[u8; BYTES_PER_G1_POINT]; NUM_G1_POINTS]);

impl Default for G1Points {
    fn default() -> Self {
        Self([[0; BYTES_PER_G1_POINT]; NUM_G1_POINTS])
    }
}

/// A newtype over list of G2 point from kzg trusted setup.
#[derive(Debug, Clone, Eq, PartialEq, AsRef, AsMut, Deref, DerefMut)]
#[repr(transparent)]
pub struct G2Points(pub [[u8; BYTES_PER_G2_POINT]; NUM_G2_POINTS]);

impl Default for G2Points {
    fn default() -> Self {
        Self([[0; BYTES_PER_G2_POINT]; NUM_G2_POINTS])
    }
}

const POINTS: &(G1Points, G2Points) =
    &match parse_kzg_trusted_setup(include_str!("trusted_setup.txt")) {
        Ok(x) => x,
        Err(_) => panic!("failed to parse kzg trusted setup"),
    };

/// Default G1 points.
pub const G1_POINTS: &G1Points = &POINTS.0;

/// Default G2 points.
pub const G2_POINTS: &G2Points = &POINTS.1;

macro_rules! tri {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Err(e),
        }
    };
}

macro_rules! unwrap_opt {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => return Err(KzgError::FileFormatError),
        }
    };
}

/// Parses the contents of a KZG trusted setup file into a list of G1 and G2 points.
///
/// These can then be used to create a KZG settings object with
/// [`KzgSettings::load_trusted_setup`](c_kzg::KzgSettings::load_trusted_setup).
pub const fn parse_kzg_trusted_setup(
    trusted_setup: &str,
) -> Result<(G1Points, G2Points), KzgError> {
    let mut contents = trusted_setup;

    macro_rules! next_line {
        () => {{
            let (rest, sl) = next_pat(contents, b'\n');
            contents = rest;
            sl
        }};
    }

    let n_g1 = unwrap_opt!(next_line!());
    let n_g1 = tri!(parse_str(n_g1));
    let n_g2 = unwrap_opt!(next_line!());
    let n_g2 = tri!(parse_str(n_g2));

    if n_g1 != NUM_G1_POINTS {
        return Err(KzgError::MismatchedNumberOfPoints);
    }
    if n_g2 != NUM_G2_POINTS {
        return Err(KzgError::MismatchedNumberOfPoints);
    }

    let mut g1_points = [[0; BYTES_PER_G1_POINT]; NUM_G1_POINTS];
    let mut i = 0;
    while i < NUM_G1_POINTS {
        let line = unwrap_opt!(next_line!());
        g1_points[i] = tri!(hex_decode(line));
        i += 1;
    }

    let mut g2_points = [[0; BYTES_PER_G2_POINT]; NUM_G2_POINTS];
    let mut i = 0;
    while i < NUM_G2_POINTS {
        let line = unwrap_opt!(next_line!());
        g2_points[i] = tri!(hex_decode(line));
        i += 1;
    }

    if next_line!().is_some() {
        return Err(KzgError::FileFormatError);
    }
    let _ = contents;

    Ok((G1Points(g1_points), G2Points(g2_points)))
}

const fn next_pat(s: &str, pat: u8) -> (&str, Option<&str>) {
    assert!(pat.is_ascii());

    let mut bytes = s.as_bytes();
    while let [x, rest @ ..] = bytes {
        if *x == pat {
            unsafe {
                let rest = str::from_utf8_unchecked(rest);
                let sl = slice::from_raw_parts(s.as_ptr(), s.len() - rest.len() - 1);
                return (rest, Some(str::from_utf8_unchecked(sl)));
            }
        }
        bytes = rest;
    }
    (s, None)
}

const fn parse_str(s: &str) -> Result<usize, KzgError> {
    let mut i = 0;
    let mut bytes = s.as_bytes();
    if bytes.is_empty() {
        return Err(KzgError::ParseError);
    }
    while let [x, rest @ ..] = bytes {
        if !matches!(x, b'0'..=b'9') {
            return Err(KzgError::ParseError);
        }
        i = i * 10 + (*x - b'0') as usize;
        bytes = rest;
    }
    Ok(i)
}

const fn hex_decode<const N: usize>(s: &str) -> Result<[u8; N], KzgError> {
    match crate::hex::const_decode_to_array(s.as_bytes()) {
        Ok(x) => Ok(x),
        Err(_) => Err(KzgError::ParseError),
    }
}

#[derive(Debug)]
pub enum KzgError {
    /// Failed to get current directory.
    FailedCurrentDirectory,
    /// The specified path does not exist.
    PathNotExists,
    /// Problems related to I/O.
    IOError,
    /// Not a valid file.
    NotValidFile,
    /// File is not properly formatted.
    FileFormatError,
    /// Not able to parse to usize.
    ParseError,
    /// Number of points does not match what is expected.
    MismatchedNumberOfPoints,
}

impl Display for KzgError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            KzgError::FailedCurrentDirectory => write!(f, "Failed to get current directory"),
            KzgError::PathNotExists => write!(f, "The specified path does not exist"),
            KzgError::IOError => write!(f, "Problems related to I/O"),
            KzgError::NotValidFile => write!(f, "Not a valid file"),
            KzgError::FileFormatError => write!(f, "File is not properly formatted"),
            KzgError::ParseError => write!(f, "Not able to parse to usize"),
            KzgError::MismatchedNumberOfPoints => {
                write!(f, "Number of points does not match what is expected")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for KzgError {}
