# Changes

## 0.2.2 (2024-10-14)
* The "(nonuniform)" command line warning can be disabled by `-n`.
* Avoids allocation in the internal function `Picker::pick_indexes()`.
* `Picker` can be reconfigured with a new `Config`.

## 0.2.1 (2024-10-12)
* Changed the "(unfair)" command line warning to "(nonuniform)".
* Added `Picker::write_to()` function that writes to a provided slice, avoiding unnecessary allocation.
* Fixed the configuration checker to avoid possible dead loop of the picker and the divide-by-zero issue in the probability calculater. `Picker::table_len()` now excludes impossible choices (p = 0).
* Added optional `serde-config` feature, not turned on by default.
* The character ':' is not treated like ';' during table input.
* Other small fixes.

## 0.2.0 (2024-10-09)
* Initial Rust version
* Based on `rand` crate, allows switching from `rand::rngs::OsRng` to `rand::rngs::ThreadRng` with `-f` option.
* Multi-thread probability calculater
* Changed the behavior of the calculater on repetitive mode

## 0.1 (2022-04-18)
* C++ version without GUI
* Based on `std::random_device`
* Table items now have only one weight value, instead of 12 values for months in a year
* Added repetitive mode
* Added probability tester and calculater

## 0.0 (2017-05-13)
* VB.NET version with WinForms GUI (not released)
* Based on `VBMath.Rnd`
* the author's oldest applet being kept
