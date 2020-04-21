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

#[cfg(test)]
mod tests {
    use super::db;
    use super::dec;

    #[test]
    fn parse_csv() -> std::io::Result<()> {
        let mut csv = csv::Reader::from_reader(b"\
Date, USD, JPY, BGN, CZK, DKK, GBP, HUF, PLN, RON, SEK, CHF, ISK, NOK, HRK, RUB, TRY, AUD, BRL, CAD, CNY, HKD, IDR, ILS, INR, KRW, MXN, MYR, NZD, PHP, SGD, THB, ZAR, 
21 April 2020, 1.0837, 116.39, 1.9558, 27.447, 7.4582, 0.88120, 355.02, 4.5291, 4.8373, 10.9543, 1.0517, 157.80, 11.4843, 7.5700, 83.2936, 7.5658, 1.7266, 5.7619, 1.5393, 7.6888, 8.3987, 17001.63, 3.8522, 83.3760, 1335.34, 26.3957, 4.7634, 1.8181, 55.096, 1.5510, 35.269, 20.5853, 
" as &[u8]);
        let mut v = super::parse_csv(&mut csv)?;
        v.sort_by(|l, r| l.partial_cmp(&r).unwrap());
        assert_eq!(v, [
            ([b'a', b'u', b'd'], dec::Decimal::new(1.7266)),
            ([b'b', b'g', b'n'], dec::Decimal::new(1.9558)),
            ([b'b', b'r', b'l'], dec::Decimal::new(5.7619)),
            ([b'c', b'a', b'd'], dec::Decimal::new(1.5393)),
            ([b'c', b'h', b'f'], dec::Decimal::new(1.0517)),
            ([b'c', b'n', b'y'], dec::Decimal::new(7.6888)),
            ([b'c', b'z', b'k'], dec::Decimal::new(27.447)),
            ([b'd', b'k', b'k'], dec::Decimal::new(7.4582)),
            ([b'g', b'b', b'p'], dec::Decimal::new(0.88120)),
            ([b'h', b'k', b'd'], dec::Decimal::new(8.3987)),
            ([b'h', b'r', b'k'], dec::Decimal::new(7.5700)),
            ([b'h', b'u', b'f'], dec::Decimal::new(355.02)),
            ([b'i', b'd', b'r'], dec::Decimal::new(17001.63)),
            ([b'i', b'l', b's'], dec::Decimal::new(3.8522)),
            ([b'i', b'n', b'r'], dec::Decimal::new(83.3760)),
            ([b'i', b's', b'k'], dec::Decimal::new(157.80)),
            ([b'j', b'p', b'y'], dec::Decimal::new(116.39)),
            ([b'k', b'r', b'w'], dec::Decimal::new(1335.34)),
            ([b'm', b'x', b'n'], dec::Decimal::new(26.3957)),
            ([b'm', b'y', b'r'], dec::Decimal::new(4.7634)),
            ([b'n', b'o', b'k'], dec::Decimal::new(11.4843)),
            ([b'n', b'z', b'd'], dec::Decimal::new(1.8181)),
            ([b'p', b'h', b'p'], dec::Decimal::new(55.096)),
            ([b'p', b'l', b'n'], dec::Decimal::new(4.5291)),
            ([b'r', b'o', b'n'], dec::Decimal::new(4.8373)),
            ([b'r', b'u', b'b'], dec::Decimal::new(83.2936)),
            ([b's', b'e', b'k'], dec::Decimal::new(10.9543)),
            ([b's', b'g', b'd'], dec::Decimal::new(1.5510)),
            ([b't', b'h', b'b'], dec::Decimal::new(35.269)),
            ([b't', b'r', b'y'], dec::Decimal::new(7.5658)),
            ([b'u', b's', b'd'], dec::Decimal::new(1.0837)),
            ([b'z', b'a', b'r'], dec::Decimal::new(20.5853)),
        ]
            .iter()
            .copied()
            .map(|(name, to_eur)| db::Currency { name, to_eur })
            .collect::<Vec<_>>());
        Ok(())
    }
}
