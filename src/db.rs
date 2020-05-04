use std::convert::TryFrom;
use std::io::prelude::*;

use super::dec;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Currency {
    pub name: [u8; 3],
    pub to_eur: dec::Decimal,
}

impl Currency {
    pub fn name_str(&self) -> &str {
        std::str::from_utf8(&self.name).unwrap()
    }
}

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub date: String,
    pub value: dec::Decimal,
    pub currency: [u8; 3],
    pub tag: u8,
    pub text: String,
}

impl Entry {
    // TODO more detailed errors
    pub fn from_line(l: &str) -> Entry {
        let mut fields = l.split(' ');
        let date = fields.next().unwrap();
        let value = fields.next().unwrap();
        let mut currency = value[value.len() - 3..].bytes();
        let tag = fields.next().unwrap().bytes().nth(0).unwrap();
        Entry {
            date: String::from(date),
            value: dec::Decimal::try_from(&value[..value.len() - 3])
                .expect("invalid decimal in entry"),
            currency: [
                currency.next().unwrap(),
                currency.next().unwrap(),
                currency.next().unwrap(),
            ],
            tag,
            text: String::from(&l[date.len() + value.len() + 4..]),
        }
    }

    pub fn to_line(&self) -> String {
        format!(
            "{} {:.2}{} {} {}",
            self.date,
            self.value,
            std::str::from_utf8(&self.currency).unwrap(),
            self.tag as char,
            self.text,
        )
    }

    pub fn unique_currencies(v: &[Entry]) -> Vec<[u8; 3]> {
        v.iter()
            .map(|x| x.currency)
            .collect::<std::collections::HashSet<_>>()
            .iter()
            .copied()
            .collect()
    }

    pub fn total<'a>(
        it: impl Iterator<Item = &'a Entry>,
    ) -> Vec<([u8; 3], dec::Decimal, dec::Decimal)> {
        let mut ret = std::collections::HashMap::new();
        for x in it {
            let (pos, neg) = ret
                .entry(x.currency)
                .or_insert((dec::Decimal::new(0.0), dec::Decimal::new(0.0)));
            if x.value < dec::Decimal::new(0.0) {
                *neg += x.value
            } else {
                *pos += x.value
            }
        }
        ret.iter().map(|(k, v)| (*k, v.0, v.1)).collect()
    }

    pub fn total_with_conversion<'a>(
        it: impl Iterator<Item = &'a Entry>,
        conv: &std::collections::HashMap<[u8; 3], dec::Decimal>,
    ) -> (dec::Decimal, dec::Decimal) {
        Entry::total(it).iter().fold(
            (dec::Decimal::new(0.0), dec::Decimal::new(0.0)),
            |(pos, neg), (cur, p, n)| {
                let c = conv[cur];
                (pos + *p * c, neg + *n * c)
            },
        )
    }

    pub fn read_db(path: &std::path::Path) -> std::io::Result<Vec<Entry>> {
        DBIterator::new(path)?.collect()
    }
}

struct Find {
    stack: Vec<std::path::PathBuf>,
}

impl Find {
    pub fn new(dir: &std::path::Path) -> Find {
        Find {
            stack: vec![std::path::PathBuf::from(dir)],
        }
    }

    fn read_dir(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        std::fs::read_dir(path)?
            .map(|x| x.and_then(|x| Ok(self.stack.push(x.path()))))
            .collect()
    }
}

impl Iterator for Find {
    type Item = std::io::Result<std::path::PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.stack.pop() {
            if x.is_file() {
                return Some(Ok(x));
            }
            if let Err(e) = self.read_dir(&x) {
                return Some(Err(e));
            }
        }
        None
    }
}

#[derive(Debug)]
struct DBIterator {
    files: Vec<std::path::PathBuf>,
    file_it: Option<FileIterator>,
}

impl DBIterator {
    fn new(path: &std::path::Path) -> std::io::Result<DBIterator> {
        let mut files = Find::new(path)
            .collect::<std::io::Result<Vec<std::path::PathBuf>>>()?;
        files.retain(|x| x.extension().unwrap_or_default() == "txt");
        files.sort();
        files.reverse();
        Ok(DBIterator {
            files,
            file_it: None,
        })
    }
}

