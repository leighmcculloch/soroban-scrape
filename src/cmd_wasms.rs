use clap::Parser;
use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
};
use stellar_xdr::{ReadXdr, ScObject, ScVal};

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

            let records = resp.embedded.records.iter().filter(|r| {
                r.r#type == "invoke_host_function"
                    && r.function.as_deref()
                        == Some("HostFunctionHostFnCreateContractWithSourceAccount")
            });

            create_dir_all("wasms").unwrap();

            for r in records {
                if let Some(code) = r.parameters.get(0) {
                    if let Ok(ScVal::Object(Some(ScObject::Bytes(bytes)))) =
                        ScVal::from_xdr_base64(&code.value)
                    {
                        let bytes = bytes.as_slice();
                        let hash = sha256::digest(bytes);

                        let mut path = Path::new("wasms").join(hash);
                        path.set_extension("wasm");
                        if !path.exists() {
                            println!("Writing: {}", path.display());
                            let mut f = File::create(path).unwrap();
                            f.write_all(bytes).unwrap();
                            f.flush().unwrap();
                        }
                    }
                }
            }

            // use soroban_spec::gen::rust::{generate_from_wasm, ToFormattedString};
            // let wasm = b"";
            // let wasm = base64::decode(wasm).unwrap();
            // let gen = generate_from_wasm(&wasm, "contract.wsm", None).unwrap();
            // let fmt = gen.to_formatted_string().unwrap();
            // println!("{fmt}");
        }
    }
}
