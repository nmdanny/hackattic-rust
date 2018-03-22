extern crate hackattic_common;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate duct;

#[macro_use]
extern crate log;
extern crate ansi_term;
extern crate pretty_env_logger;
extern crate rustyline;
extern crate reqwest;

use duct::cmd;
use std::process::{Command, Output};
use std::io::{Write, Read};
use hackattic_common::{HackatticChallenge, make_reqwest_client};
use failure::{Error, ResultExt};

#[derive(Debug, Deserialize)]
struct Problem {
    push_token: String,
    username: String,
    repo_path: String,
    ssh_key: String
}


fn push_message(push_token: &str, repo_host: &str) -> Result<reqwest::Response, Error> {
        #[derive(Debug, Serialize)]
        struct PushMessage {
            repo_host: String
        }

        let client = make_reqwest_client()?;
        let url = format!("https://hackattic.com/_/git/{}", push_token);
        let message = PushMessage {
            repo_host: repo_host.to_owned()
        };
        let res = client.post(&url).json(&message).send()?;
        Ok(res)
}

#[cfg(target_os = "unix")]
fn setup_git_server(problem: &Problem) -> Result<(), Error> {
    // BUG: command injection possible wherever format! is used

    info!("Creating user \"{}\"", problem.username);
    cmd!("adduser", problem.username, "--disabled-password").run()?;

    info!("Adding specified ssh-key to authorized_keys");
    cmd!("su", "-c", "mkdir .ssh && chmod 700 .ssh", problem.username).run()?;
    cmd!("su", "-c", "touch .ssh/authorized_keys && chmod 600 .ssh/authorized_keys", problem.username).run()?;
    cmd!("su", "-c", format!("echo {} >> .ssh/authorized_keys", problem.ssh_key), problem.username).run()?;

    let repo_folder = problem.repo_path.split("/").next().unwrap();
    info!("Initializing git directory structure at the following folder: {}", repo_folder);
    cmd!("su", "-c", format!("mkdir {} && cd {} && git init --bare", repo_folder, repo_folder), problem.username).run()?;

    
    cmd!("chsh", problem.username, "-s", "$(which git-shell)").run()?;

    info!("(Re)starting ssh-server");
    cmd!("service", "ssh", "restart");
    info!("git-server ready!!");

    Ok(())
}

#[cfg(not(target_os = "unix"))]
fn setup_git_server(problem: &Problem) -> Result<(), Error> {
    panic!("setup_git_server currently only works on *nix systems");
}

#[derive(Debug, Serialize)]
struct Solution {
    secret: String
}


fn main() {
    ansi_term::enable_ansi_support().unwrap();
    let mut builder = pretty_env_logger::formatted_builder().unwrap();
    builder.filter(Some("git_server"), log::LevelFilter::Debug);
    builder.init();
    info!("Logger initialized");
    HostingGit::process_challenge().unwrap();
    info!("Challenge processed.");
}

struct HostingGit;
impl HackatticChallenge for HostingGit {
    type Problem = Problem;
    type Solution = Solution;

    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error> {
        setup_git_server(problem)?;
        push_message(&problem.push_token, "nmdanny.ddns.net")?;
        let mut rl = rustyline::Editor::<()>::new();
        let secret = rl.readline("Type the secret you've figured out here: ")?;
        Ok(Solution {
            secret
        })
    }

    fn challenge_name() -> &'static str {
        "hosting_git"
    }
}