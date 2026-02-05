use std::env;
use std::fs;
use std::hint::black_box;
use std::io::{Read, Write};
use std::time::Instant;
use std::time::Duration;

const CHUNK_MB: usize = 256;
const CHUNK_SIZE: usize = CHUNK_MB * 1024 * 1024; // 256 MB

struct BenchmarkResults {
    input_sizes: Vec<usize>,
    compressed_sizes: Vec<usize>,
    compress_times: Vec<Duration>,
    decompress_times: Vec<Duration>,
}

impl BenchmarkResults {
    fn new() -> Self {
        Self {
            input_sizes: Vec::new(),
            compressed_sizes: Vec::new(),
            compress_times: Vec::new(),
            decompress_times: Vec::new(),
        }
    }

    fn merge(&mut self, other: BenchmarkResults) {
        self.input_sizes.extend(other.input_sizes);
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
    let mut memcpy_results = BenchmarkResults::new();

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
        memcpy_results.merge(benchmark_memcpy(&chunk, num_runs));
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
        "| {:<20} | {:>6} | {:>16} | {:>18} |",
        "Algorithm",
        "Ratio",
        "Compress (MiB/s)",
        "Decompress (MiB/s)"
    );
    println!("| {:-<20} | {:-<6} | {:-<16} | {:-<18} |", "", "", "", "");

    // Print aggregated results
    print_results("memcpy", &memcpy_results);
    print_results("flate2 (gzip)", &flate2_results);
    print_results("snap (snappy)", &snap_results);
    print_results("lz4", &lz4_results);
    print_results("zstd (level 1)", &zstd_fastest_results);
    print_results("zstd (level 3)", &zstd_balanced_results);
    print_results("zstd (level 10)", &zstd_max_results);
    print_results("xz2 (lzma)", &xz2_results);
    print_results("lzma-rs", &lzma_rs_results);
    print_results("miniz_oxide", &miniz_oxide_results);
    print_results("lz4_flex", &lz4_flex_results);
    print_results("libdeflate", &libdeflate_results);
}

