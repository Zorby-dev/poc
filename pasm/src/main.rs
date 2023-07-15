use std::{fs, path::PathBuf, io};

use clap::{command, arg, crate_version};
use pasm::{compiler::compile, source::Source, message::{Message, human_count}};

fn main() {
    let mut stdout = io::stdout();

    let matches = command!()
        .about("Assembler for the POC-8 computer architecture")
        .version(crate_version!())

        .arg(arg!(                     <input>                             "Input file"))
        .arg(arg!(-o        --output    <FILE>                            "Output file"))
        .arg(arg!(-s  --"image-size"    <SIZE>  "The size of the output image in bytes"))
        .arg(arg!(-v        --verbose                        "Toggle verbose reporting"))
        
        .get_matches();

    let input_path = PathBuf::from(matches.get_one::<String>("input")
        .expect("Input should be present"));
    let output_path = match matches.get_one::<String>("output") {
        Some(output) => PathBuf::from(output),
        None => {
            let mut output = input_path.clone();
            output.set_extension("bin");
            output
        }
    };
    let image_size = matches.get_one::<String>("image-size")
        .map(|image_size| image_size.parse::<usize>().unwrap());
    let verbose = matches.get_flag("verbose");

    let source = Source {
        text: fs::read_to_string(&input_path).unwrap(),
        path: input_path.clone()
    };

    match compile(source, output_path, image_size, verbose) {
        Ok(((), warnings)) => for warning in warnings {
            warning.format(&mut stdout)
        },
        Err(errors) => {
            for error in &errors {
                error.format(&mut stdout)
            }
            Message::error(format!("could not compile '{}' due to previous {}",
                input_path.display(), human_count("error", errors.len())
            )).format(&mut stdout);
        }
    }
}