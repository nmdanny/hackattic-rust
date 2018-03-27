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
extern crate jsonwebtoken;

use hackattic_common::{HackatticChallenge, make_reqwest_client};
use failure::{Error, Fail};
use futures::{Future, Stream};
use hyper::server::{Http, Request, Response, Service};
use std::cell::RefCell;
use std::rc::Rc;
use jsonwebtoken as jwt;

struct JottingService {
    string: Rc<RefCell<String>>,
    jwt_secret: String,
    jwt_validation: jwt::Validation
}

impl JottingService {
    fn from_secret(secret: String) -> Self {
        Self {
            jwt_secret: secret,
            string: Rc::new(RefCell::default()),
            jwt_validation: jwt::Validation { leeway: 5, ..jwt::Validation::default() }
        }
    }

}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JwtClaims {
    Append { append: String },
    Empty {},
}

impl Service for JottingService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        debug!("Got request {:?}", req);
        
        let secret = self.jwt_secret.clone().into_bytes();
        let validation = self.jwt_validation.clone();
        let string = self.string.clone();

        let fut = req.body().concat2().and_then(move |jwt_bytes| {
            debug!("Got body, decoding as JWT...");
            let jwt_st = String::from_utf8_lossy(&jwt_bytes);

            let jwt = match jwt::decode::<JwtClaims>(&jwt_st, &secret, &validation) {
                Ok(jwt) => jwt,
                Err(e) => {
                    error!("There was an error validating a JWT: {:?} - skipping", e);
                    return Ok(Response::new().with_body(format!("Skipping due to invalid JWT: {:?}", e)));
                }
            };
            debug!("JWT is {:?}", jwt);
            match jwt.claims {
                JwtClaims::Append { append } => {
                    info!("Appending '{}' to '{}'", append, string.borrow());
                    let mut string = string.borrow_mut();
                    string.push_str(&append);
                    Ok(Response::new().with_body("continue..."))
                },
                JwtClaims::Empty {} => {
                    let res = SolutionResponse { solution: string.borrow().clone() };
                    let json_bytes = serde_json::to_vec(&res).unwrap();
                    info!("Got end message, secret is {}", res.solution);
                    Ok(Response::new().with_body(json_bytes))
                }
            }
        });
        Box::new(fut)
    }
}


#[derive(Debug, Deserialize)]
struct Problem {
    jwt_secret: String
}

#[derive(Debug, Serialize)]
struct SolutionResponse {
    solution: String
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
        let addr = "0.0.0.0:80".parse()?;
        let secret = problem.jwt_secret.clone();
        let service = Rc::new(JottingService::from_secret(problem.jwt_secret.to_owned()));

        let server = Http::new().bind(&addr, move || {
            debug!("Binding new connection to service...");
            let service = service.clone();
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
