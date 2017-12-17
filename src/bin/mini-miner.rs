extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate ring;
extern crate hackattic;

use serde_json::Value;
use std::error::Error;
use ring::digest::{SHA256, digest};
use hackattic::ACCESS_TOKEN;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Problem {
    difficulty: usize,
    block: Block
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    data: Vec<Value>,
    nonce: Option<u64>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Answer {
    nonce: u64
}


fn hash_block(block: &Block) -> Vec<u8> {
    let json_block = serde_json::to_string(block).unwrap();
    let digest = digest(&SHA256, json_block.as_bytes());
    digest.as_ref().to_owned()
}

fn solve_problem(mut problem: Problem) -> Result<Answer, Box<Error>> {
    problem.block.nonce = Some(0);
    loop {
        let block_hash = hash_block(&problem.block);
        if test_hash(&block_hash, problem.difficulty) {
            return Ok(Answer {
                nonce: problem.block.nonce.unwrap()
            });
        }
        problem.block.nonce.as_mut().map(|nonce| *nonce += 1);
    }
}

fn test_hash(hash: &[u8], mut difficulty: usize) -> bool {
    let mut index = 0;
    while difficulty > 8usize && index < hash.len() - 1 {
        if hash[index] != 0 {
            return false;
        }
        difficulty -= 8usize;
        index += 1;
    }
    hash[index].leading_zeros() as usize >= difficulty
}


fn main() {
    match main_err() {
        Ok(_) => (),
        Err(e) => panic!(format!("{:?}", e))
    }
}

fn main_err() -> Result<(), Box<Error>> {
    let problem_json = reqwest::get(&format!("https://hackattic.com/challenges/mini_miner/problem?access_token={}", ACCESS_TOKEN))?;
    let problem: Problem = serde_json::from_reader(problem_json)?;
    println!("Got problem {:?}", problem);
    let answer = solve_problem(problem)?;
    println!("Answer is {:?}", answer);
    let client = reqwest::Client::new();
    let response = client.post(&format!("https://hackattic.com/challenges/mini_miner/solve?access_token={}", ACCESS_TOKEN))
        .json(&answer)
        .send()?;
    println!("Response is {:?}", response);
    Ok(())
}

#[test]
fn can_deserialize_problem() {
    let json = json!({
          "difficulty": 13,
          "block": {
            "nonce": null,
            "data": [
              [
                "a65a7af80a0881c3b0b0f168e853a1fb",
                -64
              ],
              [
                "4e6d80dd9b3f83bc51b4827c48ee9d31",
                -12
              ],
              [
                "b991c65ba20f09823ae4be755fc3da8b",
                22
              ],
              [
                "67d609c670df18f4940e2feeb6680605",
                12
              ],
              [
                "36719eb2d89c5ae68131e1c68554f3c2",
                9
              ]
            ]
          }     
    });
    let p: Problem = serde_json::from_value(json).unwrap();
}

#[test]
fn hashing_is_expected() {
    let block = Block {
        nonce: Some(45), data: Vec::new()
    };
    let digest = hash_block(&block);
    let expected = ring::test::from_hex("00d696db487caf06a2f2a8099479577c3785c37b3d8a77dc413cfb19ec2e0141").unwrap();
    assert_eq!(digest, expected, "SHA256 digest of block should match the expected one");
    assert_eq!(test_hash(&digest, 8), true, "block digest should test positively for a difficulty of 8");
}