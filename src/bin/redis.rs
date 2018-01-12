extern crate hackattic;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate failure;
extern crate rdb_parser;

use hackattic::{HackatticChallenge, from_base64};
use failure::{Error, ResultExt};
use rdb_parser::types::RedisValue;

#[derive(Deserialize, Debug, Clone)]
struct Problem {
    #[serde(deserialize_with = "from_base64")]
    rdb: Vec<u8>,
    requirements: Requirements
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Requirements {
    check_type_of: String
}

#[derive(Debug, Clone)]
struct Answer {
    db_count: usize,
    emoji_key_value: String,
    expiry_millis: u64,
    check_type_of: CustomName<String>
}

// TODO: not really related to this exercise, but find a better way of dynamically
// renaming a field for serializing
#[derive(Debug, Clone)]
struct CustomName<T: serde::Serialize>{
    name: String,
    value: T
}
impl <T: serde::Serialize> std::ops::Deref for CustomName<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl serde::Serialize for Answer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: serde::Serializer {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("db_count", &self.db_count)?;
        map.serialize_entry("emoji_key_value", &self.emoji_key_value)?;
        map.serialize_entry("expiry_millis", &self.expiry_millis)?;
        map.serialize_entry(&self.check_type_of.name, &self.check_type_of.value)?;
        map.end()
    }
}

fn main() {
    Redis::process_challenge().unwrap();
}

fn fix_rdb_header(data: &mut Vec<u8>) {
    let fixed_header = b"REDIS".iter().cloned();
    data.splice(..5, fixed_header).collect::<Vec<_>>();
}

// According to https://en.wikipedia.org/wiki/Emoticons_(Unicode_block)
// BUG: doesn't seem to include some emojis
fn is_emoji_codepoint(ch: char) -> bool {
    ch >= '\u{1f600}' && ch <= '\u{1f64f}'
}


struct Redis;
impl HackatticChallenge for Redis {
    type Problem = Problem;
    type Solution = Answer;
    fn challenge_name() -> &'static str {
        "the_redis_one"
    }
    fn make_solution(req: &Problem) -> Result<Answer, Error> {
        let mut rdb = req.rdb.to_owned();
        fix_rdb_header(&mut rdb);
        let rdb: rdb_parser::types::RDB = rdb_parser::rdb(&rdb).to_result().unwrap();
        let db_count = rdb.databases.len();
        let mut emoji_key_value = String::from("TODO");
        let mut expiry_millis = 0;
        let mut check_type_of_value = String::from("TODO");
        let mut found_duration = None;
        for entry in rdb.databases.into_iter().flat_map(|d| d.entries) {
            if let Some(duration) = entry.expiry {
                found_duration = Some(duration);
                expiry_millis = duration.as_secs() * 1000 + (duration.subsec_nanos() as u64) / 1000000;
            }
            if String::from_utf8_lossy(&entry.key).chars().any(is_emoji_codepoint) {
                emoji_key_value = format!("{}", entry.value);
            }
            if &entry.key == &req.requirements.check_type_of.as_bytes() {
                check_type_of_value = match entry.value {
                    RedisValue::String(_) => String::from("string"),
                    RedisValue::List(_) => String::from("list"),
                    RedisValue::Hash(_) => String::from("hash"),
                    RedisValue::Set(_) => String::from("set"),
                    RedisValue::SortedSet(_) => String::from("sortedset"),
                }
            }
        }
        println!("Found duration: {:?}", found_duration);
        let answer = Answer {
            db_count,
            emoji_key_value,
            expiry_millis,
            check_type_of: CustomName {
                name: req.requirements.check_type_of.to_owned(),
                value: check_type_of_value
            }
        };
        Ok(answer)
    }

}
