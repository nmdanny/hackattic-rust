extern crate hackattic_common;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

extern crate failure;
#[macro_use]
extern crate duct;
extern crate shell_escape;

#[macro_use]
extern crate log;
extern crate ansi_term;
extern crate pretty_env_logger;
extern crate reqwest;

use std::io::Read;
use hackattic_common::{HackatticChallenge, make_reqwest_client};
use failure::{Error, Fail};
use std::path::PathBuf;
use shell_escape::escape;

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

/// Represents a linux user that will be automatically deleted when dropped.
struct TempUser {
    name: String,
}
impl TempUser {
    fn new<S: Into<String>>(name: S) -> Result<Self, Error> {
        let name = name.into();
        cmd!("sudo", "useradd", "-m", &name).run()?;
        debug!("created user \"{}\"", name);
        Ok(TempUser {
            name
        })
    }
}
impl Drop for TempUser {
    fn drop(&mut self) {
        
        debug!("deleting temporary user \"{}\"", self.name);
        if let Err(e) = cmd!("sudo", "userdel", "-fr", &self.name).run() {
            error!("failed to delete temporary user \"{}\": {:?}", self.name, e);
        }
    }
}

#[cfg(target_os = "linux")]
fn setup_git_server(problem: &Problem) -> Result<TempUser, Error> {
    let username: String = escape(std::borrow::Cow::Borrowed(&problem.username)).into_owned();
    debug!("Creating user \"{}\"", &username);
    let user = TempUser::new(username.clone())?;

    debug!("Adding specified ssh-key to the newly adder user's authorized_keys");
    cmd!("sudo","-u", &username, "sh", "-c", "mkdir -p ~/.ssh && chmod 700 ~/.ssh").run()?;
    cmd!("sudo","-u", &username, "sh", "-c", "touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys").run()?;
    cmd!("sudo","-u", &username, "sh", "-c", "cat | tee -a ~/.ssh/authorized_keys").input(problem.ssh_key.as_bytes()).run()?;


    let user_home = cmd!("sudo", "-u", &username, "sh", "-c", "echo $HOME").read()?;
    let repo_path = format!("{}/{}", user_home, problem.repo_path.to_string_lossy());
    debug!("Initializing git directory structure at the following folder: {}", &repo_path);
    cmd!("sudo","-u", &username, "mkdir", "-p", &repo_path).run()?;
    cmd!("sudo", "-u", &username, "git", "init", "--bare", &repo_path).run()?;

    debug!("Disabling shell access for git user");
    let git_shell_path = cmd!("which", "git-shell").read()?;
    cmd!("sudo","chsh", &username, "-s", &git_shell_path).run()?;

    debug!("(Re)starting ssh-server");
    cmd!("sudo", "service", "ssh", "restart").run().or_else(|_| {
        cmd!("sudo", "systemctl", "restart" ,"sshd").run()
    }).map_err(|e| e.context("couldn't use either 'service' or 'systemctl' to restart ssh server, wtf are you using?"))?;
    info!("git-server ready!!");

    Ok(user)
}

fn extract_solution_from_git(problem: &Problem, user: &TempUser) -> Result<String, Error> {

    let user_home = cmd!("sudo", "-u", &user.name, "sh", "-c", "echo $HOME").read()?;
    let repo_path = PathBuf::from(user_home).join(&problem.repo_path);
    let out_dir = std::env::temp_dir().join(&problem.repo_path);

    debug!("cloning newly added git repo(via local protocol)");
    cmd!("sudo", "git", "clone", &repo_path, &out_dir).run()?;
    let solution_path = out_dir.join("solution.txt");

    debug!("opening solution file at newly cloned repo");
    let mut content = String::new();
    let mut file = std::fs::File::open(solution_path)?;
    file.read_to_string(&mut content)?;
    info!("successfully extracted the following secret: {}", content);
    Ok(content)
}


#[cfg(not(target_os = "linux"))]
fn setup_git_server(problem: &Problem) -> Result<(), Error> {
    panic!("this challenge requires linux");
}


#[derive(Debug, Serialize)]
struct Solution {
    secret: String
}

fn main() {
    #[cfg(target_os = "windows")] {
        ansi_term::enable_ansi_support();
    }
    let mut builder = pretty_env_logger::formatted_builder().unwrap();
    builder.filter(Some("hosting_git"), log::LevelFilter::Debug)
        .filter(Some("hackattic_common"), log::LevelFilter::Debug);
    builder.init();
    info!("Logger initialized");
    HostingGit::process_challenge().unwrap();
}

struct HostingGit;
impl HackatticChallenge for HostingGit {
    type Problem = Problem;
    type Solution = Solution;

    fn make_solution(problem: &Self::Problem) -> Result<Self::Solution, Error> {
        let temp_user = setup_git_server(problem)?;
        let mut response = push_message(&problem.push_token, "nmdanny.ddns.net")?;
        let response_txt = response.text()?;
        info!("Push response is: {}", response_txt);
        let secret = extract_solution_from_git(&problem, &temp_user)?;
        Ok(Solution {
            secret
        })
    }

    fn challenge_name() -> &'static str {
        "hosting_git"
    }
}
