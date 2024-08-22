//! `ephem-rs` provides Rust bindings for the Swiss Ephemeris, allowing for high-precision
//! astrological and astronomical calculations. It includes tools to handle ephemeris data,
//! planetary positions, and other celestial calculations.
//!
//! This module allows you to set the ephemeris path, perform calculations, and manage ephemeris
//! files, making it ideal for building applications in astrology or astronomy.

use chrono::{DateTime, Datelike, Timelike, Utc};
use lib_sys::{
    swe_calc_ut, swe_close, swe_get_current_file_data, swe_get_library_path, swe_get_planet_name,
    swe_julday, swe_set_ephe_path, swe_set_jpl_file, swe_version, SE_GREG_CAL,
};
use std::{env, fmt, path::Path, ptr::addr_of, ptr::null_mut, str, sync::Once};

/// Maximum string length used in ephemeris path and other string-based operations.
const MAXCH: usize = 256;

/// Singleton pattern for setting the ephemeris path.
static SET_EPHE_PATH: Once = Once::new();
/// Stores the ephemeris path after it has been set.
static mut EPHE_PATH: String = String::new();
/// Singleton pattern for closing the Swiss Ephemeris.
static CLOSED: Once = Once::new();

/// Macro to get the name of the current function.
///
/// This is helpful for debugging and error messages.
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

/// Ensures the Swiss Ephemeris is ready before invoking any functions.
///
/// This function asserts that the ephemeris path has been set and that the ephemeris files are not closed.
fn assert_ephe_ready(fn_name: &str) {
    assert!(
        !CLOSED.is_completed(),
        "Attempted to call `{}` after the ephemeris files were closed.",
        fn_name
    );
    assert!(
        SET_EPHE_PATH.is_completed(),
        "Attempted to call `{}` before setting the ephemeris path.",
        fn_name
    );
}

/// Stores information about the current ephemeris file.
pub struct FileData {
    /// Path to the ephemeris file.
    pub filepath: String,
    /// Start date of the ephemeris file.
    pub start_date: f64,
    /// End date of the ephemeris file.
    pub end_date: f64,
    /// Ephemeris number (denum) used.
    pub ephemeris_num: i32,
}

/// Represents celestial bodies that can be used in calculations.
#[repr(i32)]
#[derive(PartialEq, Copy, Clone)]
pub enum Body {
    EclipticNutation = -1,
    Sun = 0,
    Moon = 1,
    Mercury = 2,
    Venus = 3,
    Mars = 4,
    Jupiter = 5,
    Saturn = 6,
    Uranus = 7,
    Neptune = 8,
    Pluto = 9,
    MeanNode = 10,
    TrueNode = 11,
    MeanLunarApogee = 12,
    OsculatingLunarApogee = 13,
    Earth = 14,
    Chiron = 15,
    Pholus = 16,
    Ceres = 17,
    Pallas = 18,
    Juno = 19,
    Vesta = 20,
}

/// Calculation flags that define the precision and output format for celestial body positions.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Flag {
    JPLEphemeris = 1,
    SwissEphemeris = 2,
    MoshierEphemeris = 4,
    HeliocentricPos = 8,
    TruePos = 16,
    HighPrecSpeed = 256,
    CartesianCoords = 4096,
    BarycentricPos = 16384,
}

/// Result for a celestial body calculation, including both position and velocity data.
pub struct BodyResult {
    /// Position in space (x, y, z) of the celestial body.
    pub pos: Vec<f64>,
    /// Velocity in space (vx, vy, vz) of the celestial body.
    pub vel: Vec<f64>,
}

/// Result for an ecliptic and nutation calculation.
pub struct EclipticAndNutationResult {
    pub ecliptic_true_obliquity: f64,
    pub ecliptic_mean_obliquity: f64,
    pub nutation_lng: f64,
    pub nutation_obliquity: f64,
}

/// Wrapper enum for calculation results.
pub enum CalculationResult {
    Body(BodyResult),
    EclipticAndNutation(EclipticAndNutationResult),
}

/// Represents an error occurring during celestial calculations.
#[derive(Debug)]
pub struct CalculationError {
    /// Error code returned by the Swiss Ephemeris library.
    code: i32,
    /// Error message returned by the Swiss Ephemeris library.
    msg: String,
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CalculationError {{ code: {}, message: {} }}",
            self.code, self.msg
        )
    }
}

/// Sets the ephemeris path for the Swiss Ephemeris.
///
/// This function should be called before any ephemeris calculations.
/// The path can be set manually or automatically through the `SE_EPHE_PATH` environment variable.
pub fn set_ephe_path(path: Option<&str>) {
    assert!(!CLOSED.is_completed());
    SET_EPHE_PATH.call_once(|| {
        let env_ephe_path = env::var("SE_EPHE_PATH").ok();
        match env_ephe_path {
            Some(_) => unsafe { swe_set_ephe_path(null_mut()) },
            None => match path {
                Some(path_str) => {
                    let path_p = Path::new(path_str);
                    assert!(path_p.is_dir());
                    let mut mpath = path_str.to_owned();
                    unsafe {
                        swe_set_ephe_path(mpath.as_mut_ptr() as *mut i8);
                        EPHE_PATH = mpath;
                    }
                }
                None => unsafe { swe_set_ephe_path(null_mut()) },
            },
        }
    });
}

