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
        Ok(self.currencies =
           self::read_currencies(&mut std::fs::File::open(&path)?)?)
    }
}

fn cache_stale(path: &std::path::Path, max_age: u64) -> std::io::Result<bool> {
    match std::fs::metadata(&path) {
        Ok(meta) => Ok(std::time::SystemTime::now()
            .duration_since(meta.modified().unwrap())
            .unwrap()
            > std::time::Duration::new(max_age, 0)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(e) => Err(e),
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
        .map(|x| write!(f, "{} {}\n", x.name_str(), x.to_eur.to_string()))
        .collect()
}

fn read_currencies(
    r: impl std::io::Read,
) -> std::io::Result<Vec<db::Currency>> {
    let mut ret = Vec::new();
    for line in std::io::BufReader::new(r).lines() {
        let line = line?;
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

#[cfg(test)]
mod tests {
    use super::db;
    use super::dec;

    #[test]
    fn read_currencies() -> std::io::Result<()> {
        let mut ret = super::read_currencies(b"\
aud 1.7266\nbgn 1.9558\nbrl 5.7619\ncad 1.5393\nchf 1.0517\ncny 7.6888
czk 27.447\ndkk 7.4582\ngbp 0.88120\nhkd 8.3987\nhrk 7.5700\nhuf 355.02
idr 17001.63\nils 3.8522\ninr 83.3760\nisk 157.80\njpy 116.39\nkrw 1335.34
mxn 26.3957\nmyr 4.7634\nnok 11.4843\nnzd 1.8181\nphp 55.096\npln 4.5291
ron 4.8373\nrub 83.2936\nsek 10.9543\nsgd 1.5510\nthb 35.269\ntry 7.5658
usd 1.0837\nzar 20.5853\n" as &[u8])?;
        ret.sort_by(|l, r| l.partial_cmp(&r).unwrap());
        assert_eq!(ret, [
            ([b'a', b'u', b'd'], dec::Decimal::new(1.7266)),
            ([b'b', b'g', b'n'], dec::Decimal::new(1.9558)),
            ([b'b', b'r', b'l'], dec::Decimal::new(5.7619)),
            ([b'c', b'a', b'd'], dec::Decimal::new(1.5393)),
            ([b'c', b'h', b'f'], dec::Decimal::new(1.0517)),
            ([b'c', b'n', b'y'], dec::Decimal::new(7.6888)),
            ([b'c', b'z', b'k'], dec::Decimal::new(27.447)),
            ([b'd', b'k', b'k'], dec::Decimal::new(7.4582)),
            ([b'e', b'u', b'r'], dec::Decimal::new(1.0)),
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
