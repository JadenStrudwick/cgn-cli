//! # CGN-CLI (Compressed Game Notation Command Line Interface)

//! CGN-CLI is a simple command line interface for the CGN (Compressed Game Notation) library I created. It allows you to compress and decompress PGN files using the CGN library. It is designed to be fast, efficient, and flexible. It supports WASM compilation via wasm-pack, and contains 4 different compression algorithms to choose from.

//! ## Algorithms (High to Low Compression Ratios --- Low to High Speed)
//! 1) `opening-huffman` - A Huffman encoding algorithm that uses the huffman-encoding crate to compress the PGN data, but with an additional optimization for compressing common opening moves.
//! 2) `dynamic-huffman` - A Huffman encoding algorithm that uses the huffman-encoding crate to compress the PGN data, but with a huffman tree that is updated dynamically as the data is compressed.
//! 3) `huffman` - A Huffman encoding algorithm that uses a huffman-encoding crate to compress the PGN data.
//! 4) `bincode` - A simple binary encoding algorithm that uses the bincode crate to serialize the PGN data into a binary format.

//! ## Installation
//! ```bash
//! cargo install cgn-cli
//! ```

//! ## Getting Started
//! ```bash
//! cgn-cli --help
//! ```

mod benchmark;
use benchmark::{bench, ToTake};

mod genetic_algorithm;
use genetic_algorithm::{genetic_algorithm, GeneticAlgorithmConfig};

use cgn::compression::bincode::{bincode_compress_pgn_str, bincode_decompress_pgn_str};
use cgn::compression::dynamic_huffman::{
    dynamic_huffman_compress_pgn_str, dynamic_huffman_decompress_pgn_str,
};
use cgn::compression::huffman::{huffman_compress_pgn_str, huffman_decompress_pgn_str};
use cgn::compression::opening_huffman::{
    opening_huffman_compress_pgn_str, opening_huffman_decompress_pgn_str,
};
use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser)]
#[clap(name = "cgn", version = "0.1.0", author = "Jaden S")]
/// A command line interface for the cgn library
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
/// The commands for the command line interface
enum Commands {
    /// Compress a single PGN file
    Compress {
        /// Optimization level (0-3)
        #[clap(short, default_value = "3", value_parser = |s: &str| match s.parse::<u8>() {
            Ok(n) if n <= 3 => Ok(n),
            _ => Err(String::from("Optimization level must be between 0 and 3")),
        })]
        optimization_level: u8,

        /// Input file path
        #[clap(value_parser)]
        input_path: String,

        /// Output file path
        #[clap(value_parser)]
        output_path: String,
    },
    /// Decompress a single PGN file
    Decompress {
        /// Optimization level (0-3)
        #[clap(short, default_value = "3", value_parser = |s: &str| match s.parse::<u8>() {
            Ok(n) if n <= 3 => Ok(n),
            _ => Err(String::from("Optimization level must be between 0 and 3")),
        })]
        optimization_level: u8,

        /// Input file path
        #[clap(value_parser)]
        input_path: String,

        /// Output file path
        #[clap(value_parser)]
        output_path: String,
    },
    /// Benchmark the compression and decompression algorithms against a Lichess PGN database
    Bench {
        /// Number of games to benchmark each algorithm on. If set to 'all', then all games in the database will be used
        #[clap(value_parser)]
        number_of_games: ToTake,

        /// Input database path (Lichess PGN database format required)
        #[clap(value_parser)]
        input_db_path: String,

        /// Optional output file path for the benchmark results
        #[clap(value_parser)]
        output_path: Option<String>,
    },
    /// Run a genetic algorithm to find the optimal height and dev values for the dynamic Huffman compression algorithm. Used during development.
    GenAlgo {
        /// Initial population size
        #[clap(value_parser)]
        init_population: usize,

        /// Number of games to benchmark each individual on. If set to 'all', then all games in the database will be used
        #[clap(value_parser)]
        number_of_games: ToTake,

        /// Number of generations to run the genetic algorithm for
        #[clap(value_parser)]
        generations: u32,

        /// Mutation rate (0.0-1.0)
        #[clap(value_parser = |s: &str| s.parse::<f64>().map(|n| n.clamp(0.0, 1.0)))]
        mutation_rate: f64,

        /// Tournament size
        #[clap(value_parser)]
        tournament_size: usize,

        /// Minimum height value
        #[clap(value_parser)]
        height_min: f64,

        /// Maximum height value
        #[clap(value_parser)]
        height_max: f64,

        /// Minimum dev value
        #[clap(value_parser)]
        dev_min: f64,

        /// Maximum dev value
        #[clap(value_parser)]
        dev_max: f64,

        /// Input database path (Lichess PGN database format required)
        #[clap(value_parser)]
        input_db_path: String,

        /// Output file path for the genetic algorithm results
        #[clap(value_parser)]
        output_path: String,
    },
}

/// The main function for the command line interface.
fn main() {
    let cli = Args::parse();

    match cli.command {
        Commands::Compress {
            optimization_level,
            input_path,
            output_path,
        } => {
            // open and read the file into a string
            let mut input_file = File::open(input_path).unwrap();
            let mut pgn_str = String::new();
            input_file.read_to_string(&mut pgn_str).unwrap();

            // compress the PGN data using the specified optimization level
            let compressed_pgn_data = match optimization_level {
                0 => bincode_compress_pgn_str(&pgn_str),
                1 => huffman_compress_pgn_str(&pgn_str),
                2 => dynamic_huffman_compress_pgn_str(&pgn_str),
                3 => opening_huffman_compress_pgn_str(&pgn_str),
                _ => unreachable!(),
            };

            // if the vector is empty, then the compression failed
            if compressed_pgn_data.is_empty() {
                println!("Compression failed");
                return;
            }

            // write the compressed PGN data to the output file
            let mut output_file = File::create(output_path).unwrap();
            output_file.write_all(&compressed_pgn_data).unwrap();
        }
        Commands::Decompress {
            optimization_level,
            input_path,
            output_path,
        } => {
            // open and read the file into a string
            let mut input_file = File::open(input_path).unwrap();
            let mut compressed_pgn_data = Vec::new();
            input_file.read_to_end(&mut compressed_pgn_data).unwrap();

            // decompress the PGN data using the specified optimization level
            let pgn_data = match optimization_level {
                0 => bincode_decompress_pgn_str(&compressed_pgn_data),
                1 => huffman_decompress_pgn_str(&compressed_pgn_data),
                2 => dynamic_huffman_decompress_pgn_str(&compressed_pgn_data),
                3 => opening_huffman_decompress_pgn_str(&compressed_pgn_data),
                _ => unreachable!(),
            };

            // if the string is empty, then the decompression failed
            if pgn_data.is_empty() {
                println!("Decompression failed");
                return;
            }

            // write the decompressed PGN data to the output file
            let mut output_file = File::create(output_path).unwrap();
            output_file.write_all(pgn_data.as_bytes()).unwrap();
        }
        Commands::Bench {
            number_of_games,
            input_db_path,
            output_path,
        } => {
            bench(number_of_games, &input_db_path, &output_path);
        }
        Commands::GenAlgo {
            input_db_path,
            output_path,
            number_of_games,
            init_population,
            generations,
            height_min,
            height_max,
            dev_min,
            dev_max,
            mutation_rate,
            tournament_size,
        } => {
            let config = GeneticAlgorithmConfig {
                init_population,
                number_of_games,
                generations,
                mutation_rate,
                tournament_size,
                height_min,
                height_max,
                dev_min,
                dev_max,
                input_db_path,
                output_path,
            };
            genetic_algorithm(config);
        }
    }
}
