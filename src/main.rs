use clap::{AppSettings, Parser};

fn main() {
    let args = Args::parse();
    let files = match get_corpus_files(&args.files) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };
    let gen_passphrase_options = markovpass::GenPassphraseOptions {
        files,
        number: args.number,
        min_entropy: args.min_entropy,
        ngram_length: args.ngram_length,
        min_word_length: args.min_word_length,
    };
    let passphrases = match markovpass::gen_passphrases(&gen_passphrase_options) {
        Ok(passphrases) => passphrases,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    for (passphrase, entropy) in passphrases {
        if args.show_entropy {
            println!("{} <{}>", passphrase, entropy);
        } else {
            println!("{}", passphrase);
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(version, about, setting = AppSettings::DeriveDisplayOrder)]
struct Args {
    /// Files to use as markov chain input corpus. Use '-' to read from stdin
    #[clap(value_parser)]
    pub files: Vec<String>,

    /// Number of passphrases to generate
    #[clap(short = 'n', value_parser, default_value_t = 1)]
    pub number: usize,

    /// Minimum entropy
    #[clap(short = 'e', value_parser, default_value_t = 60.0)]
    pub min_entropy: f64,

    /// Ngram length
    #[clap(short = 'l', value_parser, default_value_t = 3)]
    pub ngram_length: usize,

    /// Minimum word length for corpus
    #[clap(short = 'w', value_parser, default_value_t = 5)]
    pub min_word_length: usize,

    /// Print the entropy for each passphrase
    #[clap(long, value_parser, default_value_t = false)]
    pub show_entropy: bool,
}

fn get_corpus_files(files: &[String]) -> std::io::Result<Vec<std::path::PathBuf>> {
    match files {
        [] => get_data_files(),
        [x] if x == "-" => Ok(vec![]),
        _ => Ok(files.iter().map(|f| f.into()).collect()),
    }
}

fn get_data_files() -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut data_dirs = directories::ProjectDirs::from_path("markovpass".into())
        .map(|pds| vec![pds.data_dir().to_path_buf()])
        .unwrap_or_default();
    if cfg!(target_os = "linux") {
        data_dirs.extend(
            std::env::var("XDG_DATA_DIRS")
                .unwrap_or_else(|_| "/usr/local/share/:/usr/share/".to_string())
                .split(':')
                .map(|s| std::path::PathBuf::from(s).join("markovpass")),
        );
    }
    for dir in &data_dirs {
        if dir.exists() {
            let entries = std::fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
            let paths: Vec<_> = entries
                .into_iter()
                .map(|entry| entry.path())
                .filter(|path| path.is_file())
                .collect();
            if !paths.is_empty() {
                return Ok(paths);
            }
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "No corpus data found in any of {}.",
            data_dirs
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ))
}
