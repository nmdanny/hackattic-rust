extern crate hackattic;
extern crate rdb;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate failure;

use hackattic::{HackatticChallenge, make_reqwest_client, from_base64};
use failure::Error;
use std::io::{Read, BufReader};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Problem {
    #[serde(deserialize_with = "from_base64")]
    rdb: Vec<u8>,
    requirements: Requirements
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Requirements {
    check_type_of: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Answer {
    db_count: u64,
    emoji_key_value: String,
    expiry_millis: u64,
    #[serde(rename = "$check_type_of")]
    check_type_of: String
}

fn main() {
    match main_err() {
        Ok(_) => (),
        Err(e) => panic!(format!("{:?}", e))
    }
}

fn main_err() -> Result<(), Error> {
    let mut client = make_reqwest_client()?;
    let problem = Redis::get_problem(&mut client)?;
    println!("Problem is {:?}", problem);
    let solution = Redis::make_solution(problem)?;
    Ok(())
}

fn parse_rdb(data: &[u8]) -> rdb::RdbOk {
    let mut data_vec = data.to_owned();
    let correct_string = b"REDIS".iter().cloned();
    // replace the first 5 bytes(containig 'MYSQL') with 'REDIS'
    let wrong_header = data_vec.splice(..5, correct_string).collect::<Vec<u8>>();
    println!("wrong header is {}", String::from_utf8_lossy(&wrong_header));
    let reader = BufReader::new(&data_vec[..]);
    rdb::parse(reader, rdb::formatter::JSON::new(), rdb::filter::Simple::new())
}

struct Redis;
impl HackatticChallenge for Redis {
    type Problem = Problem;
    type Solution = Answer;
    fn challenge_name() -> String {
        "the_redis_one".to_owned()
    }
    fn make_solution(req: Problem) -> Result<Answer, Error> {
        let rdb = parse_rdb(&req.rdb)?;
        println!("rdb is {:?}",rdb);
        unimplemented!()
    }

}