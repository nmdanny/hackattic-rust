extern crate hackattic;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate tempfile;

use hackattic::{HackatticChallenge, make_reqwest_client, as_base64};
use failure::{Error, ResultExt};
use std::io::{Read, Write};

#[derive(Deserialize, Debug, Clone)]
struct Problem {
    //#[serde(deserialize_with = "from_base64")]
    include: String
}

#[derive(Debug, Clone, Serialize)]
struct Solution {
    files: Vec<Base64>
}

#[derive(Debug, Clone, Serialize)]
struct Base64 (
    #[serde(serialize_with = "as_base64")]
    Vec<u8>
);

impl From<Vec<u8>> for Base64 {
    fn from(data: Vec<u8>) -> Self {
        Base64(data)
    }
}
impl <'a> From<&'a [u8]> for Base64 {
    fn from(data: &[u8]) -> Self {
        Base64(data.to_owned())
    }
}
fn main() {
    CollisionCourse::process_challenge().unwrap();
}


fn create_collision(mut include: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let mut prefix_file = tempfile::NamedTempFile::new()?;
    prefix_file.write_all(&mut include)?;
    let cmd = std::process::Command::new("extra/collision_course/fastcoll_v1.0.0.5.exe")
        .arg("-p")
        .arg(prefix_file.path())
        .output()?;
    let mut msg1 = std::fs::File::open("msg1.bin")?;
    let mut msg2 = std::fs::File::open("msg2.bin")?;
    let mut file0 = Vec::new();
    let mut file1 = Vec::new();
    msg1.read_to_end(&mut file0)?;
    msg2.read_to_end(&mut file1)?;
    std::fs::remove_file("msg1.bin")?;
    std::fs::remove_file("msg2.bin")?;
    Ok((file0, file1))
}


struct CollisionCourse;
impl HackatticChallenge for CollisionCourse {
    type Problem = Problem;
    type Solution = Solution;
    fn challenge_name() -> &'static str {
        "collision_course"
    }
    fn make_solution(req: &Problem) -> Result<Solution, Error> {
        let (file0, file1) = create_collision(req.include.as_bytes())?;
        Ok(Solution {
            files: vec![Base64::from(file0), Base64::from(file1)]
        })
    }
}