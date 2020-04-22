mod cache;
mod db;
mod dec;
mod net;
mod plot;

const PROG_NAME: &'static str = "nummi";

fn usage() {
    print!(r#"Usage: {exe} [-d <db_dir>] [<cmd>] [<args>]

  -d, --db-dir path          path to the database directory
                             (default: $XDG_DATA_HOME/{prog_name}/db)

Commands:

  <none>                     List all entries.
  check                      Verify database entries.
  currencies                 List all currencies present in the database.
  update-cache               Force an update of the currency exchange cache
                             file.
  plot                       Generate a `gnuplot` graphic summarizing with the
                             monthly historical total.
"#,
        exe = std::env::args().next().unwrap(),
        prog_name = PROG_NAME,
    )
}

struct Configuration {
    exe: String,
    dir: std::path::PathBuf,
    args: Vec<String>,
}

fn parse_args() -> Option<Configuration> {
    let mut dir = std::path::PathBuf::new();
    let mut pos = Vec::new();
    let mut args = std::env::args();
    let exe = args.next().unwrap();
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
    Some(Configuration { exe, dir, args: pos })
}

fn cmd_list(d: &std::path::Path) {
    for x in db::Entry::read_db(&d).unwrap() {
        println!("{}", x.to_line());
    }
}

fn cmd_check(d: &std::path::Path) {
    db::Entry::check_db(&d).unwrap();
}

fn cmd_currencies(d: &std::path::Path) {
    for x in db::Entry::unique_currencies(&db::Entry::read_db(&d).unwrap()) {
        println!("{}", std::str::from_utf8(&x).unwrap());
    }
}

fn cmd_plot(d: &std::path::Path) {
    let entries = db::Entry::read_db(&d).unwrap();
    let currencies = update_cache(false).unwrap().currencies;
    let currencies: std::collections::HashMap<_, _> = currencies
        .iter()
        .map(|x| (x.name, dec::Decimal::new(1.0) / x.to_eur))
        .collect();
    plot::plot(
        &entries,
        &currencies,
        &chrono::Local::now().naive_local().date()).unwrap();
}

fn update_cache(force: bool) -> std::io::Result<cache::Cache> {
    let mut cache = cache::Cache::new();
    cache.read_currencies(&cache::dir(), force, || net::fetch_currencies())?;
    Ok(cache)
}

fn main() {
    let conf = match parse_args() {
        None => return,
        Some(x) => x,
    };
    let mut args = conf.args.iter();
    match args.next().map(|x| x.as_str()).unwrap_or_default() {
        "" => cmd_list(&conf.dir),
        "check" => cmd_check(&conf.dir),
        "currencies" => cmd_currencies(&conf.dir),
        "update-cache" => update_cache(true).and(Ok(())).unwrap(),
        "plot" => cmd_plot(&conf.dir),
        x => {
            eprintln!("{}: invalid command: {}", conf.exe, x);
            std::process::exit(1);
        },
    }
}
