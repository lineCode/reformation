//! Parsing via regular expressions using format syntax
//!
//! Deriving trait `Reformation` will also implement
//! trait `FromStr`, with `Err=Box<Error>`
//!
//! Derive will require attribute reformation to specify format string,
//! which will be treated as format string -> regular expression string
//!
//! Types implementing `Reformation` by default:
//!
//! + signed integers: `i8` `i16` `i32` `i64` `i128` `isize`
//! + unsigned integers: `u8` `u16` `u32` `u64` `u128` `usize`
//! + floats: `f32` `f64`
//! + `String`
//!
//! ```
//! use reformation::Reformation;
//!
//! #[derive(Reformation, Debug)]
//! #[reformation(r"{year}-{month}-{day} {hour}:{minute}")]
//! struct Date{
//!     year: u16,
//!     month: u8,
//!     day: u8,
//!     hour: u8,
//!     minute: u8,
//! }
//!
//! fn main(){
//!     let date: Date = "2018-12-22 20:23".parse().unwrap();
//!
//!     assert_eq!(date.year, 2018);
//!     assert_eq!(date.month, 12);
//!     assert_eq!(date.day, 22);
//!     assert_eq!(date.hour, 20);
//!     assert_eq!(date.minute, 23);
//! }
//! ```
//!
//! Format string behaves as regular expression, so special symbols needs to be escaped.
//! Also they can be used for more flexible format strings.
//! AVOID capture groups, since they would mess up with indexing of capture group
//! generated by macro. use non-capturing groups `r"(?:)"` instead.
//!
//! ```
//! use reformation::Reformation;
//!
//! // '{' is special symbol in both format and regex syntax, so it must be escaped twice.
//! // Say hello to good old escape hell. Good thing its only one.
//! #[derive(Reformation, Debug)]
//! #[reformation(r"Vec\{{{x},\s*{y},\s*{z}\}}")]
//! struct Vec{
//!     x: f64,
//!     y: f64,
//!     z: f64,
//! }
//!
//! fn main(){
//!     // spaces between coordinates does not matter, since any amount of spaces
//!     // matches to r"\s*"
//!     let v: Vec = "Vec{-0.4,1e-3,   2e-3}".parse().unwrap();
//!
//!     assert_eq!(v.x, -0.4);
//!     assert_eq!(v.y, 0.001);
//!     assert_eq!(v.z, 0.002);
//! }
//! ```


pub use reformation_derive::*;

use std::fmt;
use std::error::Error;
pub use regex::{Regex, Captures};
pub use lazy_static::lazy_static;

#[derive(Debug)]
pub struct NoRegexMatch{
    pub format: &'static str,
    pub request: String,
}

impl std::error::Error for NoRegexMatch{}
impl fmt::Display for NoRegexMatch{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        write!(f, "String {:?} does not match format r{:?}", self.format, self.request)
    }
}

pub trait Reformation: Sized{
    /// regular expression for matching this struct
    fn regex_str()->&'static str;

    // Can be calculated from regex_str, but this method guaranties no
    // recalculations each parse and does not make by hand implementing
    // much more difficult, although MORE error prone.
    /// number of used capture groups.
    fn captures_count()->usize;

    /// create instance of function from captures with given offset
    fn from_captures(c: &Captures, offset: usize)->Result<Self, Box<Error>>;
}


macro_rules! group_impl_parse_primitive{
    ($re: expr, $($name: ty),*) => {
        $(group_impl_parse_primitive!{@single $re, $name})*
    };

    (@single $re: expr, $name: ty) => {
        impl Reformation for $name{
            fn regex_str()->&'static str{
                $re
            }

            fn captures_count()->usize{
                1
            }

            fn from_captures(c: &Captures, offset: usize)->Result<Self, Box<std::error::Error>>{
                let res = c.get(offset).unwrap().as_str().parse::<$name>()?;
                Ok(res)
            }
        }
    };
}

group_impl_parse_primitive!{r"(\d+)", u8, u16, u32, u64, u128, usize}
group_impl_parse_primitive!{r"([\+-]?\d+)", i8, i16, i32, i64, i128, isize}
group_impl_parse_primitive!{r"((?:[\+-]?\d+(?:.\d*)?|.\d+)(?:[eE][\+-]?\d+)?)", f32, f64}
group_impl_parse_primitive!{r"(.*)", String}


