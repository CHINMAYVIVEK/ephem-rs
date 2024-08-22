use chrono::DateTime;
use chrono::Datelike;
use chrono::Timelike;
use chrono::Utc;
use lib_sys::swe_get_planet_name;
use lib_sys::{
    swe_calc_ut, swe_close, swe_get_current_file_data, swe_get_library_path, swe_julday,
    swe_set_ephe_path, swe_set_jpl_file, swe_version, SE_GREG_CAL,
};
use std::env;
use std::fmt;
use std::ptr::addr_of;
use std::str;
use std::sync::Once;
use std::{path::Path, ptr::null_mut};

const MAXCH: usize = 256;
static SET_EPHE_PATH: Once = Once::new();
static mut EPHE_PATH: String = String::new();
static CLOSED: Once = Once::new();

// macro for getting the name of the current function.
// creates a new function f inside of the current function,
// and gets its type_name, and then strips the last three
// chars (which would be "::f") to get the module path and
// name of the current function.
// Adapted from https://stackoverflow.com/questions/38088067/equivalent-of-func-or-function-in-rust
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

fn assert_ephe_ready(fn_name: &str) {
    assert!(
        !CLOSED.is_completed(),
        "Invoked libswe function {} after closing the ephemeris files.",
        fn_name
    );
    assert!(
        SET_EPHE_PATH.is_completed(),
        "Invoked libswe function {} before calling set_ephe_path.",
        fn_name
    );
}

pub struct FileData {
    pub filepath: String,
    pub start_date: f64,
    pub end_date: f64,
    pub ephemeris_num: i32,
}

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

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Flag {
    JPLEphemeris = 1,
    SwissEphemeris = 2,
    MoshierEphemeris = 4,
    HeliocentricPos = 8,
    TruePos = 16,
    NoPrecession = 32,
    NoNutation = 64,
    HighPrecSpeed = 256,
    NoGravDeflection = 512,
    NoAnnualAbberation = 1024,
    AstrometricPos = 1536,
    EquatorialPos = 2048,
    CartesianCoords = 4096,
    Radians = 8192,
    BarycentricPos = 16384,
    TopocentricPos = 32 * 1024,
    Sideral = 64 * 1024,
    ICRS = 128 * 1024,
    JPLHorizons = 256 * 1024,
    JPLHorizonsApprox = 512 * 1024,
    CenterOfBody = 1024 * 1024,
}

pub struct BodyResult {
    pub pos: Vec<f64>,
    pub vel: Vec<f64>,
}

pub struct EclipticAndNutationResult {
    pub ecliptic_true_obliquity: f64,
    pub ecliptic_mean_obliquity: f64,
    pub nutation_lng: f64,
    pub nutation_obliquity: f64,
}

pub enum CalculationResult {
    Body(BodyResult),
    EclipticAndNutation(EclipticAndNutationResult),
}

#[derive(Debug)]
pub struct CalculationError {
    code: i32,
    msg: String,
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CalculationError {{ code: {} message: {} }}",
            self.code, self.msg
        )
    }
}

pub fn set_ephe_path(path: Option<&str>) {
    assert!(!CLOSED.is_completed());
    SET_EPHE_PATH.call_once(|| {
        let null = null_mut();
        let env_ephe_path = env::var("SE_EPHE_PATH").ok();
        match env_ephe_path {
            Some(_) => unsafe { swe_set_ephe_path(null) },
            None => match path {
                Some(path_str) => {
                    assert!(path_str.len() < MAXCH);

                    let path_p = Path::new(path_str);
                    assert!(path_p.is_dir());

                    let mut mpath = path_str.to_owned();
                    unsafe {
                        swe_set_ephe_path(mpath.as_mut_ptr() as *mut i8);
                        EPHE_PATH = mpath;
                    }
                }
                None => unsafe { swe_set_ephe_path(null) },
            },
        }
    })
}

pub fn close() {
    CLOSED.call_once(|| unsafe { swe_close() })
}

pub fn get_ephe_path() -> &'static str {
    unsafe {addr_of!(EPHE_PATH).as_ref().unwrap()}
}

pub fn set_jpl_file(filename: &str) {
    assert_ephe_ready(function!());

    let env_ephe_path = env::var("SE_EPHE_PATH").ok();
    assert!(filename.len() < MAXCH);
    let epath = get_ephe_path();

    let path = match env_ephe_path {
        Some(path_str) => Path::new(&path_str).join(filename),
        None =>  Path::new(epath).join(filename) ,

    };
    assert!(path.is_file());
    let mut mfilename = filename.to_owned();
    unsafe {
        swe_set_jpl_file(mfilename.as_mut_ptr() as *mut i8);
    }
}

pub fn version() -> String {
    assert_ephe_ready(function!());
    let mut swe_vers_i: [u8; MAXCH] = [0; MAXCH];
    unsafe {
        swe_version(swe_vers_i.as_mut_ptr() as *mut i8);
    }
    String::from(str::from_utf8(&swe_vers_i).unwrap())
}

pub fn get_library_path() -> String {
    assert_ephe_ready(function!());
    let mut swe_lp_i: [u8; MAXCH] = [0; MAXCH];
    unsafe {
        swe_get_library_path(swe_lp_i.as_mut_ptr() as *mut i8);
    }
    String::from(str::from_utf8(&swe_lp_i).unwrap())
}

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
    let term = b'\0';
    while unsafe { *fp_p } != term {
        unsafe {
            let i = *fp_p;
            let i_slice = &[i as u8];
            let s = str::from_utf8(i_slice).unwrap();
            filepath.push_str(s);
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

pub fn calc_ut(
    julian_day_ut: f64,
    body: Body,
    flag_set: &[Flag],
) -> Result<CalculationResult, CalculationError> {
    let mut flags: i32 = 0;
    for f in flag_set.iter() {
        flags = flags | *f as i32;
    }
    let mut results: [f64; 6] = [0.0; 6];
    let mut error_i: [u8; MAXCH] = [0; MAXCH];
    let code = unsafe {
        swe_calc_ut(
            julian_day_ut,
            body as i32,
            flags,
            &mut results as *mut f64,
            error_i.as_mut_ptr() as *mut i8,
        )
    };
    let msg = String::from(str::from_utf8(&error_i).unwrap());
    if code < 0 {
        Err(CalculationError { code, msg })
    } else {
        match body {
            Body::EclipticNutation => Ok(CalculationResult::EclipticAndNutation(
                EclipticAndNutationResult {
                    ecliptic_true_obliquity: results[0],
                    ecliptic_mean_obliquity: results[1],
                    nutation_lng: results[2],
                    nutation_obliquity: results[3],
                },
            )),
            _ => Ok(CalculationResult::Body(BodyResult {
                pos: vec![results[0], results[1], results[2]],
                vel: vec![results[3], results[4], results[5]],
            })),
        }
    }
}

pub fn julday(dt: DateTime<Utc>) -> f64 {
    // NaiveDateTime because julday assumes UTC.
    unsafe {
        swe_julday(
            dt.year(),
            dt.month() as i32,
            dt.day() as i32,
            dt.hour().into(),
            SE_GREG_CAL as i32,
        )
    }
}

pub fn get_planet_name(body: Body) -> String {
    let mut name: [u8; MAXCH] = [0; MAXCH];

    unsafe { swe_get_planet_name(body as i32, name.as_mut_ptr() as *mut i8) };
    String::from_utf8(Vec::from(name)).unwrap()
}
