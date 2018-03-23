extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use failure::Error;
use std::fmt::Debug;

/* utility libraries useful for many challenges */
mod hex_slice;
pub use hex_slice::*;
mod serde_utils;
pub use serde_utils::*;

lazy_static! {
    static ref ACCESS_TOKEN: String = {
        let token = std::env::var_os("HACKATTIC_ACCESS_TOKEN");
        if let Some(token) = token {
            token.to_string_lossy().into_owned()
        } else {
            panic!("Missing HACKATTIC_ACCESS_TOKEN environment variable.")
        }
    };

    static ref REPEAT: bool = {
        if let Some(_) = std::env::var_os("HACKATTIC_RETRY") {
            debug!("HACKATTIC_RETRY is set, will force checking of solved challenges.");
            true
        } else {
            debug!("HACKATTIC_RETRY isn't set, solved challenges won't be checked.");
            false
        }
    };
}

pub fn make_reqwest_client() -> Result<reqwest::Client, Error> {
    let mut builder = reqwest::Client::builder();
    if std::env::args()
        .find(|a| a.to_lowercase() == "fiddler")
        .is_some()
    {
        builder.proxy(reqwest::Proxy::https("http://127.0.0.1:8888")?);
    }
    let client = builder.build()?;
    Ok(client)
}

pub trait HackatticChallenge {
    type Problem;
    type Solution;
    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error>;
    fn challenge_name() -> &'static str;
    fn get_problem(client: &mut reqwest::Client) -> Result<Self::Problem, Error>
    where
        Self::Problem: serde::de::DeserializeOwned,
    {
        let problem_json = client
            .get(&format!(
                "https://hackattic.com/challenges/{}/problem?access_token={}",
                Self::challenge_name(),
                &*ACCESS_TOKEN
            ))
            .send()?;
        let problem = serde_json::from_reader(problem_json)?;
        Ok(problem)
    }
    fn send_solution(
        solution: &Self::Solution,
        client: &mut reqwest::Client,
    ) -> Result<String, Error>
    where
        Self::Solution: serde::Serialize,
    {
        let playground = if *REPEAT { "&playground=1" } else { "" };
        let mut response = client
            .post(&format!(
                "https://hackattic.com/challenges/{}/solve?access_token={}{}",
                Self::challenge_name(),
                &*ACCESS_TOKEN,
                playground
            ))
            .json(solution)
            .send()?;
        Ok(format!("{}", response.text()?))
    }

    fn process_challenge() -> Result<(), Error>
    where
        Self::Problem: serde::de::DeserializeOwned + Debug,
        Self::Solution: serde::Serialize + Debug,
    {
        info!("processing challenge \"{}\"", Self::challenge_name());
        let mut client = make_reqwest_client()?;
        let problem = Self::get_problem(&mut client)?;
        debug!("got problem: {:?}", problem);
        let solution = Self::make_solution(&problem)?;
        debug!("sending solution: {:?}", solution);
        let response = Self::send_solution(&solution, &mut client)?;
        debug!("got response: {}", response);
        Ok(())
    }
}
