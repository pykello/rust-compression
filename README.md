# rust-compression

A benchmark tool for comparing 10 different Rust compression libraries based on throughput.

## Compression Libraries Tested

1. **flate2** - DEFLATE/gzip compression
2. **snap** - Snappy compression (Google)
3. **lz4** - LZ4 compression
4. **zstd** - Zstandard compression (Facebook)
5. **brotli** - Brotli compression (Google)
6. **bzip2** - BZ2 compression
7. **xz2** - LZMA/XZ compression
8. **lzma-rs** - Pure Rust LZMA implementation
9. **miniz_oxide** - Pure Rust DEFLATE implementation
10. **lz4_flex** - Pure Rust LZ4 implementation

## Usage

```bash
cargo build --release
./target/release/compression-bench <file_path>
```

## Output

The benchmark outputs three metrics for each compression algorithm:
- **Compression Ratio**: Original size / Compressed size
- **Compression Throughput**: MiB/s (average of 3 runs)
- **Decompression Throughput**: MiB/s (average of 3 runs)

Each throughput measurement is based purely on the compression/decompression algorithm itself, excluding any memory copying or file I/O overhead.

## Example

```
$ ./target/release/compression-bench test_file.txt
File: test_file.txt
Original size: 152000 bytes (0.14 MiB)

Algorithm                      Ratio     Compress (MiB/s)   Decompress (MiB/s)
--------------------------------------------------------------------------------
flate2 (gzip)                 243.20               257.04              1211.89
snap (snappy)                  20.59              4877.69              1783.78
lz4                           199.21              7606.04              1662.32
zstd                         1034.01               472.13              1542.20
brotli                       1196.85                16.73               775.20
bzip2                         380.00                 5.75               177.97
xz2 (lzma)                    513.51                23.53               456.56
lzma-rs                         2.08                38.16                34.33
miniz_oxide                   250.41               334.75              2987.99
lz4_flex                      198.95             13798.12              3684.14
```