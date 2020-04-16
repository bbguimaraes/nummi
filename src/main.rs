mod cache;
mod db;
mod dec;
mod net;

const PROG_NAME: &'static str = "nummi";

fn usage() {
    print!(r#"Usage: {exe} [-d <db_dir>] [<cmd>] [<args>]

  -d, --db-dir path          path to the database directory
                             (default: $XDG_DATA_HOME/{prog_name}/db)

Commands:

  <none>                     List all entries.
  currencies                 List all currencies present in the database.
  update-cache               Force an update of the currency exchange cache
                             file.
"#,
        exe = std::env::args().next().unwrap(),
        prog_name = PROG_NAME,
    )
}

fn parse_args() -> Option<(std::path::PathBuf, Vec<String>)> {
    let mut dir = std::path::PathBuf::new();
    let mut pos = Vec::new();
    let mut args = std::env::args();
    args.next().unwrap();
    loop {
        match args.next() {
            None => break,
            Some(arg) => match arg.as_str() {
                "-h" | "--help" => { usage(); return None; },
                "-d" | "--db-dir" => dir = std::path::PathBuf::from(
                    args.next().expect("-d requires an argument")),
                _ => pos.push(String::from(arg)),
            },
        }
    }
    if dir == std::path::PathBuf::new() {
        dir = std::env::var("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").expect("HOME not set");
                std::path::PathBuf::from(home).join(".local/share")
            })
            .join(PROG_NAME)
            .join("db");
    }
    Some((dir, pos))
}

fn cmd_list(d: &std::path::Path) {
    for x in db::Entry::read_db(&d).unwrap() {
        println!("{}", x.to_line());
    }
}

fn cmd_currencies(d: &std::path::Path) {
    for x in db::Entry::unique_currencies(&db::Entry::read_db(&d).unwrap()) {
        println!("{}", std::str::from_utf8(&x).unwrap());
    }
}

fn update_cache(force: bool) -> std::io::Result<cache::Cache> {
    let mut cache = cache::Cache::new();
    cache.read_currencies(&cache::dir(), force, || net::fetch_currencies())?;
    Ok(cache)
}

fn main() {
    let (dir, args) = match parse_args() {
        None => return,
        Some(x) => x,
    };
    let mut args = args.iter();
    match args.next().map(|x| x.as_str()).unwrap_or_default() {
        "" => cmd_list(&dir),
        "currencies" => cmd_currencies(&dir),
        "update-cache" => update_cache(true).and(Ok(())).unwrap(),
        x => panic!("invalid command: {}", x),
    }
}
