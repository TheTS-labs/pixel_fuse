mod write;
mod read;
mod meta;

use read::Read;
use write::Write;

use clap::{Parser, Subcommand};

/// Encode file as a GIF and decode it back.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Encode input as a GIF and write the GIF to output
    Encode {
        /// The file to work with
        input: std::path::PathBuf,
        /// The file to write output to
        output: std::path::PathBuf,
        /// Width of the generated GIF
        #[clap(long, default_value_t = 100)]
        width: u16,
        /// Height of the generated GIF
        #[clap(long, default_value_t = 100)]
        height: u16
    },

    /// Decode input to a normal file and write the content to output
    Decode {
        /// The file to work with
        input: std::path::PathBuf,
        /// The file to write output to
        output: std::path::PathBuf,
    },
}


fn main() {
    let args = Cli::parse();

    match &args.command {
        Some(Commands::Encode { input, output, width, height }) => {
            let input = input.clone().into_os_string().into_string().unwrap();
            let output = output.clone().into_os_string().into_string().unwrap();

            let mut write = Write::from(
                &input,
                &output,
                *width,
                *height
            );
            write.go();
        }
        Some(Commands::Decode { input, output }) => {
            let input = input.clone().into_os_string().into_string().unwrap();
            let output = output.clone().into_os_string().into_string().unwrap();

            let read = Read::from(&input, &output);
            read.go();
        }
        None => {
            panic!("No command specified");
        }
    }
}
