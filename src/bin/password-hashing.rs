extern crate hackattic;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate base64;
extern crate hex;
extern crate ring;
extern crate reqwest;
extern crate failure;
extern crate openssl;

use failure::Error;
use hackattic::{from_base64,from_hex,as_hex, make_reqwest_client, HackatticChallenge};

#[derive(Deserialize,Debug,Clone)]
struct Problem {
    password: String,
    #[serde(deserialize_with = "from_base64")]
    salt: Vec<u8>,
    pbkdf2: PBKDF2,
    scrypt: Scrypt
}
impl Problem {
    fn digest_sha256(&self) -> Vec<u8> {
        ring::digest::digest(&ring::digest::SHA256, self.password.as_bytes()).as_ref().to_owned()
    }
    fn hmac(&self) -> Result<Vec<u8>, Error> {
        use ring::hmac;
        let signing_key = hmac::SigningKey::new(&ring::digest::SHA256, &self.salt);
        let hmac = hmac::sign(&signing_key, self.password.as_bytes()).as_ref().to_owned();
        Ok(hmac)
    }
    fn derive_pbkdf2(&self) -> Vec<u8> {
        self.pbkdf2.derive(&self.salt, self.password.as_bytes())
    }
    fn derive_scrypt(&self) -> Result<Vec<u8>, Error> {
        self.scrypt.derive(&self.salt, self.password.as_bytes())

    }
}

#[derive(Deserialize,Debug,Clone)]
struct PBKDF2 {
    rounds: u32,
    hash: Hash
}

impl PBKDF2 {
    fn derive(&self, salt: &[u8], secret: &[u8]) -> Vec<u8> {
        use ring::pbkdf2;
        let digest_alg = self.hash.to_alg();
        let mut out = vec![0u8; digest_alg.output_len];
        pbkdf2::derive(digest_alg, self.rounds, salt, secret, &mut out);
        out
    }
}


#[derive(Deserialize,Debug,Clone)]
struct Scrypt {
    #[serde(rename="N")]
    n: u64,
    r: u64,
    p: u64,
    buflen: usize,
    #[serde(deserialize_with="from_hex")]
    _control: Vec<u8>
}

impl Scrypt {
    fn derive(&self, salt: &[u8], password: &[u8]) -> Result<Vec<u8>, Error> {
        let mut out = vec![0u8; self.buflen];
        let maxmem = 4000000000u64;
        openssl::pkcs5::scrypt(password, salt, self.n, self.r, self.p, maxmem, &mut out)?;
        Ok(out)
    }
}

// Fields are in hex notation(without the 0x)
#[derive(Serialize,Debug,Clone)]
struct Answer {
    #[serde(serialize_with="as_hex")]
    sha256: Vec<u8>,
    #[serde(serialize_with="as_hex")]
    hmac: Vec<u8>,
    #[serde(serialize_with="as_hex")]
    pbkdf2: Vec<u8>,
    #[serde(serialize_with="as_hex")]
    scrypt: Vec<u8>
}

#[derive(Serialize,Deserialize,Debug,Copy,Clone)]
enum Hash {
    #[serde(rename = "sha256")]
    SHA256
}
impl Hash {
    fn to_alg(self) -> &'static ring::digest::Algorithm {
        match self {
            Hash::SHA256 => &ring::digest::SHA256
        }
    }
}

struct PasswordHashing;
impl HackatticChallenge for PasswordHashing {
    type Problem = Problem;
    type Solution = Answer;
    fn challenge_name() -> String {
        "password_hashing".to_owned()
    }
    fn make_solution(req: Problem) -> Result<Answer, Error> {
        let answer = Answer {
            sha256: req.digest_sha256(),
            hmac: req.hmac()?,
            pbkdf2: req.derive_pbkdf2(),
            scrypt: req.derive_scrypt()?
        };
        Ok(answer)
    }
}

fn main() {
    match main_err() {
        Ok(_) => (),
        Err(e) => panic!(format!("{:?}", e))
    }
}

fn main_err() -> Result<(), Error> {
    let mut client = make_reqwest_client()?;
    let problem = PasswordHashing::get_problem(&mut client)?;
    println!("problem is {:?}", problem);
    let solution = PasswordHashing::make_solution(problem)?;
    println!("solution is {:?}", solution);
    let response = PasswordHashing::send_solution(solution, &mut client)?;
    println!("response is {}", response);
    Ok(())
}

#[test]
fn can_scrypt_hash() {
    let password = "rosebud";
    let salt = "pepper";
    let control = hex::decode("b19a18ea8a50a861d08eb94be602f6cbfe67ab98d2021400a3b83fbe3b8ba698").unwrap();
    let scrypt = Scrypt {
        n: 128,
        r: 4,
        p: 8,
        buflen: control.len(),
        _control: Vec::new()
    };
    let derivation = scrypt.derive(salt.as_bytes(), password.as_bytes());
    assert_eq!(&derivation, &control, "Derived Scrypt key isn't equal to control parameter");
}

#[test]
fn can_fetch_and_deserialize_correctly() {
    let req = PasswordHashing::get_problem(&mut make_reqwest_client().unwrap()).unwrap();
    let control_bytes = req.scrypt._control;
    let control_expected = hex::decode("b19a18ea8a50a861d08eb94be602f6cbfe67ab98d2021400a3b83fbe3b8ba698").unwrap();
    assert_eq!(control_bytes, control_expected, "should deserialize the control value correctly");
}