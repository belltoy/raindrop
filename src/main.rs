use std::io::BufRead;
use std::path::PathBuf;
use glob::glob;
use structopt::StructOpt;
use log::{debug, info, error};
use pretty_env_logger::env_logger as logger;

mod exhaust;
mod db;

#[derive(StructOpt, Debug)]
struct Args {

    #[structopt(short, long, help = "DONOT execute SQL statements")]
    check: bool,

    #[structopt(short, long, help = "Input paths", required(true))]
    inputs: Vec<String>,

    #[structopt(short, long, help = "Execute or not")]
    execute: bool,

    #[structopt(short, long, help = "MySQL URL, example: `mysql://user:password@host:port/db_name`", required_if("execute", "true"))]
    mysql: Option<String>,
}

fn main() -> Result<(), String> {
    logger::from_env(logger::Env::default().default_filter_or("info")).init();
    let args = Args::from_args();

    if args.execute && args.mysql.is_none() {
        return Err("Require mysql URL".into());
    }

    let inputs = read_files(&args.inputs)?;

    let filtered: Vec<_> = inputs.iter()
        .filter(|(file, p)| {
            if p.len() == 0 {
                info!("Filter out empty file: {:?}", file);
                false
            } else {
                true
            }
        })
        .enumerate() // attach client No.
        .map(|(no, (file, p))| {
            (no, file, p)
        }).collect();

    let input_contents: Vec<_> = filtered.iter().map(|(no, _file, p)| {
        p.iter().map(|s| (no, s.as_str())).collect::<Vec<_>>()
    }).collect();

    let input_files: Vec<_> = filtered.iter().map(|(_no, file, _p)| file).collect();
    debug!("input files: {:?}", input_files);

    assert_eq!(input_files.len(), input_contents.len());

    // ref
    let input_contents: Vec<_> = (&input_contents[..]).iter().map(|p| &p[..]).collect();
    // shuffle and exhaust all possible cases
    let outputs = exhaust::shuffle(&input_contents[..]);

    outputs.iter().enumerate().for_each(|(i, case)| {
        debug!("-- Case: {}", i);
        case.iter().for_each(|(idx, line)| {
            let idx = **idx;
            debug!("   {} -- [client: {}] [from file: {:?}]", line, idx, input_files[idx]);
        });
    });

    // Need execution
    if args.execute {
        execute_sqls(args.mysql.as_ref().unwrap(), &input_files, &outputs)?;
    }

    Ok(())
}

/// Read sql statements from input files which's path was given in the parameter
fn read_files(inputs: &Vec<String>) -> Result<Vec<(PathBuf, Vec<String>)>, String> {
    let inputs_paths = inputs.iter().flat_map(|p| {
        let mut result = glob(p).unwrap().peekable();
        if result.peek().is_none() {
            error!("Invalid input argument: {}", p);
        }
        result
    });

    let mut unique_inputs = Vec::new();
    inputs_paths
    .flat_map(|p| {
        p.map_err(|e| format!("glob pattern error: {}", e))
         .and_then(|p| {
             // If path is dir, we read files in that dir
             // Only support one level currently.
             if p.is_dir() {
                 p.read_dir().map_err(|e| format!("read_dir error: {}", e))
                 .map(|op: std::fs::ReadDir| {
                     op.map(|p| {
                         p
                         .map(|dir_entry| dir_entry.path())
                         .map_err(|e| format!("IO error during read dir iteration: {}", e))
                     })
                     .filter(|p| {
                         // ignore inner dir
                         if let Ok(p) = p {
                             !p.is_dir()
                         } else {
                             true
                         }
                     }).collect::<Vec<_>>()
                 })
             } else {
                 Ok(vec![Ok(p)])
             }
         })
    })
    .flat_map(|p| p)

    // collect unique inputs paths
    .for_each(|p| {
        if !unique_inputs.contains(&p) {
            unique_inputs.push(p);
        }
    });

    let (inputs, errors): (Vec<Result<_, _>>, _) = unique_inputs.into_iter().partition(Result::is_ok);

    info!("inputs files: {:?}", inputs);
    if errors.len() > 0 {
        // error!("Errors: {:?}", errors);
        return Err(format!("{:?}", errors));
    }

    let inputs = inputs.into_iter().map(Result::unwrap);

    let (inputs_contents, io_errors): (Vec<_>, _) = inputs.map(|input_file| -> Result<_, String> {
        debug!("Open file: {:?}", input_file);
        let f = std::fs::File::open(&input_file).map_err(|e| {
            format!("open file {} error: {}", input_file.to_str().unwrap_or("unknown path"), e)
        })?;

        let reader = std::io::BufReader::new(f);
        let lines: Vec<_> = reader.lines().map(|line| {
            // turn std::io::Error to String error
            let line = line.map_err(|e| format!("IO Error when reading lines: {}", e));
            line
        }).collect();
        Ok((input_file, lines))
    }).partition(Result::is_ok);

    let inputs_contents = inputs_contents.into_iter().map(Result::unwrap);
    let io_errors: Vec<_> = io_errors.into_iter().map(Result::unwrap_err).collect();
    // report errors and exit
    if io_errors.len() > 0 {
        let e = &io_errors[0];
        return Err(e.into());
    }

    let inputs = inputs_contents.map(|(file_name, lines_result)| {
        let lines: Vec<_> = lines_result.into_iter()

            // TODO handle read line error
            .map(Result::unwrap)

            // filter out empty lines
            .map(|line| line.trim().to_owned())
            .filter(|line| line.len() > 0)

            .collect();

        (file_name, lines)
    }).collect::<Vec<_>>();

    Ok(inputs)
}

/// Execute sql cases
fn execute_sqls<I: std::fmt::Debug>(url: &str, input_files: &[I], outputs: &[Vec<(&usize, &str)>]) -> Result<(), String> {
    let pool = db::init_clients(&url, input_files.len()).map_err(|e| {
        format!("db error: {:?}", e)
    })?;

    let (clients, errors): (Vec<_>, _) = input_files.iter().map(|file| {
        let file_path = format!("{:?}", file);
        pool.get_conn().map(|p| (p, file_path))
    })
    .partition(Result::is_ok);
    let mut clients: Vec<_> = clients.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    if errors.len() > 0 {
        return Err(format!("get connection error: {:?}", errors[0]));
    }

    db::execute_sqls(&mut clients[..], &outputs[..]).map_err(|e| {
        format!("mysql error: {:?}", e)
    })?;

    Ok(())
}