impl Iterator for DBIterator {
    type Item = std::io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.file_it.is_none() {
                match self.files.pop() {
                    None => return None,
                    Some(x) => match FileIterator::new(&x) {
                        Ok(x) => self.file_it = Some(x),
                        Err(e) => return Some(Err(e)),
                    },
                }
            }
            if let Some(x) = self.file_it.as_mut().unwrap().next() {
                return Some(x);
            }
            self.file_it = None;
        }
    }
}

#[derive(Debug)]
struct FileIterator {
    lines: std::io::Lines<std::io::BufReader<std::fs::File>>,
}

impl FileIterator {
    fn new(path: &std::path::Path) -> std::io::Result<FileIterator> {
        Ok(FileIterator {
            lines: std::io::BufReader::new(std::fs::File::open(path)?).lines(),
        })
    }
}

impl Iterator for FileIterator {
    type Item = std::io::Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lines.next() {
            None => None,
            Some(x) => match x {
                Err(e) => Some(Err(e)),
                Ok(x) if x == "" => None,
                Ok(x) => Some(Ok(Entry::from_line(&x))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Entry;
    use super::dec;

    const EUR: [u8; 3] = [b'e', b'u', b'r'];
    const USD: [u8; 3] = [b'u', b's', b'd'];
    const GBP: [u8; 3] = [b'g', b'b', b'p'];

    #[test]
    fn from_line() {
        let e = Entry::from_line(
            "2020-04-20 -100.00eur t description");
        assert_eq!(e.date, "2020-04-20");
        assert_eq!(e.value, super::dec::Decimal::new(-100.0));
        assert_eq!(e.currency, EUR);
        assert_eq!(e.tag, b't');
        assert_eq!(e.text, "description");
    }

    #[test]
    fn to_line() {
        let e = Entry {
            date: String::from("2020-04-20"),
            value: super::dec::Decimal::new(-100.0),
            currency: EUR,
            tag: b't',
            text: String::from("description"),
        };
        assert_eq!(e.to_line(), "2020-04-20 -100.00eur t description");
    }

    #[test]
    fn unique_currencies() {
        let mut ret = Entry::unique_currencies(
            &[EUR, GBP, EUR, GBP, USD]
                .iter()
                .map(|&c| Entry {
                    date: String::from("2020-04-20"),
                    value: super::dec::Decimal::new(0.0),
                    currency: c,
                    tag: b't',
                    text: String::from("description"),
                })
                .collect::<Vec<Entry>>());
        ret.sort();
        assert_eq!(ret, vec![EUR, GBP, USD]);
    }

    #[test]
    fn total() {
        let v: Vec<Entry> = [
            (dec::Decimal::new(-100.0), EUR),
            (dec::Decimal::new(-200.0), EUR),
            (dec::Decimal::new( 300.0), USD),
            (dec::Decimal::new(-400.0), USD),
            (dec::Decimal::new( 500.0), EUR),
        ].iter().map(|&(v, c)| Entry {
            date: String::from("2020-04-20"),
            value: v,
            currency: c,
            tag: b't',
            text: String::from("description"),
        }).collect();
        let mut total = Entry::total(v.iter());
        total.sort_by(|l, r| l.0.cmp(&r.0));
        assert_eq!(total, vec![(
            EUR,
            dec::Decimal::new(500.0),
            dec::Decimal::new(-300.0),
        ), (
            USD,
            dec::Decimal::new(300.0),
            dec::Decimal::new(-400.0),
        )]);
    }

    #[test]
    fn total_with_conversion() {
        let v: Vec<Entry> = [
            (dec::Decimal::new(-100.0), EUR),
            (dec::Decimal::new(-200.0), EUR),
            (dec::Decimal::new( 300.0), USD),
            (dec::Decimal::new(-400.0), USD),
            (dec::Decimal::new( 500.0), EUR),
        ].iter().map(|&(v, c)| Entry {
            date: String::from("2020-04-20"),
            value: v,
            currency: c,
            tag: b't',
            text: String::from("description"),
        }).collect();
        let conv = [
            (EUR, dec::Decimal::new(1.0)),
            (USD, dec::Decimal::new(3.0)),
        ].iter().copied().collect::<std::collections::HashMap<_, _>>();
        assert_eq!(Entry::total_with_conversion(v.iter(), &conv), (
            dec::Decimal::new(1400.0),
            dec::Decimal::new(-1500.0),
        ));
    }
}
