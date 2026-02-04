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
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    // Get file size
    let metadata = fs::metadata(filename).expect("Failed to read file metadata");
    let original_size = metadata.len() as usize;

    println!("File: {}", filename);
    println!(
        "Original size: {} bytes ({:.2} MiB)",
        original_size,
        original_size as f64 / (1024.0 * 1024.0)
    );
    println!();

    // Process file in chunks
    let mut file = fs::File::open(filename).expect("Failed to open file");
    let mut chunk_number = 0;

    let mut flate2_results = BenchmarkResults::new();
    let mut snap_results = BenchmarkResults::new();
    let mut lz4_results = BenchmarkResults::new();
    let mut zstd_results = BenchmarkResults::new();
    let mut brotli_results = BenchmarkResults::new();
    let mut bzip2_results = BenchmarkResults::new();
    let mut xz2_results = BenchmarkResults::new();
    let mut lzma_rs_results = BenchmarkResults::new();
    let mut miniz_oxide_results = BenchmarkResults::new();
    let mut lz4_flex_results = BenchmarkResults::new();

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
        flate2_results.merge(benchmark_flate2(&chunk, bytes_read));
        snap_results.merge(benchmark_snap(&chunk, bytes_read));
        lz4_results.merge(benchmark_lz4(&chunk, bytes_read));
        zstd_results.merge(benchmark_zstd(&chunk, bytes_read));
        brotli_results.merge(benchmark_brotli(&chunk, bytes_read));
        bzip2_results.merge(benchmark_bzip2(&chunk, bytes_read));
        xz2_results.merge(benchmark_xz2(&chunk, bytes_read));
        lzma_rs_results.merge(benchmark_lzma_rs(&chunk, bytes_read));
        miniz_oxide_results.merge(benchmark_miniz_oxide(&chunk, bytes_read));
        lz4_flex_results.merge(benchmark_lz4_flex(&chunk, bytes_read));
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
    print_results("zstd", original_size, &zstd_results);
    print_results("brotli", original_size, &brotli_results);
    print_results("bzip2", original_size, &bzip2_results);
    print_results("xz2 (lzma)", original_size, &xz2_results);
    print_results("lzma-rs", original_size, &lzma_rs_results);
    print_results("miniz_oxide", original_size, &miniz_oxide_results);
    print_results("lz4_flex", original_size, &lz4_flex_results);
}

fn benchmark_flate2(data: &[u8], _original_size: usize) -> BenchmarkResults {
    use flate2::write::{GzDecoder, GzEncoder};
    use flate2::Compression;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        let compressed = encoder.finish().unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decoder = GzDecoder::new(Vec::new());
        decoder.write_all(&compressed).unwrap();
        let _decompressed = decoder.finish().unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_snap(data: &[u8], _original_size: usize) -> BenchmarkResults {
    use snap::raw::{Decoder, Encoder};

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = Encoder::new().compress_vec(data).unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = Decoder::new().decompress_vec(&compressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lz4(data: &[u8], original_size: usize) -> BenchmarkResults {
    use lz4::block::{compress, decompress};

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = compress(data, None, false).unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = decompress(&compressed, Some(original_size as i32)).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_zstd(data: &[u8], _original_size: usize) -> BenchmarkResults {
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = zstd::encode_all(data, 3).unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = zstd::decode_all(&compressed[..]).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_brotli(data: &[u8], _original_size: usize) -> BenchmarkResults {
    use brotli::enc::BrotliEncoderParams;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let mut compressed = Vec::new();
        let params = BrotliEncoderParams::default();
        brotli::BrotliCompress(&mut &data[..], &mut compressed, &params).unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decompressed = Vec::new();
        brotli::BrotliDecompress(&mut &compressed[..], &mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_bzip2(data: &[u8], _original_size: usize) -> BenchmarkResults {
    use bzip2::read::{BzDecoder, BzEncoder};
    use bzip2::Compression;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let mut encoder = BzEncoder::new(&data[..], Compression::default());
        let mut compressed = Vec::new();
        encoder.read_to_end(&mut compressed).unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decoder = BzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_xz2(data: &[u8], _original_size: usize) -> BenchmarkResults {
    use xz2::read::{XzDecoder, XzEncoder};

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let mut encoder = XzEncoder::new(&data[..], 6);
        let mut compressed = Vec::new();
        encoder.read_to_end(&mut compressed).unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decoder = XzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lzma_rs(data: &[u8], _original_size: usize) -> BenchmarkResults {
    use lzma_rs::lzma_compress;
    use lzma_rs::lzma_decompress;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let mut compressed = Vec::new();
        lzma_compress(&mut &data[..], &mut compressed).unwrap();
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let mut decompressed = Vec::new();
        lzma_decompress(&mut &compressed[..], &mut decompressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_miniz_oxide(data: &[u8], _original_size: usize) -> BenchmarkResults {
    use miniz_oxide::deflate::compress_to_vec;
    use miniz_oxide::inflate::decompress_to_vec;

    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = compress_to_vec(data, 6);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = decompress_to_vec(&compressed).unwrap();
        decompress_times.push(start.elapsed());
    }

    BenchmarkResults {
        compressed_sizes,
        compress_times,
        decompress_times,
    }
}

fn benchmark_lz4_flex(data: &[u8], original_size: usize) -> BenchmarkResults {
    let mut compressed_sizes = Vec::new();
    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();

    for _ in 0..3 {
        // Compression
        let start = Instant::now();
        let compressed = lz4_flex::compress(data);
        compress_times.push(start.elapsed());
        compressed_sizes.push(compressed.len());

        // Decompression
        let start = Instant::now();
        let _decompressed = lz4_flex::decompress(&compressed, original_size).unwrap();
        decompress_times.push(start.elapsed());
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
