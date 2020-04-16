use std::convert::TryFrom;

use super::db;
use super::dec;

const EUR_SERVICE_URL: &'static str =
    "https://www.ecb.europa.eu/stats/eurofxref/eurofxref.zip";

pub fn fetch_currencies() -> std::io::Result<Vec<db::Currency>> {
    let resp = reqwest::blocking::get(EUR_SERVICE_URL)
        .unwrap()
        .bytes()
        .unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(resp))?;
    let mut csv = csv::Reader::from_reader(zip.by_name("eurofxref.csv")?);
    parse_csv(&mut csv)
}

fn parse_csv<T: std::io::Read>(
    csv: &mut csv::Reader<T>,
) -> std::io::Result<Vec<db::Currency>> {
    let headers: Vec<String> =
        csv.headers()?.iter().map(String::from).collect();
    let record = csv.records().next().unwrap().unwrap_or_default();
    Ok(headers
        .iter()
        .map(|x| x.trim().to_lowercase())
        .zip(record.iter())
        .filter(|(k, _)| !k.is_empty() && k != "date" )
        .map(|(k, v)| {
            let mut k = k.bytes();
            db::Currency {
                name: [
                    k.next().unwrap(),
                    k.next().unwrap(),
                    k.next().unwrap(),
                ],
                to_eur: dec::Decimal::try_from(v.trim())
                    .expect("invalid decimal from currency service"),
            }
        })
        .collect())
}
