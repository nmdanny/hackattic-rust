extern crate hackattic;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate failure;
extern crate tempfile;
#[macro_use]
extern crate log;
extern crate ansi_term;
extern crate env_logger;
extern crate pretty_env_logger;
extern crate futures;
extern crate futures_cpupool;
extern crate trust_dns_server;
extern crate trust_dns_proto;
extern crate tokio_core;
extern crate base64;


use std::collections::BTreeMap;
use std::str::FromStr;
use std::net::{UdpSocket, Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use serde::{Deserialize, Deserializer};
use hackattic::{HackatticChallenge, make_reqwest_client};
use failure::{Error, ResultExt};
use trust_dns_proto::rr::{Record, RecordType, RData, Name};
use trust_dns_proto::rr::rdata::{TXT, NULL, SOA};
use trust_dns_proto::serialize::binary::{BinSerializable, BinEncoder};
use trust_dns_server::ServerFuture;
use trust_dns_server::authority::{Authority, Catalog, ZoneType};
use futures::Future;


#[derive(Deserialize, Debug, Clone)]
struct Problem {
    records: Vec<ProblemRecord>
}


#[derive(Deserialize, Debug, Clone)]
struct ProblemRecord {
    name: String,
    #[serde(rename = "type")]
    _type: String,
    data: String
}

// TrustDNS doesn't support RP, so we manually handle RData creation here
// see https://tools.ietf.org/html/rfc1183#section-2.2
// consists of the following fields, both required
fn make_rp_rdata(mailbox: &str) -> RData {
    // using mailbox encoding (see rname @ create_soa)
    let mbox_dname = Name::from_str(mailbox).unwrap();
    // associated TXT RR. Use root when none is available
    let txt_dname = Name::root();
    let mut bytes = Vec::new();
    {
        let mut encoder = BinEncoder::new(&mut bytes);
        mbox_dname.emit(&mut encoder).unwrap();
        txt_dname.emit(&mut encoder).unwrap();
    }
    RData::Unknown {
        code: 17, rdata: NULL::with(bytes)
    }

}

// This is an ugly hack, because TrustDNS doesn't support wildcard records
// https://github.com/bluejekyll/trust-dns/issues/30
// Hackattic will sometimes ask for "extra.label.label" under the wildcard, so we
// can simply replace "*" with "extra".
// If you get an error "we asked for woot/w00t/etc, simply re-run this challenge
// until it asks for "extra" and it'll pass.
fn patch_wildcard_name_for_challenge(name: &mut Name) {
    let mut st = name.to_string();
    if st.contains("*") {
        st = st.replace("*","extra");
        *name = Name::from_str(&st).unwrap();
        error!("Found wildcard, patched with extra, name is now {:?}", name);
    }
}

impl ProblemRecord {
    fn as_record(&self) -> Result<Record, Error>{
        let _type = if &self._type == "RP" {
            RecordType::Unknown(17)
        } else {
            RecordType::from_str(&self._type).unwrap()
        };
        let mut name = Name::from_str(&self.name).unwrap();
        patch_wildcard_name_for_challenge(&mut name);
        let rdata = match _type {
            RecordType::A => RData::A(Ipv4Addr::from_str(&self.data)?),
            RecordType::AAAA => RData::AAAA(Ipv6Addr::from_str(&self.data)?),
            RecordType::TXT => RData::TXT(TXT::new(vec![self.data.clone()])),
            RecordType::Unknown(17) => make_rp_rdata(&self.data),
            e => Err(format_err!("Unsupported RecordType {:?}", e))?
        };
        Ok(Record::from_rdata(name, 3600, _type, rdata))
    }
}
#[derive(Debug, Clone, Serialize)]
struct Solution {
    dns_ip: String,
    dns_port: u32
}

fn main() {
    ansi_term::enable_ansi_support().unwrap();
    let mut log_builder = pretty_env_logger::init();
    info!("Logger initialized");
    DnsServer::process_challenge().unwrap()
}

fn create_soa(name: Name) -> Result<Record, Error>{
    let _type = RecordType::SOA;
    let rdata = RData::SOA(SOA::new(
        name.clone(),
        Name::from_str("nmdanny@gmail.com").unwrap(),
        0,
        5,
        1,
        600000,
        10
    ));
    Ok(Record::from_rdata(name, 3600, _type, rdata))
}

fn records_to_catalog(records: &[Record]) -> Result<Catalog, Error> {
    let origin = records[0].name();
    let record_map = BTreeMap::new();
    let mut authority = Authority::new(origin.clone(), record_map , ZoneType::Master, false, false);
    authority.update_records(records, true).map_err(|e| format_err!(
        "Couldn't update records: {:?}", e
    ))?;
    let mut catalog = Catalog::new();
    catalog.upsert(origin.clone(), authority);
    Ok(catalog)
}

struct DnsServer;
impl HackatticChallenge for DnsServer {
    type Problem = Problem;
    type Solution = Solution;
    fn challenge_name() -> &'static str {
        "serving_dns"
    }
    fn make_solution(req: &Problem) -> Result<Solution, Error> {
        let mut records = req.records.iter().map(ProblemRecord::as_record).collect::<Result<Vec<_>,_>>()?;
        let soa = create_soa(records[0].name().clone())?;
        records.push(soa);
        let catalog = records_to_catalog(&records)?;
        let mut server = ServerFuture::new(catalog)?;
        let udp = std::net::UdpSocket::bind("0.0.0.0:13801")?;
        let tcp = std::net::TcpListener::bind("0.0.0.0:13801")?;
        server.register_socket(udp);
        server.register_listener(tcp, Duration::from_secs(30));
        let solution = Solution {
            dns_port: 13801,
            dns_ip: "37.142.197.254".to_owned()
        };
        let mut core = server.tokio_core();
        let mut client = make_reqwest_client()?;

        core.handle().spawn_fn(move || {
            std::thread::spawn(move || {
                let res = DnsServer::send_solution(&solution, &mut client).unwrap();
                error!("Got response {:?}", res);
            });
            Ok(())
        });

        loop {
            core.turn(None);
        }
        unreachable!("never end")
    }
}
