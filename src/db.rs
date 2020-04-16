use std::convert::TryFrom;

use super::dec;

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

    pub fn read_db_file(
        r: &mut impl std::io::Read,
        v: &mut Vec<Entry>,
    ) -> std::io::Result<()> {
        let mut s = String::new();
        r.read_to_string(&mut s)?;
        Ok(s.lines()
            .take_while(|&x| x != "")
            .map(Entry::from_line)
            .for_each(|x| v.push(x)))
    }

    pub fn read_db(path: &std::path::Path) -> std::io::Result<Vec<Entry>> {
        use std::io::Result;
        let mut ret = Vec::new();
        let mut files =
            Find::new(path).collect::<Result<Vec<std::path::PathBuf>>>()?;
        files.sort();
        files
            .iter()
            .filter(|x| x.extension().unwrap_or_default() == "txt")
            .map(|x|
                 Entry::read_db_file(
                     &mut std::fs::File::open(x)?, &mut ret))
            .collect::<Result<()>>()
            .and(Ok(ret))
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
            if !x.is_dir() {
                return Some(Ok(x));
            }
            if let Err(e) = self.read_dir(&x) {
                return Some(Err(e));
            }
        }
        None
    }
}
