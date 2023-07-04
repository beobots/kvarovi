use anyhow::{Ok, Result};
use csv::{Reader, StringRecord};
use serde::Deserialize;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
struct Record {
    street_name: String,
    old_street_name: Option<String>,
    municipality: String,
    settlement: String,
    settlement_part: String,
    si_list: Option<String>,
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"StaticStreet {{street_name: "{}", old_street_name: {}, municipality: "{}", settlement: "{}", settlement_part: "{}", si_list: {}, }},"#,
            self.street_name,
            if let Some(ref it) = self.old_street_name {
                format!(r#"Some("{it}")"#)
            } else {
                "None".to_string()
            },
            self.municipality,
            self.settlement,
            self.settlement_part,
            if let Some(ref it) = self.si_list {
                format!(r#"Some("{it}")"#)
            } else {
                "None".to_string()
            }
        )
    }
}

fn write_records<Out>(mut out: Out, iter: impl Iterator<Item = Record>) -> Result<()>
where
    Out: Write,
{
    write!(out, "&[")?;
    for item in iter {
        write!(&mut out, "{item}")?;
    }
    write!(&mut out, "]")?;
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=beograd_streets.csv");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is defined");
    let data_file = Path::new(&out_dir).join("streets_data.rs");
    let out = BufWriter::new(File::create(data_file)?);

    let mut rdr = Reader::from_path("beograd_streets.csv").expect("csv file not found");
    rdr.set_headers(StringRecord::from(vec![
        "street_name",
        "old_street_name",
        "municipality",
        "settlement",
        "settlement_part",
        "si_list",
    ]));

    let records = rdr.into_deserialize().filter_map::<Record, _>(Result::ok);
    write_records(out, records)?;

    Ok(())
}
