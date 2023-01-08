/*
 * Copyright (c) 2020 Grell, Robin <grellr@hochschule-trier.de>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![deny(clippy::pedantic)]

// Imports from other crates
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
use relm::Widget;

// Modules
mod config;
mod dbus;
mod decoder;
mod keyboard;
mod submitter;
mod user_interface;

/// Gets the region from the locale
/// E.g. if the locale is set to 'en-US', this function returns 'us'
/// It is used as the "language", hence the name
fn get_locale_language() -> String {
    let locale = format!("{}", locale_config::Locale::user_default());
    let locale_language: String = locale.rsplit('-').take(1).collect();
    locale_language.to_lowercase()
}

/// Initiates the logger and starts the main loop
fn main() {
    pretty_env_logger::init();
    user_interface::Win::run(()).unwrap();
}
