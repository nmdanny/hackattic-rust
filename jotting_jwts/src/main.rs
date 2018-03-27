extern crate hackattic_common;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

extern crate failure;
#[macro_use]
extern crate duct;
extern crate shell_escape;

#[macro_use]
extern crate log;
extern crate ansi_term;
extern crate pretty_env_logger;
extern crate reqwest;

use hackattic_common::{HackatticChallenge, make_reqwest_client};
use failure::{Error, Fail};

#[derive(Debug, Deserialize)]
struct Problem {
    jwt_secret: String
}

#[derive(Debug, Serialize)]
struct Solution {
    app_url: String
}

fn main() {
    #[cfg(target_os = "windows")] {
        ansi_term::enable_ansi_support();
    }
    let mut builder = pretty_env_logger::formatted_builder().unwrap();
    builder.filter(Some("jotting_jwts"), log::LevelFilter::Debug)
        .filter(Some("hackattic_common"), log::LevelFilter::Debug);
    builder.init();
    info!("Logger initialized");
    JottingJwts::process_challenge().unwrap();
}

struct JottingJwts;
impl HackatticChallenge for JottingJwts {
    type Problem = Problem;
    type Solution = Solution;

    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error> {
        unimplemented!()
    }

    fn challenge_name() -> &'static str {
        "jotting_jwts"
    }
}