fn benchmark_memcpy(data: &[u8], num_runs: usize) -> BenchmarkResults {
    println!("  [memcpy] Preparing buffers ...");

    // 1. Pre-allocate buffers
    let len = data.len();
    let mut compressed = vec![0u8; len];
    let mut decompressed = vec![0u8; len];

    // 2. WARM-UP & PAGE-FAULTING: Ensure OS has actually allocated physical RAM
    // This prevents "cold start" latency from ruining the first run.
    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }

    // Warm-up run
    unsafe {
        std::ptr::copy_nonoverlapping(black_box(data.as_ptr()), compressed.as_mut_ptr(), len);
        std::ptr::copy_nonoverlapping(compressed.as_ptr(), decompressed.as_mut_ptr(), len);
    }
    black_box(&mut compressed);
    black_box(&mut decompressed);

    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    println!("  [memcpy] Starting benchmark ({} runs)...", num_runs);

    for run in 0..num_runs {
        // --- Forward Copy (Data -> Compressed) ---
        let src = black_box(data.as_ptr());
        let dst = black_box(compressed.as_mut_ptr());

        let start = Instant::now();
        unsafe {
            // This is the closest Rust equivalent to C's memcpy
            std::ptr::copy_nonoverlapping(src, dst, len);
        }
        // Force the CPU to treat the memory as 'dirty' so the copy isn't skipped
        black_box(&mut compressed);
        let compress_time = start.elapsed();

        // --- Backward Copy (Compressed -> Decompressed) ---
        let src_back = black_box(compressed.as_ptr());
        let dst_back = black_box(decompressed.as_mut_ptr());

        let start_back = Instant::now();
        unsafe {
            std::ptr::copy_nonoverlapping(src_back, dst_back, len);
        }
        black_box(&mut decompressed);
        let decompress_time = start_back.elapsed();

        // Calculate and Print MiB/s for immediate feedback
        let mib_s = (len as f64 / (1024.0 * 1024.0)) / compress_time.as_secs_f64();
        println!(
            "  Run {}: {:.2} MiB/s ({:.3}ms)",
            run + 1,
            mib_s,
            compress_time.as_secs_f64() * 1000.0
        );

        compress_times.push(compress_time);
        decompress_times.push(decompress_time);
        input_sizes.push(len);
        compressed_sizes.push(black_box(len));
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_flate2(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use flate2::write::{GzDecoder, GzEncoder};
    use flate2::Compression;

    println!("  [flate2] Starting benchmark...");
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let mut compressed = vec![0u8; data.len() + 1024];
    let mut decompressed = vec![0u8; data.len()];

    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }
    compressed.clear();
    decompressed.clear();

    // Warm-up run
    let mut encoder = GzEncoder::new(compressed, Compression::default());
    encoder.write_all(black_box(data)).unwrap();
    compressed = encoder.finish().unwrap();
    let mut decoder = GzDecoder::new(decompressed);
    decoder.write_all(black_box(&compressed)).unwrap();
    decompressed = decoder.finish().unwrap();
    black_box(compressed.len());
    black_box(decompressed.len());

    for run in 0..num_runs {
        // Compression
        compressed.clear();
        let start = Instant::now();
        let mut encoder = GzEncoder::new(compressed, Compression::default());
        encoder.write_all(black_box(data)).unwrap();
        compressed = encoder.finish().unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed.len());
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [flate2] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        decompressed.clear();
        let start = Instant::now();
        let mut decoder = GzDecoder::new(decompressed);
        decoder.write_all(black_box(&compressed)).unwrap();
        decompressed = decoder.finish().unwrap();
        let decompress_time = start.elapsed();
        let decompressed_len = black_box(decompressed.len());
        decompress_times.push(decompress_time);
        println!("  [flate2] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_snap(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use snap::raw::{Decoder, Encoder};
    use snap::raw::max_compress_len;

    println!("  [snap] Starting benchmark...");
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let max_len = max_compress_len(data.len());
    let mut compressed = vec![0u8; max_len];
    let mut decompressed = vec![0u8; data.len()];

    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }

    let mut encoder = Encoder::new();
    let mut decoder = Decoder::new();

    // Warm-up run
    let compressed_len = encoder.compress(black_box(data), &mut compressed).unwrap();
    let decompressed_len = decoder
        .decompress(black_box(&compressed[..compressed_len]), &mut decompressed)
        .unwrap();
    black_box(compressed_len);
    black_box(decompressed_len);

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed_len = encoder.compress(black_box(data), &mut compressed).unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed_len);
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [snap] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let decompressed_len = decoder
            .decompress(black_box(&compressed[..compressed_len]), &mut decompressed)
            .unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [snap] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lz4(data: &[u8], original_size: usize, num_runs: usize) -> BenchmarkResults {
    use lz4::block::{compress_bound, compress_to_buffer, decompress_to_buffer};

    println!("  [lz4] Starting benchmark...");
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let max_len = compress_bound(data.len()).unwrap_or(data.len());
    let mut compressed = vec![0u8; max_len];
    let mut decompressed = vec![0u8; original_size];

    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }

    // Warm-up run
    let compressed_len =
        compress_to_buffer(black_box(data), None, false, &mut compressed).unwrap();
    let decompressed_len = decompress_to_buffer(
        black_box(&compressed[..compressed_len]),
        Some(original_size as i32),
        &mut decompressed,
    )
    .unwrap();
    black_box(compressed_len);
    black_box(decompressed_len);

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed_len =
            compress_to_buffer(black_box(data), None, false, &mut compressed).unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed_len);
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [lz4] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let decompressed_len = decompress_to_buffer(
            black_box(&compressed[..compressed_len]),
            Some(original_size as i32),
            &mut decompressed,
        )
        .unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [lz4] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_zstd(data: &[u8], original_size: usize, num_runs: usize, level: i32, level_name: &str) -> BenchmarkResults {
    println!("  [zstd {}] Starting benchmark...", level_name);
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let max_len = zstd::zstd_safe::compress_bound(data.len());
    let mut compressed = vec![0u8; max_len];
    let mut decompressed = vec![0u8; original_size];

    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }

    // Warm-up run
    let compressed_len = zstd::bulk::compress_to_buffer(black_box(data), &mut compressed, level)
        .unwrap();
    let decompressed_len =
        zstd::bulk::decompress_to_buffer(black_box(&compressed[..compressed_len]), &mut decompressed)
            .unwrap();
    black_box(compressed_len);
    black_box(decompressed_len);

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed_len =
            zstd::bulk::compress_to_buffer(black_box(data), &mut compressed, level).unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed_len);
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [zstd {}] Run {}: compressed to {} bytes in {:.3}ms", 
                 level_name, run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let decompressed_len = zstd::bulk::decompress_to_buffer(
            black_box(&compressed[..compressed_len]),
            &mut decompressed,
        )
        .unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [zstd {}] Run {}: decompressed in {:.3}ms", 
                 level_name, run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
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
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let mut compressed = vec![0u8; data.len() + 1024 * 1024];
    let mut decompressed = vec![0u8; data.len()];
    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }
    compressed.clear();
    decompressed.clear();

    // Warm-up run
    let mut encoder = XzEncoder::new(black_box(&data[..]), 6);
    encoder.read_to_end(&mut compressed).unwrap();
    let mut decoder = XzDecoder::new(black_box(&compressed[..]));
    decoder.read_to_end(&mut decompressed).unwrap();
    black_box(compressed.len());
    black_box(decompressed.len());

    for run in 0..num_runs {
        // Compression
        compressed.clear();
        let start = Instant::now();
        let mut encoder = XzEncoder::new(black_box(&data[..]), 6);
        encoder.read_to_end(&mut compressed).unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed.len());
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [xz2] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        decompressed.clear();
        let start = Instant::now();
        let mut decoder = XzDecoder::new(black_box(&compressed[..]));
        decoder.read_to_end(&mut decompressed).unwrap();
        let decompress_time = start.elapsed();
        let decompressed_len = black_box(decompressed.len());
        decompress_times.push(decompress_time);
        println!("  [xz2] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lzma_rs(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use lzma_rs::lzma_compress;
    use lzma_rs::lzma_decompress;

    println!("  [lzma-rs] Starting benchmark...");
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let mut compressed = vec![0u8; data.len() + 1024 * 1024];
    let mut decompressed = vec![0u8; data.len()];
    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }
    compressed.clear();
    decompressed.clear();

    // Warm-up run
    let mut warm_input = black_box(&data[..]);
    lzma_compress(&mut warm_input, &mut compressed).unwrap();
    let mut warm_compressed = black_box(&compressed[..]);
    lzma_decompress(&mut warm_compressed, &mut decompressed).unwrap();
    black_box(compressed.len());
    black_box(decompressed.len());

    for run in 0..num_runs {
        // Compression
        compressed.clear();
        let start = Instant::now();
        let mut input = black_box(&data[..]);
        lzma_compress(&mut input, &mut compressed).unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed.len());
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [lzma-rs] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        decompressed.clear();
        let start = Instant::now();
        let mut input = black_box(&compressed[..]);
        lzma_decompress(&mut input, &mut decompressed).unwrap();
        let decompress_time = start.elapsed();
        let decompressed_len = black_box(decompressed.len());
        decompress_times.push(decompress_time);
        println!("  [lzma-rs] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_miniz_oxide(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use miniz_oxide::deflate::core::{compress, create_comp_flags_from_zip_params, CompressorOxide, TDEFLFlush, TDEFLStatus};
    use miniz_oxide::inflate::decompress_slice_iter_to_slice;

    println!("  [miniz_oxide] Starting benchmark...");
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let max_len = data.len().saturating_mul(2).saturating_add(64);
    let mut compressed = vec![0u8; max_len];
    let mut decompressed = vec![0u8; data.len()];
    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }

    let flags = create_comp_flags_from_zip_params(6, 0, 0);
    let mut compressor = CompressorOxide::new(flags);

    fn compress_into(
        compressor: &mut CompressorOxide,
        input: &[u8],
        output: &mut [u8],
    ) -> usize {
        let mut input_remaining = input;
        let mut out_pos = 0;
        loop {
            let (status, bytes_in, bytes_out) = compress(
                compressor,
                input_remaining,
                &mut output[out_pos..],
                TDEFLFlush::Finish,
            );
            out_pos += bytes_out;
            input_remaining = &input_remaining[bytes_in..];
            match status {
                TDEFLStatus::Done => return out_pos,
                TDEFLStatus::Okay => continue,
                _ => panic!("miniz_oxide compression failed"),
            }
        }
    }

    // Warm-up run
    compressor.reset();
    let compressed_len = compress_into(&mut compressor, black_box(data), &mut compressed);
    let decompressed_len = decompress_slice_iter_to_slice(
        &mut decompressed,
        std::iter::once(black_box(&compressed[..compressed_len])),
        false,
        false,
    )
    .unwrap();
    black_box(compressed_len);
    black_box(decompressed_len);

    for run in 0..num_runs {
        // Compression
        compressor.reset();
        let start = Instant::now();
        let compressed_len = compress_into(&mut compressor, black_box(data), &mut compressed);
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed_len);
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [miniz_oxide] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let decompressed_len = decompress_slice_iter_to_slice(
            &mut decompressed,
            std::iter::once(black_box(&compressed[..compressed_len])),
            false,
            false,
        )
        .unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [miniz_oxide] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lz4_flex(data: &[u8], original_size: usize, num_runs: usize) -> BenchmarkResults {
    println!("  [lz4_flex] Starting benchmark...");
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let max_len = lz4_flex::block::get_maximum_output_size(data.len());
    let mut compressed = vec![0u8; max_len];
    let mut decompressed = vec![0u8; original_size];
    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }

    // Warm-up run
    let compressed_len = lz4_flex::compress_into(black_box(data), &mut compressed).unwrap();
    let decompressed_len =
        lz4_flex::decompress_into(black_box(&compressed[..compressed_len]), &mut decompressed)
            .unwrap();
    black_box(compressed_len);
    black_box(decompressed_len);

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed_len = lz4_flex::compress_into(black_box(data), &mut compressed).unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed_len);
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [lz4_flex] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let decompressed_len =
            lz4_flex::decompress_into(black_box(&compressed[..compressed_len]), &mut decompressed)
                .unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [lz4_flex] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_libdeflate(data: &[u8], _original_size: usize, num_runs: usize) -> BenchmarkResults {
    use libdeflater::{Compressor, Decompressor, CompressionLvl};

    println!("  [libdeflate] Starting benchmark...");
    let mut input_sizes = Vec::with_capacity(num_runs);
    let mut compressed_sizes = Vec::with_capacity(num_runs);
    let mut compress_times = Vec::with_capacity(num_runs);
    let mut decompress_times = Vec::with_capacity(num_runs);

    let mut compressor = Compressor::new(CompressionLvl::default());
    let mut decompressor = Decompressor::new();
    let max_sz = compressor.deflate_compress_bound(data.len());
    let mut compressed = vec![0u8; max_sz];
    let mut decompressed = vec![0u8; data.len()];

    for byte in &mut compressed {
        *byte = 0;
    }
    for byte in &mut decompressed {
        *byte = 0;
    }

    // Warm-up run
    let compressed_len = compressor
        .deflate_compress(black_box(data), &mut compressed)
        .unwrap();
    let decompressed_len = decompressor
        .deflate_decompress(black_box(&compressed[..compressed_len]), &mut decompressed)
        .unwrap();
    black_box(compressed_len);
    black_box(decompressed_len);

    for run in 0..num_runs {
        // Compression
        let start = Instant::now();
        let compressed_len = compressor
            .deflate_compress(black_box(data), &mut compressed)
            .unwrap();
        let compress_time = start.elapsed();
        let compressed_len = black_box(compressed_len);
        compress_times.push(compress_time);
        input_sizes.push(data.len());
        compressed_sizes.push(compressed_len);
        println!("  [libdeflate] Run {}: compressed to {} bytes in {:.3}ms", 
                 run + 1, compressed_len, compress_time.as_secs_f64() * 1000.0);

        // Decompression
        let start = Instant::now();
        let decompressed_len = decompressor
            .deflate_decompress(black_box(&compressed[..compressed_len]), &mut decompressed)
            .unwrap();
        let decompress_time = start.elapsed();
        decompress_times.push(decompress_time);
        println!("  [libdeflate] Run {}: decompressed in {:.3}ms", 
                 run + 1, decompress_time.as_secs_f64() * 1000.0);
        black_box(decompressed_len);
    }

    BenchmarkResults {
        input_sizes,
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn print_results(name: &str, results: &BenchmarkResults) {
    // Guard against empty results
    if results.input_sizes.is_empty()
        || results.compressed_sizes.is_empty() 
        || results.compress_times.is_empty() 
        || results.decompress_times.is_empty() {
        eprintln!("Warning: No results to display for {}", name);
        return;
    }

    let total_input_size = results.input_sizes.iter().sum::<usize>() as f64;
    let total_compressed_size = results.compressed_sizes.iter().sum::<usize>() as f64;
    let ratio = if total_compressed_size > 0.0 {
        total_input_size / total_compressed_size
    } else {
        0.0
    };

    let total_compress_time = results
        .compress_times
        .iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>();
    let compress_throughput = if total_compress_time > 0.0 {
        (total_input_size / (1024.0 * 1024.0)) / total_compress_time
    } else {
        f64::INFINITY
    };

    let total_decompress_time = results
        .decompress_times
        .iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>();
    let decompress_throughput = if total_decompress_time > 0.0 {
        (total_input_size / (1024.0 * 1024.0)) / total_decompress_time
    } else {
        f64::INFINITY
    };

    println!(
        "| {:<20} | {:>6.2} | {:>16.2} | {:>18.2} |",
        name, ratio, compress_throughput, decompress_throughput
    );
}