/// Closes the Swiss Ephemeris files.
///
/// Once this function is called, no further calculations can be performed until the ephemeris path is reset.
pub fn close() {
    CLOSED.call_once(|| unsafe { swe_close() })
}

/// Retrieves the current ephemeris path.
///
/// This function returns the ephemeris path set by `set_ephe_path`.
pub fn get_ephe_path() -> &'static str {
    unsafe { addr_of!(EPHE_PATH).as_ref().unwrap() }
}

/// Sets the JPL ephemeris file for use in calculations.
///
/// The file should exist at the provided path or in the ephemeris directory.
pub fn set_jpl_file(filename: &str) {
    assert_ephe_ready(function!());

    let epath = get_ephe_path();
    let path = Path::new(epath).join(filename);
    assert!(path.is_file());
    let mut mfilename = filename.to_owned();
    unsafe {
        swe_set_jpl_file(mfilename.as_mut_ptr() as *mut i8);
    }
}

/// Retrieves the Swiss Ephemeris version as a string.
pub fn version() -> String {
    assert_ephe_ready(function!());
    let mut swe_vers_i: [u8; MAXCH] = [0; MAXCH];
    unsafe {
        swe_version(swe_vers_i.as_mut_ptr() as *mut i8);
    }
    String::from(str::from_utf8(&swe_vers_i).unwrap())
}

/// Retrieves the path to the currently loaded Swiss Ephemeris library.
pub fn get_library_path() -> String {
    assert_ephe_ready(function!());
    let mut swe_lp_i: [u8; MAXCH] = [0; MAXCH];
    unsafe {
        swe_get_library_path(swe_lp_i.as_mut_ptr() as *mut i8);
    }
    String::from(str::from_utf8(&swe_lp_i).unwrap())
}

/// Retrieves data about the currently loaded ephemeris file.
pub fn get_current_file_data(ifno: i32) -> FileData {
    assert_ephe_ready(function!());
    let mut tfstart: f64 = 0.0;
    let mut tfend: f64 = 0.0;
    let mut denum: i32 = 0;
    let mut filepath = String::with_capacity(MAXCH);

    let fp_i = unsafe {
        swe_get_current_file_data(
            ifno,
            &mut tfstart as *mut f64,
            &mut tfend as *mut f64,
            &mut denum as *mut i32,
        )
    } as *const u8;
    let mut fp_p = fp_i;
    while unsafe { *fp_p } != b'\0' {
        unsafe {
            filepath.push(*fp_p as char);
            fp_p = fp_p.add(1);
        }
    }

    FileData {
        filepath,
        start_date: tfstart,
        end_date: tfend,
        ephemeris_num: denum,
    }
}

/// Calculates celestial body positions based on Universal Time.
///
/// This function takes a Julian day in Universal Time (UT), a celestial `Body`, and a set of `Flag`s
/// to determine the precision and format of the output.
///
/// It returns a `BodyResult` containing the calculated position and velocity of the celestial body.
pub fn calculate_ut(jd: f64, body: Body, flags: Flag) -> Result<BodyResult, CalculationError> {
    assert_ephe_ready(function!());

    let mut rsmi = vec![0f64; 6];
    let mut serr = vec![0u8; MAXCH];
    let swe_err = unsafe {
        swe_calc_ut(
            jd,
            body as i32,
            flags as i32,
            rsmi.as_mut_ptr() as *mut f64,
            serr.as_mut_ptr() as *mut i8,
        )
    };
    let err_message = str::from_utf8(&serr)
        .unwrap()
        .trim_end_matches(char::from(0));

    match swe_err {
        0 => Ok(BodyResult {
            pos: rsmi[..3].to_vec(),
            vel: rsmi[3..6].to_vec(),
        }),
        _ => Err(CalculationError {
            code: swe_err,
            msg: err_message.to_string(),
        }),
    }
}

/// Converts a given Universal Time (UTC) date into Julian Day.
///
/// Julian Day is the continuous count of days since the beginning of the Julian Period.
/// This function uses the Swiss Ephemeris to convert a `DateTime<Utc>` into a Julian Day.
pub fn utc_to_julian_day(utc: DateTime<Utc>) -> f64 {
    assert_ephe_ready(function!());

    unsafe {
        swe_julday(
            utc.year(),
            utc.month() as i32,
            utc.day() as i32,
            utc.hour() as f64
                + utc.minute() as f64 / 60.0
                + utc.second() as f64 / 3600.0
                + utc.timestamp_subsec_micros() as f64 / 3600_000_000.0,
            SE_GREG_CAL as i32,
        )
    }
}

/// Retrieves the name of a celestial body based on its ID.
///
/// This function is useful for verifying body identification or for display purposes.
pub fn get_planet_name(body: Body) -> String {
    assert_ephe_ready(function!());

    let mut swe_name_i: [u8; MAXCH] = [0; MAXCH];
    unsafe {
        swe_get_planet_name(body as i32, swe_name_i.as_mut_ptr() as *mut i8);
    }
    String::from(str::from_utf8(&swe_name_i).unwrap())
}
