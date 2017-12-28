extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate failure;

use failure::Error;
use std::fmt::Debug;

mod hex_slice;
pub use hex_slice::*;
mod serde_utils;
pub use serde_utils::*;

pub static ACCESS_TOKEN: &str = "ACCESS_TOKEN_HERE";


pub fn make_reqwest_client() -> Result<reqwest::Client, Error>  {
    let mut builder = reqwest::Client::builder();
    if std::env::args().find(|a| a.to_lowercase() == "fiddler").is_some() {
        builder.proxy(reqwest::Proxy::https("http://127.0.0.1:8888")?);
    }
    let client = builder.build()?;
    Ok(client)
}

pub trait HackatticChallenge {
    type Problem;
    type Solution;
    fn make_solution(problem: Self::Problem) -> Result<Self::Solution, Error>;
    fn challenge_name() -> String;
    fn get_problem(client: &mut reqwest::Client) -> Result<Self::Problem, Error>
        where Self::Problem : serde::de::DeserializeOwned 
    {
        let problem_json = client
            .get(&format!("https://hackattic.com/challenges/{}/problem?access_token={}", Self::challenge_name(), ACCESS_TOKEN))
            .send()?;
        let problem = serde_json::from_reader(problem_json)?;
        Ok(problem)
    }
    fn send_solution(solution: Self::Solution, client: &mut reqwest::Client) -> Result<String, Error>
        where Self::Solution : serde::Serialize {
            let mut response = client.post(&format!("https://hackattic.com/challenges/{}/solve?access_token={}", Self::challenge_name(), ACCESS_TOKEN))
                .json(&solution)
                .send()?;
            Ok(format!("{}", response.text()?))
        }

    fn process_challenge() -> Result<(), Error>
        where Self::Problem : serde::de::DeserializeOwned + Debug, Self::Solution : serde::Serialize + Debug
    {
        println!("processing challenge \"{}\"", Self::challenge_name());
        let mut client = make_reqwest_client()?;
        let problem = Self::get_problem(&mut client)?;
        println!("got problem: {:?}", problem);
        let solution = Self::make_solution(problem)?;
        println!("got solution: {:?}", solution);
        let response = Self::send_solution(solution, &mut client)?;
        println!("got response: {}", response);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
