use std::io::Write;

use chrono::Datelike;

use super::db;
use super::dec;

struct DateSeries {
    d: chrono::NaiveDate,
}

impl DateSeries {
    fn new(d: &chrono::NaiveDate) -> DateSeries {
        DateSeries { d: *d }
    }
}

impl Iterator for DateSeries {
    type Item = chrono::NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.d;
        self.d = Self::Item::from_ymd(
            ret.year() + ret.month() as i32 / 12,
            ret.month() % 12u32 + 1,
            1,
        );
        Some(ret)
    }
}

pub fn plot(
    v: &[db::Entry],
    to_eur: &std::collections::HashMap<[u8; 3], dec::Decimal>,
    end: &chrono::NaiveDate,
) -> std::io::Result<()> {
    plot_data(&gen_data(v, to_eur, end)?)
}

fn gen_data(
    v: &[db::Entry],
    to_eur: &std::collections::HashMap<[u8; 3], dec::Decimal>,
    end: &chrono::NaiveDate,
) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let series = DateSeries::new(&v[0].date);
    let mut sum = dec::Decimal::new(0.0);
    for d in series.take_while(|x| x <= &end) {
        // TODO entries are ordered, implement `group_by`
        let filtered = v.iter().filter(|x|
            (x.date.year(), x.date.month()) == (d.year(), d.month()));
        let (pos, neg) = db::Entry::total_with_conversion(filtered, &to_eur);
        let net = pos + neg;
        sum += net;
        write!(
            &mut out,
            "{}-{:02} {:.2} {:.2} {:.2} {:.2}\n",
            d.year(), d.month(), pos, neg, net, sum,
        )?;
    }
    Ok(out)
}

// TODO adjust width
fn plot_data(b: &[u8]) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new("gnuplot")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    let stdin = cmd.stdin.as_mut().unwrap();
    stdin.write_all(b"$d <<EOD\n")?;
    stdin.write_all(b)?;
    stdin.write_all(b"EOD\n")?;
    stdin.write_all(
        br#"
set term png size 4096,1080
set grid
set xtics 3 * 30 * 24 * 60 * 60 rotate
set xdata time
set format x "%Y-%m"
set timefmt "%Y-%m"
w = 15 * 24 * 60 * 60
o(x) = (x + 200 * (x < 0 ? -1 : 1))
plot \
	$d using 1:2:(w)     with boxes  lc "blue"        title "in", \
	$d using 1:(o($2)):2 with labels tc "blue"        notitle, \
	$d using 1:3:(w)     with boxes  lc "red"         title "out", \
	$d using 1:(o($3)):3 with labels tc "red"         notitle, \
	$d using 1:4         with lines  lc "dark-yellow" title "net", \
	$d using 1:4:4       with labels tc "dark-yellow" notitle, \
	$d using 1:5         with lines  lc "dark-green"  title "sum", \
	$d using 1:5:5       with labels tc "dark-green"  notitle
"#,
    )
}

#[cfg(test)]
mod tests {
    use super::DateSeries;

    use super::db;
    use super::dec;

    #[test]
    fn date_series() {
        let s = DateSeries::new(&chrono::NaiveDate::from_ymd(2020, 04, 1));
        let end = chrono::NaiveDate::from_ymd(2021, 04, 1);
        let v = s.take_while(|x| x <= &end).collect::<Vec<_>>();
        assert_eq!(v, vec![
            chrono::NaiveDate::from_ymd(2020, 04, 1),
            chrono::NaiveDate::from_ymd(2020, 05, 1),
            chrono::NaiveDate::from_ymd(2020, 06, 1),
            chrono::NaiveDate::from_ymd(2020, 07, 1),
            chrono::NaiveDate::from_ymd(2020, 08, 1),
            chrono::NaiveDate::from_ymd(2020, 09, 1),
            chrono::NaiveDate::from_ymd(2020, 10, 1),
            chrono::NaiveDate::from_ymd(2020, 11, 1),
            chrono::NaiveDate::from_ymd(2020, 12, 1),
            chrono::NaiveDate::from_ymd(2021, 01, 1),
            chrono::NaiveDate::from_ymd(2021, 02, 1),
            chrono::NaiveDate::from_ymd(2021, 03, 1),
            chrono::NaiveDate::from_ymd(2021, 04, 1),
        ]);
    }

    #[test]
    fn gen_data() -> std::io::Result<()> {
        const EUR: [u8; 3] = [b'e', b'u', b'r'];
        const USD: [u8; 3] = [b'u', b's', b'd'];
        let entries: Vec<db::Entry> = [
            (1, 1, dec::Decimal::new(-100.0), EUR),
            (1, 1, dec::Decimal::new(-200.0), EUR),
            (1, 2, dec::Decimal::new( 300.0), USD),
            (2, 1, dec::Decimal::new(-400.0), USD),
            (3, 1, dec::Decimal::new( 500.0), EUR),
        ].iter().map(|&(m, d, v, c)| db::Entry {
            date: chrono::NaiveDate::from_ymd(2020, m, d),
            value: v,
            currency: c,
            tag: b't',
            text: String::from("description"),
        }).collect();
        let to_eur: std::collections::HashMap<_, _> = [
            (EUR, dec::Decimal::new(1.0)),
            (USD, dec::Decimal::new(3.0)),
        ].iter().copied().collect();
        let ret = super::gen_data(
            &entries,
            &to_eur,
            &chrono::NaiveDate::from_ymd(2020, 4, 1))?;
        assert_eq!(std::str::from_utf8(&ret).unwrap(), "\
2020-01 900.00 -300.00 600.00 600.00
2020-02 0.00 -1200.00 -1200.00 -600.00
2020-03 500.00 0.00 500.00 -100.00
2020-04 0.00 0.00 0.00 -100.00
");
        Ok(())
    }
}
