use std::env;
use std::fs;
use std::io::{Read, Write};
use std::time::Instant;

// Maximum file size for lz4 (uses i32 for sizes)
const LZ4_MAX_SIZE: usize = i32::MAX as usize;

// Macro to handle errors in benchmark functions
macro_rules! handle_error {
    ($result:expr, $name:expr, $iter:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                if $iter == 0 {
                    println!(
                        "{:<20} {:>15} {:>20} {:>20}",
                        $name, "SKIPPED", "-", "-"
                    );
                    eprintln!("Warning: {} failed: {}", $name, e);
                }
                return;
            }
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    // Load file into memory
    let data = fs::read(filename).expect("Failed to read file");
    let original_size = data.len();

    println!("File: {}", filename);
    println!(
        "Original size: {} bytes ({:.2} MiB)",
        original_size,
        original_size as f64 / (1024.0 * 1024.0)
    );
    println!();
    println!(
        "{:<20} {:>15} {:>20} {:>20}",
        "Algorithm", "Ratio", "Compress (MiB/s)", "Decompress (MiB/s)"
    );
    println!("{}", "-".repeat(80));

    // Benchmark each compression algorithm
    benchmark_flate2(&data, original_size);
    benchmark_snap(&data, original_size);
    benchmark_lz4(&data, original_size);
    benchmark_zstd(&data, original_size);
    benchmark_brotli(&data, original_size);
    benchmark_bzip2(&data, original_size);
    benchmark_xz2(&data, original_size);
    benchmark_lzma_rs(&data, original_size);
    benchmark_miniz_oxide(&data, original_size);
    benchmark_lz4_flex(&data, original_size);
}

fn benchmark_flate2(data: &[u8], original_size: usize) {
    use flate2::write::{GzDecoder, GzEncoder};
    use flate2::Compression;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        handle_error!(encoder.write_all(data), "flate2 (gzip)", i);
        let compressed = handle_error!(encoder.finish(), "flate2 (gzip)", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decoder = GzDecoder::new(Vec::new());
        decoder.write_all(&compressed).unwrap();
        let _decompressed = decoder.finish().unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "flate2 (gzip)",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_snap(data: &[u8], original_size: usize) {
    use snap::raw::{Decoder, Encoder};

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = handle_error!(Encoder::new().compress_vec(data), "snap (snappy)", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = Decoder::new().decompress_vec(&compressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "snap (snappy)",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_lz4(data: &[u8], original_size: usize) {
    use lz4::block::{compress, decompress};

    // lz4 crate uses i32 for sizes, check if original_size fits
    if original_size > LZ4_MAX_SIZE {
        println!(
            "{:<20} {:>15} {:>20} {:>20}",
            "lz4", "SKIPPED", "-", "-"
        );
        eprintln!("Warning: lz4 does not support files larger than {} bytes (file size: {} bytes)", LZ4_MAX_SIZE, original_size);
        return;
    }

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = handle_error!(compress(data, None, false), "lz4", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = decompress(&compressed, Some(original_size as i32)).unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "lz4",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_zstd(data: &[u8], original_size: usize) {
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = handle_error!(zstd::encode_all(data, 3), "zstd", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = zstd::decode_all(&compressed[..]).unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "zstd",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_brotli(data: &[u8], original_size: usize) {
    use brotli::enc::BrotliEncoderParams;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let mut compressed = Vec::new();
        let params = BrotliEncoderParams::default();
        handle_error!(brotli::BrotliCompress(&mut &data[..], &mut compressed, &params), "brotli", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decompressed = Vec::new();
        brotli::BrotliDecompress(&mut &compressed[..], &mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "brotli",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_bzip2(data: &[u8], original_size: usize) {
    use bzip2::read::{BzDecoder, BzEncoder};
    use bzip2::Compression;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let mut encoder = BzEncoder::new(&data[..], Compression::default());
        let mut compressed = Vec::new();
        handle_error!(encoder.read_to_end(&mut compressed), "bzip2", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decoder = BzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "bzip2",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_xz2(data: &[u8], original_size: usize) {
    use xz2::read::{XzDecoder, XzEncoder};

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let mut encoder = XzEncoder::new(&data[..], 6);
        let mut compressed = Vec::new();
        handle_error!(encoder.read_to_end(&mut compressed), "xz2 (lzma)", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decoder = XzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "xz2 (lzma)",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_lzma_rs(data: &[u8], original_size: usize) {
    use lzma_rs::lzma_compress;
    use lzma_rs::lzma_decompress;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let mut compressed = Vec::new();
        handle_error!(lzma_compress(&mut &data[..], &mut compressed), "lzma-rs", i);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decompressed = Vec::new();
        lzma_decompress(&mut &compressed[..], &mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    print_results(
        "lzma-rs",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_miniz_oxide(data: &[u8], original_size: usize) {
    use miniz_oxide::deflate::compress_to_vec;
    use miniz_oxide::inflate::decompress_to_vec;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = compress_to_vec(data, 6);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = match decompress_to_vec(&compressed) {
            Ok(d) => d,
            Err(e) => {
                if i == 0 {
                    println!(
                        "{:<20} {:>15} {:>20} {:>20}",
                        "miniz_oxide", "SKIPPED", "-", "-"
                    );
                    eprintln!("Warning: miniz_oxide failed: {:?}", e);
                }
                return;
            }
        };
        decompress_times.push(start.elapsed());
    }

    print_results(
        "miniz_oxide",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn benchmark_lz4_flex(data: &[u8], original_size: usize) {
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for i in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = lz4_flex::compress(data);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = handle_error!(lz4_flex::decompress(&compressed, original_size), "lz4_flex", i);
        decompress_times.push(start.elapsed());
    }

    print_results(
        "lz4_flex",
        original_size,
        &compressed_sizes,
        &compress_times,
        &decompress_times,
    );
}

fn print_results(
    name: &str,
    original_size: usize,
    compressed_sizes: &[usize],
    compress_times: &[std::time::Duration],
    decompress_times: &[std::time::Duration],
) {
    let avg_compressed_size =
        compressed_sizes.iter().sum::<usize>() as f64 / compressed_sizes.len() as f64;
    let ratio = original_size as f64 / avg_compressed_size;

    let avg_compress_time =
        compress_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / compress_times.len() as f64;
    let compress_throughput = (original_size as f64 / (1024.0 * 1024.0)) / avg_compress_time;

    let avg_decompress_time = decompress_times
        .iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>()
        / decompress_times.len() as f64;
    let decompress_throughput = (original_size as f64 / (1024.0 * 1024.0)) / avg_decompress_time;

    println!(
        "{:<20} {:>15.2} {:>20.2} {:>20.2}",
        name, ratio, compress_throughput, decompress_throughput
    );
}