/// Creates function for parsing tuple of values from
/// strings corresponding to given template.
///
/// usage: `create_parse_fn!{function_name, re, types..}`
///
/// where:
/// + function_name -- Name of function to be created.
/// + re -- Format string for matching arguments. Format string is regular
///     expression, preprocessed by macro, and rules simular to regexprs
///     applies to it. In order to macro work properly usage of capture
///     groups should be avoided. Non capturing `(:?groups)` are fine.
/// + types.. -- sequence of types expected as function output. Each type must
///     implement trait ```ParsePrimitive```. Default implementors:
///     + unsigned integers: u8, u16, u32, u64, u128, usize
///     + signed integers: i8, i16, i16, i64, i128, isize
///     + floating point numbers: f32, f64,
///     + String
///
/// ```
/// use reformation::create_parse_fn;
///
/// // "\(" and "\)" are escaped, since they are special characters in
/// // regular expression syntax
/// create_parse_fn!{parse_vec, r"^Vec\({}, {}\)$", i32, i32}
///
///
/// fn main(){
///     let (x, y) = parse_vec("Vec(-16, 8)").unwrap();
///     assert_eq!(x, -16i32);
///     assert_eq!(y, 8i32);
/// }
/// ```
///
/// ```
/// use reformation::create_parse_fn;
///
/// // "\{{" and "\}}" is the way to use symbols {, } in format string,
/// // since { is special symbol for formatting and also special symbol for
/// // regular expressions, so it needs to be escaped twice.
/// create_parse_fn!{parse_curly, r"^Vec\{{{}, {}, {}\}}$", i32, i32, usize}
///
/// fn main(){
///     let (x, y, z) = parse_curly("Vec{-16, 8, 800}").unwrap();
///     assert_eq!(x, -16i32);
///     assert_eq!(y, 8i32);
///     assert_eq!(z, 800usize);
/// }
///
/// ```
///
/// You can use features of regular expression
/// ```
/// use reformation::create_parse_fn;
///
/// // Ignore spaces between coordinates
/// create_parse_fn!{parse_vec, r"^Vec\({}, {}\)$", f32, f32}
///
/// fn main(){
///     let (x, y) = parse_vec("Vec(-16, 8e-3)").unwrap();
///     assert_eq!(x, -16.0);
///     assert_eq!(y, 0.008);
/// }
/// ```
#[macro_export]
macro_rules! create_parse_fn{
    ($name: ident, $re: expr, $($res: ty),*) => {
        fn $name(s: &str)->Result<($($res),*), Box<std::error::Error>>{
            create_parse_fn!(@body s, $re, $($res),*)
        }
    };
    (@body $str: expr, $re: expr, $($res: ty),*) => {
        {
            type OkType = ($($res),*);

            // create regex automation with captures for each argument
            ::reformation::lazy_static!{
                static ref REGEX: ::reformation::Regex = {
                    let re_str = format!($re, $(<$res as ::reformation::Reformation>::regex_str()),*);
                    ::reformation::Regex::new(&re_str).unwrap()
                };
            }

            let captures = REGEX.captures($str).ok_or_else(||{
                ::reformation::NoRegexMatch{
                    format: $re,
                    request: $str.to_string()
                }
            })?;
            let mut i=0;
            Ok((
                $({
                    i += 1;
                    captures.get(i).unwrap().as_str().parse::<$res>()?
                }),*
            ))
        }
    };
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_float_parse(){
        // test regular expression for floating point numbers
        let re = regex::Regex::new(&format!("^{}$", f32::regex_str())).unwrap();
        // positive
        assert!(check_float_capture(&re, "10"));
        assert!(check_float_capture(&re, "10.2"));
        assert!(check_float_capture(&re, "10."));
        assert!(check_float_capture(&re, "0.34"));
        assert!(check_float_capture(&re, "00.34"));
        assert!(check_float_capture(&re, ".34"));
        assert!(check_float_capture(&re, ".34e2"));
        assert!(check_float_capture(&re, ".34e+2"));
        assert!(check_float_capture(&re, ".34e-2"));
        assert!(check_float_capture(&re, "-0.34e-2"));
        assert!(check_float_capture(&re, "5e-2"));
        assert!(check_float_capture(&re, "5.e-2")); // should this pass?

        // negative
        assert!(! re.is_match("5.."));
        assert!(! re.is_match("."));
        assert!(! re.is_match("--4."));
        assert!(! re.is_match("-.0"));
    }

    fn check_float_capture(r: &regex::Regex, s: &str)->bool{
         r.captures(s).map(|c|{
            c.len() == 2 && c.get(1).map(|x| x.as_str()) == Some(s)
        }).unwrap_or(false)
    }

}