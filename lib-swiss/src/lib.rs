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

/// The `swiss_ephm` module provides utilities to interface with the Swiss Ephemeris,
/// allowing for astrological and astronomical calculations based on high-precision data.
pub mod swiss_ephm;

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the `set_ephe_path` function in the `swiss_ephm` module works correctly.
    ///
    /// This test sets the ephemeris path to `None`, which should result in the default
    /// path being used (often the current directory or a predefined location).
    #[test]
    fn test_set_ephe_path_with_none() {
        // Setting the ephemeris path to `None`. This should configure the Swiss Ephemeris
        // to use the default ephemeris path.
        swiss_ephm::set_ephe_path(None);
    }
}
