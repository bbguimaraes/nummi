use std::convert::TryFrom;
use std::io::prelude::*;

use super::db;
use super::dec;
use super::PROG_NAME;

const CURRENCIES_MAX_AGE: u64 = 24 * 60 * 60;

pub struct Cache {
    pub currencies: Vec<db::Currency>,
}

pub fn dir() -> std::path::PathBuf {
    std::env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME not set");
            std::path::PathBuf::from(home).join(".cache")
        })
        .join(PROG_NAME)
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            currencies: Vec::new(),
        }
    }

    pub fn read_currencies(
        &mut self,
        dir: &std::path::Path,
        force: bool,
        fetch: impl Fn() -> std::io::Result<Vec<db::Currency>>,
    ) -> std::io::Result<()> {
        let path = dir.join("currencies");
        if force || cache_stale(&path, CURRENCIES_MAX_AGE)? {
            update_currencies(&path, &fetch()?)?;
        }
        Ok(self.currencies = self::read_currencies(&path)?)
    }
}

fn cache_stale(path: &std::path::Path, max_age: u64) -> std::io::Result<bool> {
    match std::fs::metadata(&path) {
        Ok(meta) => Ok(std::time::SystemTime::now()
            .duration_since(meta.modified().unwrap())
            .unwrap()
            > std::time::Duration::new(max_age, 0)),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(true),
            _ => Err(e),
        },
    }
}

fn update_currencies(
    path: &std::path::Path,
    v: &Vec<db::Currency>,
) -> std::io::Result<()> {
    create_dir(path.parent().unwrap())?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)?;
    v.iter()
        .map(|x| {
            f.write_all(x.name_str().as_bytes())?;
            f.write_all(b" ")?;
            f.write_all(x.to_eur.to_string().as_bytes())?;
            f.write_all(b"\n")
        })
        .collect()
}

fn read_currencies(
    path: &std::path::Path,
) -> std::io::Result<Vec<db::Currency>> {
    let s = std::fs::read_to_string(&path)?;
    let mut ret = Vec::new();
    for line in s.lines() {
        let mut it = line.split(" ");
        let mut name = it.next().unwrap_or_default().bytes();
        let to_eur = it.next().unwrap_or_default();
        ret.push(db::Currency {
            name: [
                name.next().unwrap(),
                name.next().unwrap(),
                name.next().unwrap(),
            ],
            to_eur: dec::Decimal::try_from(to_eur)
                .expect("invalid decimal in currency file"),
        });
    }
    ret.push(db::Currency {
        name: [b'e', b'u', b'r'],
        to_eur: dec::Decimal::new(1.0),
    });
    Ok(ret)
}

fn create_dir(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::metadata(&path).and(Ok(())).or_else(|e| {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e);
        }
        std::fs::create_dir(&path)
    })
}
