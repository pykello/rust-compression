use std::env;
use std::fs;
use std::io::{Read, Write};
use std::time::Instant;
use std::time::Duration;

const CHUNK_MB: usize = 200;
const CHUNK_SIZE: usize = CHUNK_MB * 1024 * 1024; // 200 MB

struct BenchmarkResults {
    compressed_sizes: Vec<usize>,
    compress_times: Vec<Duration>,
    decompress_times: Vec<Duration>,
}

impl BenchmarkResults {
    fn new() -> Self {
        Self {
            compressed_sizes: Vec::new(),
            compress_times: Vec::new(),
            decompress_times: Vec::new(),
        }
    }

    fn merge(&mut self, other: BenchmarkResults) {
        self.compressed_sizes.extend(other.compressed_sizes);
        self.compress_times.extend(other.compress_times);
        self.decompress_times.extend(other.decompress_times);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments
    let mut filename = None;
    let mut num_runs = 1; // Default to 1 run
    
    let mut arg_index = 1;
    while arg_index < args.len() {
        match args[arg_index].as_str() {
            "--runs" => {
                if arg_index + 1 >= args.len() {
                    eprintln!("Error: --runs requires a value");
                    eprintln!("Usage: {} [--runs <N>] <filename>", args[0]);
                    std::process::exit(1);
                }
                num_runs = args[arg_index + 1].parse().unwrap_or_else(|_| {
                    eprintln!("Error: --runs value must be a number");
                    std::process::exit(1);
                });
                if num_runs == 0 {
                    eprintln!("Error: --runs value must be at least 1");
                    std::process::exit(1);
                }
                arg_index += 2;
            }
            arg if !arg.starts_with("--") => {
                if filename.is_some() {
                    eprintln!("Error: multiple filenames provided");
                    eprintln!("Usage: {} [--runs <N>] <filename>", args[0]);
                    std::process::exit(1);
                }
                filename = Some(arg.to_string());
                arg_index += 1;
            }
            _ => {
                eprintln!("Error: unknown option '{}'", args[arg_index]);
                eprintln!("Usage: {} [--runs <N>] <filename>", args[0]);
                std::process::exit(1);
            }
        }
    }
    
    let filename = filename.unwrap_or_else(|| {
        eprintln!("Usage: {} [--runs <N>] <filename>", args[0]);
        std::process::exit(1);
    });

    // Get file size
    let metadata = fs::metadata(&filename).expect("Failed to read file metadata");
    let original_size = metadata.len() as usize;

    println!("File: {}", filename);
    println!(
        "Original size: {} bytes ({:.2} MiB)",
        original_size,
        original_size as f64 / (1024.0 * 1024.0)
    );
    println!("Number of runs per algorithm: {}", num_runs);
    println!();

    // Process file in chunks
    let mut file = fs::File::open(&filename).expect("Failed to open file");
    let mut chunk_number = 0;

    let mut flate2_results = BenchmarkResults::new();
    let mut snap_results = BenchmarkResults::new();
    let mut lz4_results = BenchmarkResults::new();
    let mut zstd_fastest_results = BenchmarkResults::new();
    let mut zstd_balanced_results = BenchmarkResults::new();
    let mut zstd_max_results = BenchmarkResults::new();
    let mut xz2_results = BenchmarkResults::new();
    let mut lzma_rs_results = BenchmarkResults::new();
    let mut miniz_oxide_results = BenchmarkResults::new();
    let mut lz4_flex_results = BenchmarkResults::new();
    let mut libdeflate_results = BenchmarkResults::new();

    loop {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        let bytes_read = file.read(&mut chunk).expect("Failed to read chunk");

        if bytes_read == 0 {
            break;
        }

        chunk.truncate(bytes_read);
        chunk_number += 1;

        println!("Processing chunk {} ({} bytes)...", chunk_number, bytes_read);

        // Benchmark each compression algorithm on this chunk
        flate2_results.merge(benchmark_flate2(&chunk, bytes_read, num_runs));
        snap_results.merge(benchmark_snap(&chunk, bytes_read, num_runs));
        lz4_results.merge(benchmark_lz4(&chunk, bytes_read, num_runs));
        zstd_fastest_results.merge(benchmark_zstd_fastest(&chunk, bytes_read, num_runs));
        zstd_balanced_results.merge(benchmark_zstd_balanced(&chunk, bytes_read, num_runs));
        zstd_max_results.merge(benchmark_zstd_max(&chunk, bytes_read, num_runs));
        xz2_results.merge(benchmark_xz2(&chunk, bytes_read, num_runs));
        lzma_rs_results.merge(benchmark_lzma_rs(&chunk, bytes_read, num_runs));
        miniz_oxide_results.merge(benchmark_miniz_oxide(&chunk, bytes_read, num_runs));
        lz4_flex_results.merge(benchmark_lz4_flex(&chunk, bytes_read, num_runs));
        libdeflate_results.merge(benchmark_libdeflate(&chunk, bytes_read, num_runs));
    }

    println!();
    println!(
        "{:<20} {:>15} {:>20} {:>20}",
        "Algorithm", "Ratio", "Compress (MiB/s)", "Decompress (MiB/s)"
    );
    println!("{}", "-".repeat(80));

    // Print aggregated results
    print_results("flate2 (gzip)", original_size, &flate2_results);
    print_results("snap (snappy)", original_size, &snap_results);
    print_results("lz4", original_size, &lz4_results);
    print_results("zstd (level 1)", original_size, &zstd_fastest_results);
    print_results("zstd (level 3)", original_size, &zstd_balanced_results);
    print_results("zstd (level 10)", original_size, &zstd_max_results);
    print_results("xz2 (lzma)", original_size, &xz2_results);
    print_results("lzma-rs", original_size, &lzma_rs_results);
    print_results("miniz_oxide", original_size, &miniz_oxide_results);
    print_results("lz4_flex", original_size, &lz4_flex_results);
    print_results("libdeflate", original_size, &libdeflate_results);
}

fn benchmark_flate2(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use flate2::write::{GzDecoder, GzEncoder};
    use flate2::Compression;

    println!("  [flate2] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        let compressed = encoder.finish().unwrap();
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [flate2] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let mut decoder = GzDecoder::new(Vec::new());
        decoder.write_all(&compressed).unwrap();
        let _decompressed = decoder.finish().unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [flate2] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_snap(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use snap::raw::{Decoder, Encoder};

    println!("  [snap] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed = Encoder::new().compress_vec(data).unwrap();
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [snap] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let _decompressed = Decoder::new().decompress_vec(&compressed).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [snap] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lz4(data: &[u8], original_size: usize, num_runs: usize) -> BenchmarkResults {
    use lz4::block::{compress, decompress};

    println!("  [lz4] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed = compress(data, None, false).unwrap();
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [lz4] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let _decompressed = decompress(&compressed, Some(original_size as i32)).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [lz4] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_zstd(data: &[u8], _original_size: usize, num_runs: usize, level: i32, level_name: &str) -> BenchmarkResults {
    println!("  [zstd {}] Starting benchmark...", level_name);
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed = zstd::encode_all(data, level).unwrap();
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [zstd {}] Run {}: compressed to {} bytes in {:.3}ms", 
                 level_name, run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let _decompressed = zstd::decode_all(&compressed[..]).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [zstd {}] Run {}: decompressed in {:.3}ms", 
                 level_name, run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_zstd_fastest(data: &[u8], original_size: usize, num_runs: usize) -> BenchmarkResults {
    benchmark_zstd(data, original_size, num_runs, 1, "fastest")
}

fn benchmark_zstd_balanced(data: &[u8], original_size: usize, num_runs: usize) -> BenchmarkResults {
    benchmark_zstd(data, original_size, num_runs, 3, "balanced")
}

fn benchmark_zstd_max(data: &[u8], original_size: usize, num_runs: usize) -> BenchmarkResults {
    benchmark_zstd(data, original_size, num_runs, 10, "max")
}

fn benchmark_xz2(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use xz2::read::{XzDecoder, XzEncoder};

    println!("  [xz2] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let mut encoder = XzEncoder::new(&data[..], 6);
        let mut compressed = Vec::new();
        encoder.read_to_end(&mut compressed).unwrap();
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [xz2] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let mut decoder = XzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [xz2] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lzma_rs(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use lzma_rs::lzma_compress;
    use lzma_rs::lzma_decompress;

    println!("  [lzma-rs] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let mut compressed = Vec::new();
        lzma_compress(&mut &data[..], &mut compressed).unwrap();
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [lzma-rs] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let mut decompressed = Vec::new();
        lzma_decompress(&mut &compressed[..], &mut decompressed).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [lzma-rs] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_miniz_oxide(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use miniz_oxide::deflate::compress_to_vec;
    use miniz_oxide::inflate::decompress_to_vec;

    println!("  [miniz_oxide] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed = compress_to_vec(data, 6);
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [miniz_oxide] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let _decompressed = decompress_to_vec(&compressed).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [miniz_oxide] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lz4_flex(data: &[u8], original_size: usize, num_runs: usize) -> BenchmarkResults {
    println!("  [lz4_flex] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed = lz4_flex::compress(data);
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [lz4_flex] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let _decompressed = lz4_flex::decompress(&compressed, original_size).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [lz4_flex] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_libdeflate(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use libdeflater::{Compressor, Decompressor, CompressionLvl};

    println!("  [libdeflate] Starting benchmark...");
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let mut compressor = Compressor::new(CompressionLvl::default());
        let max_sz = compressor.deflate_compress_bound(data.len());
        let mut compressed = vec![0u8; max_sz];
        let compressed_size = compressor.deflate_compress(data, &mut compressed).unwrap();
        compressed.truncate(compressed_size);
        let compress_time = start.elapsed();
        compress_times.push(compress_time);
        compressed_sizes.push(compressed.len());
        println!("  [libdeflate] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed.len(), compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let mut decompressor = Decompressor::new();
        let mut decompressed = vec![0u8; data.len()];
        let _decompressed_size = decompressor.deflate_decompress(&compressed, &mut decompressed).unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [libdeflate] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn print_results(
    name: &str,
    original_size: usize,
    results: &BenchmarkResults,
) {
    // Guard against empty results
    if results.compressed_sizes.is_empty() 
        || results.compress_times.is_empty() 
        || results.decompress_times.is_empty() {
        eprintln!("Warning: No results to display for {}", name);
        return;
    }

    let avg_compressed_size =
        results.compressed_sizes.iter().sum::<usize>() as f64 / results.compressed_sizes.len() as f64;
    let ratio = original_size as f64 / avg_compressed_size;

    let avg_compress_time =
        results.compress_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / results.compress_times.len() as f64;
    let compress_throughput = (original_size as f64 / (1024.0 * 1024.0)) / avg_compress_time;

    let avg_decompress_time = results.decompress_times
        .iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>()
        / results.decompress_times.len() as f64;
    let decompress_throughput = (original_size as f64 / (1024.0 * 1024.0)) / avg_decompress_time;

    println!(
        "{:<20} {:>15.2} {:>20.2} {:>20.2}",
        name, ratio, compress_throughput, decompress_throughput
    );
}
