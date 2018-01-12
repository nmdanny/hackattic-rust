extern crate reqwest;
extern crate openssl;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate ring;
extern crate base64;
extern crate hackattic;
extern crate failure;

use std::fs::File;
use std::io::Write;
use failure::Error;
use std::collections::HashMap;
use openssl::x509::{X509Builder, X509, X509NameBuilder};
use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::pkey::PKey;
use openssl::hash::MessageDigest;
use hackattic::HackatticChallenge;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CertRequirements {
    private_key: String, // base-64 DER encoded(not a PEM although similar)
    required_data: RequiredData
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RequiredData {
    domain: String,
    serial_number: String,
    country: String // long country name
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Answer {
    certificate: String // base-64 DER encoded certificate(not a PEM although similar)
}

fn create_certificate(inputs: &CertRequirements) -> Result<X509, Error> {
    let mut builder = X509Builder::new()?;
    let serial_number = BigNum::from_hex_str(&inputs.required_data.serial_number[2..])?.to_asn1_integer()?;
    builder.set_serial_number(&serial_number)?;
    let now = Asn1Time::days_from_now(0)?;
    let expiry = Asn1Time::days_from_now(10)?;
    builder.set_not_before(&now)?;
    builder.set_not_after(&expiry)?;
    let mut subject_builder = X509NameBuilder::new()?;
    subject_builder.append_entry_by_text("CN", &inputs.required_data.domain)?;
    subject_builder.append_entry_by_text("C", &country_name_to_code(&inputs.required_data.country).unwrap())?;
    let name = subject_builder.build();
    builder.set_subject_name(&name)?;
    builder.set_issuer_name(&name)?;
    let private_key = get_private_key_from_string(&inputs.private_key)?;
    builder.set_pubkey(&private_key)?; // the private part of the X509's pubkey won't be exposed in the X509 once we serialize it to DER/PEM
    builder.sign(&private_key, MessageDigest::sha256())?;
    let x509 = builder.build();
    Ok(x509)
}

fn country_name_to_code(name: &str) -> Option<String> {
    if name.len() == 2 {
        return Some(name.to_uppercase());
    }
    let mappings = vec![
        ("Cocos Islands", "CC"),
        ("Christmas Island", "CX"),
        ("Tokelau Islands", "TK"),
        ("Sint Maarten", "TL"),
        ("Keeling Islands", "CC")
    ].into_iter().collect::<HashMap<_,_>>();
    mappings.get(name).map(|n| n.to_string())
}

fn dump_cert_to_file(cert: &X509) -> Result<(), Error> {
    let der = cert.to_der()?;
    let mut der_file = File::create("cert.der")?;
    der_file.write_all(&der)?;
    Ok(())
}

fn get_private_key_from_string(string: &str) -> Result<PKey, Error> {
    let private_key = if string.starts_with("-----BEGIN RSA PRIVATE KEY-----") {
        PKey::private_key_from_pem(string.as_bytes())?
    } else {
        let der = base64::decode(string)?;
        PKey::private_key_from_der(&der)?
    };
    Ok(private_key)
}



fn x509_to_answer(cert: X509) -> Result<Answer, Error> {
    let der = cert.to_der()?;
    let b64enc = base64::encode(&der);
    Ok(Answer {
        certificate: b64enc
    })
}

fn main() {
    TalesOfSsl::process_challenge().unwrap();
}

struct TalesOfSsl;
impl HackatticChallenge for TalesOfSsl {
    type Problem = CertRequirements;
    type Solution = Answer;
    fn challenge_name() -> &'static str {
        "tales_of_ssl"
    }
    fn make_solution(req: &CertRequirements) -> Result<Answer, ::failure::Error> {
        let x509 = create_certificate(&req)?;
        dump_cert_to_file(&x509)?;
        let answer = x509_to_answer(x509)?;
        Ok(answer)
    }
}

#[test]
fn can_make_cert() {
    let private_key = "-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAp/tNplkj/QorFgYhCtan1lPSQ6DEHlA3GGcYsAphvgBVoYDa
JLdbD63Y3KLjSXynZ2tdtrWS+Nm7jk/3GCU3Dt5x+Wn7LZ6vvbg2mbJWDlldCePS
oLLu6vdes25qdF5aNpkMVnu6jD+v4M9r4rJcTaZGaBvJwn62nh9VowV85xlk/b11
ZhucO5bx93ENa9yyllZkigDoBWGHG/CtI8v4ZUKh4XonAtCYUqb8Har7fbJ+fM/E
IlhpaC5mNfrjwvpzVuHMJX9N3aGQkkTI6lGYJfXGC1ylA5X9A+z2b6f6gmZRjOaT
pXLzRo4OC509uQbzUdtD3cHuNZf0SphOF1l4xQIDAQABAoIBAFPTVCxuz0a2jEmx
RUvjUl3h9ktJLpTx8ox65hbzF1N5V0IAytsHpKluV4nCvyksy10QdvN2KKGGBdI+
lU+3QoJo/vc3q+BYD9rc0mJgXBFNBuCoj7Mhl5gpWVixAO6RJvOX7FA77huOrHXD
DFNULjuRfhD1zPkBwp+lg1aWpn2rnxaOMxXbZoIffWrYO5QsUcb8JZlEeuckJSwf
6bKyfhzRN4cSz6Gdpure2VmNrUuiR82MQUuJwK4NcCpDxs9aGmYJnRqnVsiIySYa
YF1SopgnQza7m0vVxyFy7kzGHJJ84EYTjJzmhFwNPhiwMdi8YG6AYVk5eNhDvZ2h
AT+cvQECgYEA3QeNy4C6DXePKXgE7VU2p78wUYan6yHToUxBV/N2mYxNVWXwmj3u
0fvbLUWfoa+Dy6ZgUy4kGoI1ri6ibhutE3Eff3xWpOsxVFwKu2j+V2D+ZplG5Gto
l3LGv9+BX0thaQkOlhRg50Y0RsZgpE+bBgzOdvYv7jhCiqx0aRycLN8CgYEAwo8h
j5AZ8jq3IRcMo3+KsUPAhxKMbkUAhn99EGi5LyJ7ZtgjLCu6obNul8u3If+oSMG8
+gkosKc9HcrSUG5vHUQpefdG1SjMRt3Gxu6j6UOi/oWh4ppMTxd1PpTAFULX7H/u
Y2MUytqjt/G2QStwfTdESPJhw7u7cN8HGNNtqtsCgYADmvTwJdhjEdku9vs1l9c9
Yzv5iHXCuxmFnwXN1nXPyV8VjoUfLvVvTWlk0qbo70D6GGunz6/dEtSnU7FolGD9
WTIHVVLge8mhM6MlLXBAop9jswpZ/XqGReQCCzZEBKnBGdm4DvsJqrZ9lQzMgVPp
BFp1zEw52YcUVf3MHSBbBwKBgCUJkl3+FLJkMxB9js5hLUnpl/EeutUTFbKE+o3x
Ia+zZMKDSs7R1EmMGvWStl4miSawmwUOUUyvyZauUbM2ErkDjNHHHRjfF2Q2O+0K
6PEzCwhg8BxvOy+jS5KKRrbFbs163SrWZdLoJFqUDRoC5vsvVjR8z9evGVO3YWZ8
eVgZAoGAEVPTxuyWpd6jaj41TvRLNaA6dVqi/ZCgUrW7eVAwEN3j6fTYzbWfQ8uy
CfgolQnmCeNmT524AUunKMkuhi8S9zttuJ6thonY9euECVDyIEzGGxmqLbYhXuI7
jyA9qXqpWbpYDVmP6NaKewg8xWqPusJXzjeP8m0rIh2Huu8WdNs=
-----END RSA PRIVATE KEY-----
".to_owned();
    let req = CertRequirements {
        private_key,
        required_data: RequiredData {
            domain: "hello-there.com".to_owned(),
            serial_number: "0xdeadbeef".to_owned(),
            country: "Tokelau Islands".to_owned()
        }
    };
    let cert = create_certificate(&req);
    assert!(cert.is_ok(), "Failed to create certificate");
}