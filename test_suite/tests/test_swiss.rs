/*  ephem-rs | Rust bindings for lib-swiss, the Swiss Ephemeris C library.
 *  Copyright (c) 2024 Chinmay Vivek. All rights reserved.

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

extern crate lib_swiss;
use lib_sys::*;
use std::convert::TryInto;

fn main() {
    unsafe {
        let null: *mut i8 = std::ptr::null_mut();
        let iflag: i32 = SEFLG_TROPICAL.try_into().unwrap();
        swe_set_ephe_path(null);
        let gregorian_calendar_flag: i32 = SE_GREG_CAL.try_into().unwrap();
        let julian_day_ut = swe_julday(1991, 10, 13, 20.0, gregorian_calendar_flag);
        println!("Planet\tlon\tlat\tdist");
        for body in SE_SUN..SE_CHIRON {
            let mut coordinates: [f64; 6] = [0.0; 6];
            let mut name: [u8; 64] = [0; 64];
            let mut error_message: [u8; 256] = [0; 256];
            if body == SE_EARTH {
                continue;
            }
            let body_signed: i32 = body.try_into().unwrap();
            let return_flag = swe_calc_ut(
                julian_day_ut,
                body_signed,
                iflag,
                coordinates.as_mut_ptr(),
                error_message.as_mut_ptr() as *mut i8,
            );
            if return_flag < 0 {
                let error_vec: Vec<u8> = error_message.clone().as_ref().into();
                let error_string = String::from_utf8_unchecked(error_vec);
                eprintln!("Error: {}", error_string);
            } else {
                swe_get_planet_name(body_signed, name.as_mut_ptr() as *mut i8);

                println!(
                    "{}\t{}\t{}\t{}",
                    String::from_utf8_unchecked(name.clone().as_ref().into()),
                    coordinates[0],
                    coordinates[1],
                    coordinates[2]
                );
            }
        }
        swe_close();
    }
}