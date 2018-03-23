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
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Problem {
    push_token: String,
    username: String,
    repo_path: PathBuf,
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


fn su_as(cmd: duct::Expression, user: &str) -> duct::Expression {
    cmd.pipe(cmd!("su", user))
}
fn sudo_as(cmd: duct::Expression, user: &str) -> duct::Expression {
    cmd.pipe(cmd!("sudo", "-u", user, "sh"))
}

//#[cfg(target_os = "unix")]
fn setup_git_server(problem: &Problem) -> Result<(), Error> {

    info!("Creating user \"{}\"", &problem.username);
    sudo_as(cmd!("adduser", "--disabled-password", "--gecos", "''", &problem.username), "root").run()?;

    info!("Adding specified ssh-key to the newly adder user's authorized_keys");
    su_as(cmd!("mkdir ~/.ssh && chmod 700 ~/.ssh"), &problem.username).run()?;
    su_as(cmd!("touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"), &problem.username).run()?;
    su_as(cmd!("cat").input(problem.ssh_key.as_bytes()).pipe(cmd!("tee", "-a", "~/.ssh/authorized_keys")), &problem.username).run()?;


    let repo_path = format!("~/{:?}", &problem.repo_path);
    info!("Initializing git directory structure at the following folder: {}", repo_path);
    su_as(cmd!("mkdir", "-p", &repo_path).then(
          cmd!("cd", &repo_path)).then(
          cmd!("git", "init", "--bare")),
          &problem.username).run()?;

    
    sudo_as(cmd!("chsh", &problem.username, "-s", "$(which git-shell)"), "root").run()?;

    info!("(Re)starting ssh-server");
    sudo_as(cmd!("service", "ssh", "restart"), "root").run()?;
    info!("git-server ready!!");

    Ok(())
}

/*
#[cfg(not(target_os = "unix"))]
fn setup_git_server(problem: &Problem) -> Result<(), Error> {
    panic!("setup_git_server currently only works on *nix systems");
}
*/

#[derive(Debug, Serialize)]
struct Solution {
    secret: String
}

fn dummy_run() {
    let problem = Problem {
        push_token: "nevermind".to_owned(),
        username: "somebody".to_owned(),
        repo_path: "who/cares.git".into(),
        // pkey of nmdanny on wsl
        ssh_key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDARUm67vu1ibLIV0yoF/ObZvHyNbviuFCFnlvaukmUIjL1067T2CX5/+rtPhj19l+pKeKJ/t+PYB9tBuRaMv+cfJEMDlJkOshTVsciev85ncUSODy/+2SQelN34+GsFlgzOmVcNE5LMSPId78IeiHkh/BOB/bKo68PXpZLOJncvk5/LjUWXz7E/n460NbEZhIvuCzBcmcAcE8o/sU8plWReSOrbykPTb7jk4+xpJDO0TEIoj7QemNRU8Ms8ruAukE7bg349si45XcZoJ0adtQnpFvxm9/LglkrTecCsp1taEqn5Owwnw75eOdM9MmyYZp99ljMdzcCTpSq5OcF/0eJ nmdanny@DESKTOP-7VLKOF6".to_owned()
    };
    setup_git_server(&problem).unwrap();
}

fn main() {
    #[cfg(target_os = "windows")] {
        ansi_term::enable_ansi_support();
    }
    let mut builder = pretty_env_logger::formatted_builder().unwrap();
    builder.filter(Some("hosting_git"), log::LevelFilter::Debug);
    builder.init();
    info!("Logger initialized");
    dummy_run();
    //HostingGit::process_challenge().unwrap();
}

struct HostingGit;
impl HackatticChallenge for HostingGit {
    type Problem = Problem;
    type Solution = Solution;

    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error> {
        info!("Processing problem:\n{:?}", problem);
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