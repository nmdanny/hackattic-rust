extern crate hackattic;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate failure;
extern crate diesel;
extern crate tempfile;

use hackattic::{HackatticChallenge, from_base64};
use failure::{Error, ResultExt};
use std::io::{Read, Write, BufWriter};
use std::process::{Command, Stdio};

#[derive(Deserialize, Debug, Clone)]
struct Problem {
    #[serde(deserialize_with = "from_base64")]
    dump: Vec<u8>
}

#[derive(Debug, Clone, Serialize)]
struct Solution {
    alive_ssns: Vec<String>
}

fn main() {
    main_err().unwrap();
}

fn main_err() -> Result<(), Error> {
    BackupRestore::process_challenge()?;
    Ok(())
}

fn load_db(dump: &[u8]) -> Result<(), Error> {
    let mut file = ::tempfile::NamedTempFile::new()?;
    file.write_all(dump);
    let file_path = file.path().to_str().unwrap();
    println!("pg_restore -C -d hackattic_dump -d postgres {}", file_path);
    let cmd = Command::new("pg_restore")
        .args(&["-C", "-d hackattic_dump", "-d postgres", file_path])
        .output()?;
    Ok(())

}

struct BackupRestore;
impl HackatticChallenge for BackupRestore {
    type Problem = Problem;
    type Solution = Solution;
    fn challenge_name() -> String {
        "backup_restore".to_owned()
    }
    fn make_solution(req: Problem) -> Result<Solution, Error> {
        load_db(&req.dump)?;
        Ok(Solution {
            alive_ssns: vec![]
        })
    }
}