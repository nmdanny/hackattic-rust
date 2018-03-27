extern crate hackattic_common;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate log;
extern crate ansi_term;
extern crate pretty_env_logger;

extern crate hyper;
extern crate tokio_core;
extern crate futures;
extern crate reqwest;

use hackattic_common::{HackatticChallenge, make_reqwest_client};
use failure::{Error, Fail};
use futures::{Future, Stream};
use hyper::server::{Http, Request, Response, Service};
use std::cell::Cell;

struct JottingService {
    string: Cell<String>,
    jwt_secret: String
}
impl JottingService {
    fn from_secret(secret: String) -> Self {
        Self {
            jwt_secret: secret,
            string: Cell::default()
        }
    }
}

impl Service for JottingService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        debug!("Got request {:?}", req);
        Box::new(futures::future::ok(
            Response::new()
                .with_body("TODO")
        ))
    }
}


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
        ansi_term::enable_ansi_support().unwrap();
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
        let mut core = tokio_core::reactor::Core::new()?;

        let addr = "0.0.0.0:80".parse()?;
        let secret = problem.jwt_secret.clone();
        let server = Http::new().bind(&addr, move || {
            // TODO: share 'service' between different connections (will need to make it Sync by wrapping its string in a mutex)
            debug!("Creating a JottingService for new connection");
            let service = JottingService::from_secret(secret.clone());
            Ok(service)
        }).map_err(|hyp_err| format_err!("Error while binding server: {:?}", hyp_err))?;
        let handle = server.handle();

        info!("Began listening to connections on http://{}", server.local_addr()?);

        let solution = Solution {
            app_url: "http://nmdanny.ddns.net".to_owned()
        };
        
        /*
        let mut client = make_reqwest_client()?;
        // TODO: schedule this to happen after running the server
        handle.spawn_fn(move || {
            info!("sending server's http server...");
            let res = Self::send_solution(&solution, &mut client).unwrap_or_else(|e| panic!("Couldn't send server address to hackattic: {:?}", e));
            info!("response of sending server's http address: {:}", res);
            
            futures::future::ok(())
        });
        */
        server.run()?;
        unreachable!()
    }

    fn challenge_name() -> &'static str {
        "jotting_jwts"
    }
}
