extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate hackattic;
extern crate byteorder;
extern crate failure;

use hackattic::{HackatticChallenge, from_base64};
use failure::Error;
use byteorder::{LE, BE, ReadBytesExt};
use std::io::Cursor;
#[derive(Debug, Deserialize)]
struct Problem {
    #[serde(deserialize_with = "from_base64")]
    bytes: Vec<u8>
}

#[derive(Debug, Serialize)]
struct Solution {
    int: i32,
    uint: u32,
    short: i16,
    float: f64, // Hackattic seems to expect this to have a double precision, even though it's read as a float.
    double: f64,
    big_endian_double: f64
}

fn main() {
    HelpMeUnpack::process_challenge().unwrap();
}

struct HelpMeUnpack;
impl HackatticChallenge for HelpMeUnpack {
    type Problem = Problem;
    type Solution = Solution;

    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error> {
        println!("Input has {} bytes.", problem.bytes.len());
        println!("Input is {}", hackattic::HexSlice::new(&problem.bytes));
        let mut reader = Cursor::new(problem.bytes.to_owned());
        let int = reader.read_i32::<LE>()?;
        let uint = reader.read_u32::<LE>()?;
        let short = reader.read_i16::<LE>()?;
        let _padding = reader.read_i16::<LE>()?;
        let float = reader.read_f32::<LE>()? as f64;
        let double = reader.read_f64::<LE>()?;
        let big_endian_double = reader.read_f64::<BE>()?;
        Ok(Solution {
            int, uint, short, float, double, big_endian_double
        })
    }

    fn challenge_name() -> &'static str {
        "help_me_unpack"
    }
}