extern crate hackattic;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate failure;
extern crate postgres;
extern crate tempfile;
extern crate flate2;

use hackattic::{HackatticChallenge, from_base64};
use failure::{Error, ResultExt};
use std::io::prelude::*;
use std::process::{Command, Child, ChildStdin, ChildStdout, Stdio};
use std::mem::ManuallyDrop;
use postgres::{TlsMode, Connection};

#[derive(Deserialize, Debug, Clone)]
struct Problem {
    #[serde(deserialize_with = "from_base64")]
    dump: Vec<u8>
}

#[derive(Debug, Clone, Serialize)]
struct Solution {
    alive_ssns: Vec<String>
}

fn main() {
    BackupRestore::process_challenge().unwrap();
}

/// Wraps a `Connection` to a temporary database that
/// will be automatically deleted once dropped.
pub struct TempDb {
    db_name: String,
    connection: ManuallyDrop<Connection>
}
impl TempDb {
    pub fn new(dump: &[u8]) -> Result<Self, Error> {
        let dump = {
            let mut decoder = flate2::read::GzDecoder::new(dump);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf)
                .context("It's possible that this dump wasn't encoded via GZip, try running this challenge again")?;
            buf
        };
        let db_name = "hackattic_backup_restore_challenge".to_owned();
        let create_db = Command::new("createdb")
            .args(&["-U", "postgres", &db_name])
            .env("PGPASSWORD", "postgres")
            .status()?;
        let psql = Command::new("psql")
            .args(&["-U", "postgres", "-d", &db_name])
            .env("PGPASSWORD", "postgres")
            .stdin(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = psql.stdin.ok_or(format_err!("Failed to get stdin from new psql process"))?;
            stdin.write_all(&dump)?;
        }
        let conn_string = format!("postgres://postgres:postgres@localhost:5432/{}", db_name);
        let connection = Connection::connect(conn_string, TlsMode::None)?;
        Ok(TempDb {
            db_name, connection: ManuallyDrop::new(connection)
        })
    }
    fn find_ssns(&mut self) -> Result<Vec<String>, Error> {
        let results = self.query("SELECT ssn FROM criminal_records where status = 'alive'", &[])?
            .iter()
            .map(|row| row.get(0))
            .collect();
        Ok(results)
    }
}
impl Drop for TempDb {
    fn drop(&mut self) {
        unsafe {
            // we must drop the connection before we can drop the database
            std::mem::ManuallyDrop::drop(&mut self.connection);
        }
        Command::new("dropdb")
            .args(&["-U", "postgres", &self.db_name])
            .env("PGPASSWORD", "postgres")
            .status().unwrap();

    }
}

impl ::std::ops::Deref for TempDb {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        self.connection.deref()
    }
}

struct BackupRestore;
impl HackatticChallenge for BackupRestore {
    type Problem = Problem;
    type Solution = Solution;
    fn challenge_name() -> &'static str {
        "backup_restore"
    }
    fn make_solution(req: &Problem) -> Result<Solution, Error> {
        let mut db = TempDb::new(&req.dump)?;
        let ssns = db.find_ssns()?;
        Ok(Solution {
            alive_ssns: ssns
        })
    }
}