extern crate hackattic;
extern crate rdb;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate failure;
extern crate reqwest;

use hackattic::{HackatticChallenge, make_reqwest_client};
use failure::{Error, ResultExt};
use std::thread;
use std::sync::mpsc;
use std::time::{Instant, Duration};

static PROXIES: &'static [&str] = &[
    "http://127.0.0.1:8888", // fiddler
    // random https proxies from the web
    "http://51.15.83.8:3128",
    "http://190.0.35.6:3128",
    "http://5.160.237.242:8080",
    "http://180.244.11.220:8080",
    "http://177.131.51.142:8080",
    "http://188.214.122.153:53281",
    "http://188.225.189.215:8080",
    "http://203.177.36.84:55555",
    "http://197.250.8.162:65103",
    "http://168.253.92.10:8080",
    "http://78.26.207.173:53281",
    "http://122.154.235.18:8080",
    "http://118.179.151.172:8080",
    "http://88.103.229.151:8081"
];

#[derive(Deserialize, Debug, Clone)]
struct Problem {
    presence_token: String
}


#[derive(Debug, Clone, Serialize)]
struct Answer {

}



fn main() {
    GlobalPresence::process_challenge().unwrap();
}

fn call_with_proxy(presence_token: &str, proxy: &str ) -> Result<String, Error> {
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::https(proxy)?)
        .build()?;
    let mut resp = client.get(&format!("https://hackattic.com/_/presence/{}", presence_token))
        .send().context(format!("with proxy {}", proxy))?;
    let st = resp.text()?;
    Ok(st)
}

fn coordinate_https(token: &str) -> Result<(), Error> {
    let (tx, rx) = mpsc::channel();
    for &proxy in PROXIES.iter() {
        let tx = tx.clone();
        let token = token.to_owned();
        thread::spawn(move || {
            let res = call_with_proxy(&token, &proxy);
            tx.send(res).unwrap();
        });
    }
    println!("Spawned {} threads", PROXIES.len());
    let mut count = 0;
    let mut nations = std::collections::HashSet::new();
    let begin = Instant::now();
    while count < PROXIES.len() && nations.len() < 7 &&
          Instant::now().duration_since(begin) < Duration::from_secs(25) {
        rx.recv().map(|r| match r {
           Ok(s) => {
               nations.extend(s.split(',').map(|s| s.to_owned()));
           },
            Err(e) => eprintln!("HTTP request yielded an error: {:?}", e)
        }).unwrap_or_else(|e| {
            eprintln!("Error receiving: {:?}", e)
        });
        count += 1;
    }
    println!("Finished with {} unique nations: {:?}", nations.len(), nations);
    Ok(())
}


struct GlobalPresence;
impl HackatticChallenge for GlobalPresence {
    type Problem = Problem;
    type Solution = Answer;
    fn challenge_name() -> &'static str {
        "a_global_presence"
    }
    fn make_solution(req: &Problem) -> Result<Answer, Error> {
        coordinate_https(&req.presence_token)?;
        Ok(Answer {})
    }

}
