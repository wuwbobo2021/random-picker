use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

const MSG_HELP: &str = "\
random-picker [conf|calc|test] <table_file> [pick_amount] [-n] [-f]
Description:
conf    Create the table file by user input
calc    Calculate and print probabilities of being picked up
test    Generate some amount of results and print the frequency table
-n      Do not print warning for the nonuniform distribution
-f      Use the fast pseudo random generator instead of OS random source
Note:
`pick_amount` is set to 1 if not given, and it makes no sense with `conf`.
When repetitive mode is off, `pick_amount` must not exceed the table length.
More information: <https://crates.io/crates/random-picker/0.2.3>
";

struct Params {
    operation: Operation,
    table_path: PathBuf,
    pick_amount: usize,
    know_nonuniform: bool,
    use_fast_rng: bool,
}

#[derive(PartialEq, Eq)]
enum Operation {
    Conf,
    Pick,
    Calc,
    Test,
}

impl Params {
    fn build(args: std::env::Args) -> Result<Self, &'static str> {
        let mut params = Self {
            operation: Operation::Pick,
            table_path: PathBuf::new(),
            pick_amount: 1,
            know_nonuniform: false,
            use_fast_rng: false,
        };

        let cur_exe = env::current_exe().unwrap_or_default();
        let cur_exe_name = cur_exe.file_name();

        for arg in args {
            match &arg as &str {
                "conf" => params.operation = Operation::Conf,
                "calc" => params.operation = Operation::Calc,
                "test" => params.operation = Operation::Test,
                "-n" => params.know_nonuniform = true,
                "-f" => params.use_fast_rng = true,
                _ => {
                    if let Ok(n) = usize::from_str(&arg) {
                        params.pick_amount = n;
                        continue;
                    }
                    let path = Path::new(&arg);
                    if path.file_name() == cur_exe_name {
                        continue;
                    }
                    if let Ok(true) = path.try_exists() {
                        params.table_path = path.to_path_buf();
                    } else if params.operation == Operation::Conf {
                        params.table_path = path.to_path_buf();
                    }
                }
            }
        }

        if params.table_path != PathBuf::new() {
            Ok(params)
        } else {
            Err("Table file not found")
        }
    }
}

fn main() {
    let params = Params::build(std::env::args()).unwrap_or_else(|err| {
        eprintln!("Failed to parse arguments: {err}");
        print!("\n{MSG_HELP}");
        std::process::exit(1);
    });

    use Operation::*;

    let mut conf = random_picker::Config::new();
    if let Ok(s) = fs::read_to_string(&params.table_path) {
        conf.append_str(&s);
    }
    if params.operation != Operation::Conf {
        if conf.check().is_err() {
            eprintln!("Failed to open table file");
            std::process::exit(1);
        }
    } else {
        configure(&mut conf);
        fs::write(&params.table_path, format!("{conf}")).expect("Failed to save file");
        return;
    }

    use random_picker::Picker;
    match params.operation {
        Pick => {
            let is_fair = conf.is_fair();
            let result = if !params.use_fast_rng {
                random_picker::pick(params.pick_amount, conf)
            } else {
                let mut picker = Picker::build_with_rng(conf, rand::thread_rng()).unwrap();
                picker.pick(params.pick_amount)
            };
            match result {
                Ok(table) => {
                    for item in table {
                        print!("{item} ");
                    }
                    if !is_fair && !params.know_nonuniform {
                        print!("(nonuniform)");
                    }
                    println!();
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        Calc => {
            let mut table = random_picker::Table::new();
            println!("Calculating, please wait...");
            let time_cost = measure_exec_time(|| {
                table = conf.calc_probabilities(params.pick_amount).unwrap();
            });
            println!("Time passed: {} ms", time_cost.as_millis());
            table.iter_mut().for_each(|(_, val)| *val *= 100.);
            random_picker::print_table(&table);
        }
        Test => {
            print!("Input amount of result groups for making statistics: ");
            let _ = io::stdout().flush();
            let test_times = if let Some(Ok(input)) = io::stdin().lines().next() {
                input.trim().parse().unwrap_or(5_000_000)
            } else {
                5_000_000
            };
            println!("Testing for {test_times} times, please wait...");
            let mut table = random_picker::Table::new();
            let time_cost = measure_exec_time(|| {
                let result = if !params.use_fast_rng {
                    let mut picker = Picker::build(conf).unwrap();
                    picker.test_freqs(params.pick_amount, test_times)
                } else {
                    let mut picker = Picker::build_with_rng(conf, rand::thread_rng()).unwrap();
                    picker.test_freqs(params.pick_amount, test_times)
                };
                if let Err(e) = result {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
                table = result.unwrap();
            });
            println!("Time passed: {} ms", time_cost.as_millis());
            table.iter_mut().for_each(|(_, val)| *val *= 100.);
            random_picker::print_table(&table);
        }
        _ => (),
    }
}

fn configure(conf: &mut random_picker::Config<String>) {
    if conf.check().is_ok() {
        println!("Existing configuration:\n{conf}");
    }
    if let Some(b) = ask_yes_no("Is it allowed to pick items repetitively?") {
        conf.repetitive = b;
    }
    if let Some(b) = ask_yes_no("Should the probability values be inversed (x -> 1/x)?") {
        conf.inversed = b;
    }
    println!("Input items by line (or use ';' seperator): <name> [=] <val>");
    println!("(name: string without space, val: positive numeric value)");
    println!("delete item with `delete <name>`, enter `end` to end input: ");
    let mut lines_input = io::stdin().lines();
    while let Some(Ok(s)) = lines_input.next() {
        if s == "end" {
            break;
        }
        conf.append_str(&s);
    }
    print!("\nNew configuration:\n{conf}");
    if let Err(e) = conf.check() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn ask_yes_no(question: &str) -> Option<bool> {
    print!("{} (Y/n) ", question);
    io::stdout().flush().ok()?;
    let s = io::stdin().lines().next()?.ok()?;
    match s.trim().chars().next()? {
        'Y' | 'y' => Some(true),
        'n' | 'N' => Some(false),
        _ => None,
    }
}

fn measure_exec_time<F: FnOnce()>(fn_exec: F) -> Duration {
    use std::time::SystemTime;
    let t_start = SystemTime::now();
    fn_exec();
    SystemTime::now().duration_since(t_start).unwrap()
}
