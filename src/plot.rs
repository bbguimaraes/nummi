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
) -> std::io::Result<()> {
    let mut out = Vec::new();
    let series = DateSeries::new(
        &chrono::NaiveDate::parse_from_str(&v[0].date, "%Y-%m-%d").unwrap());
    let end = chrono::Local::now().naive_local().date();
    let mut sum = dec::Decimal::new(0.0);
    for d in series.take_while(|x| x <= &end) {
        let (y, m) = (d.year(), d.month());
        let date = format!("{}-{:02}", y, m);
        // TODO entries are ordered, implement `group_by`
        let filtered = v.iter().filter(|x| x.date.starts_with(&date));
        let eur_total = db::Entry::total(filtered).iter().fold(
            (dec::Decimal::new(0.0), dec::Decimal::new(0.0)),
            |(acc_pos, acc_neg), (cur, pos, neg)| {
                let conv = to_eur[cur];
                (acc_pos + *pos * conv, acc_neg + *neg * conv)
            },
        );
        let net = eur_total.0 + eur_total.1;
        sum += net;
        write!(
            &mut out,
            "{} {:.2} {:.2} {:.2} {:.2}\n",
            date, eur_total.0, eur_total.1, net, sum,
        )?;
    }
    plot_data(&out)
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
