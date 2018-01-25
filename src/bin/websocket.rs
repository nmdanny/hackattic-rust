extern crate hackattic;
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

extern crate websocket;
extern crate tokio_core;
extern crate futures;

use hackattic::HackatticChallenge;
use std::time::{Instant, Duration};
use tokio_core::reactor::Core;
use websocket::async::Client;
use futures::{Future, Stream, Sink};
use websocket::async::stream::{AsyncRead, AsyncWrite};
use websocket::{ClientBuilder, OwnedMessage};
use futures::future::{ok, err, loop_fn, Loop};
use failure::Error;

#[derive(Debug, Deserialize)]
struct Problem {
    token: String
}

#[derive(Debug, Serialize)]
struct Solution {
    secret: String
}

fn main() {
    ansi_term::enable_ansi_support().unwrap();
    let mut builder = pretty_env_logger::formatted_builder().unwrap();
    builder.filter(Some("websocket"), log::LevelFilter::Info);
    builder.init();
    info!("Logger initialized");
    Websocket::process_challenge().unwrap();

}

struct PingState {
    previous_ping: Option<Instant>,
}
impl PingState {
    fn new() -> Self {
        PingState {
            previous_ping: None
        }
    }
    fn handle_message<S: 'static + AsyncWrite + AsyncRead>(mut self, message: &str, client: Client<S>)
        -> Box<Future<Item = Loop<String, (Client<S>, Self)>, Error = Error>> {
        if message.contains("ouch! no! that was a") {
            Box::new(err(format_err!("Incorrect duration: {}", message)))
        } else if message.contains("missed a beat") {
            Box::new(err(format_err!("Missed a ping: {}", message)))
        } else if message.contains("ping!") || message.contains("hello! start counting") {
            let now = Instant::now();
            if let Some(previous_ping) = self.previous_ping {
                let duration = now - previous_ping;
                let approx_ms = Self::duration_to_interval(&duration);
                info!("Got a ping, measured latency is {:?}, which is approximately {} ms", duration, approx_ms);
                let msg = OwnedMessage::Text(approx_ms);
                self.previous_ping = Some(now);
                let send_fut = client.send(msg)
                    .map_err(|e| Error::from(e))
                    .map(|client| {
                        Loop::Continue((client, self))
                    });
                return Box::new(send_fut);
            } else {
                info!("Got initial ping")
            }
            self.previous_ping = Some(now);
            Box::new(ok(Loop::Continue((client, self))))
        } else if message.contains("good!") {
            info!("Got good message, continuing");
            Box::new(ok(Loop::Continue((client, self))))
        } else if message.contains("congratulations!") {
            let secret = message.trim_left_matches("congratulations! the solution to this challenge is \"")
                .trim_right_matches("\"");
            Box::new(ok(Loop::Break(secret.to_owned())))
        } else {
            warn!("Got unknown message: {}, skipping", message);
            Box::new(ok(Loop::Continue((client, self))))
        }
    }

    fn duration_to_interval(duration: &Duration) -> String {
        let intervals = [700, 1500, 2000, 2500, 3000];
        let duration_ms = ((duration.as_secs() as f64 * 1e9 + duration.subsec_nanos() as f64)/1e6) as i32;
        let closest_interval = intervals.iter()
            .map(|&interval| (interval, (interval - duration_ms).abs()))
            .min_by_key(|&t| t.1)
            .unwrap();
        closest_interval.0.to_string()
    }
}

struct Websocket;
impl HackatticChallenge for Websocket {
    type Problem = Problem;
    type Solution = Solution;


    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error> {
        let mut core = Core::new()?;
        let url = format!("wss://hackattic.com/_/ws/{}", problem.token);
        let client_future = ClientBuilder::new(&url)?
            .async_connect(None,&core.handle())
            .map_err(|e| Error::from(e))
            .and_then(|(client, _)| {
                info!("Client obtained, looping...");
                loop_fn((client, PingState::new()),|(stream, ping)| {
                    stream.into_future()
                        .or_else(|(e,_)| {
                            Box::new(err(Error::from(e))) // map WebSocketError to failure::Error
                        })
                        .and_then(|(msg, stream)| {
                            if let Some(OwnedMessage::Text(text)) = msg {
                                ping.handle_message(&text, stream)
                            } else if let Some(OwnedMessage::Close(_)) = msg {
                                Box::new(err(format_err!("Unexpected close message encountered!")))
                            } else {
                                warn!("Unknown message type, skipping.");
                                Box::new(ok(Loop::Continue((stream, ping))))
                            }
                        })
                })
            }).map_err(|err| Error::from(err));


        let secret = core.run(client_future)?;
        Ok(Solution {
            secret
        })
    }

    fn challenge_name() -> &'static str {
        "websocket_chit_chat"
    }
}