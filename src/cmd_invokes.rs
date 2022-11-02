use clap::Parser;
use std::io::stdout;
use stellar_xdr::{ReadXdr, ScVal};

use crate::horizon::Response;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Cmd {}

impl Cmd {
    pub fn run(&self) {
        let mut next = "https://horizon-futurenet.stellar.org/operations?order=desc".to_string();
        loop {
            let backoff = backoff::ExponentialBackoff::default();
            let op = || ureq::get(&next).call().map_err(backoff::Error::transient);
            let resp = backoff::retry(backoff, op);
            let resp = resp.unwrap().into_json::<Response>().unwrap();

            let next_next = resp.links.next.href;
            if next_next == next {
                break;
            }
            next = next_next;

            let records = resp
                .embedded
                .records
                .iter()
                .filter(|r| r.r#type == "invoke_host_function");

            let mut wtr = csv::WriterBuilder::new()
                .flexible(true)
                .from_writer(stdout());
            for r in records {
                let mut cols = vec![r.function.clone().unwrap_or_default()];
                cols.extend(
                    r.parameters
                        .iter()
                        .map(|p| ScVal::from_xdr_base64(&p.value))
                        .map(|v| format!("{v:?}")),
                );
                wtr.write_record(cols).unwrap();
            }
            wtr.flush().unwrap();
        }
    }
}
