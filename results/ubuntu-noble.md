Image was created using the following instructions. We truncated 0 stripes, since we skip compressing & uploading those in ubiblk.

```
wget https://cloud-images.ubuntu.com/noble/20260108/noble-server-cloudimg-amd64.img
qemu-img convert -f qcow2 -O raw ./noble-server-cloudimg-amd64.img noble-server-cloudimg-amd64.raw
cat noble-server-cloudimg-amd64.raw | python3 -c "import sys; r, w = sys.stdin.buffer, sys.stdout.buffer; [w.write(b) for b in iter(lambda: r.read(1048576), b'') if any(b)]" > noble-server-cloudimg-amd64.raw.truncated
```

```
hadi@w2295:~$ ls -lh noble-server-cloudimg-amd64.raw.truncated
-rw-rw-r-- 1 hadi hadi 1.9G Feb  5 00:10 noble-server-cloudimg-amd64.raw.truncated
```

Results using AMD EPYC 9454P

| Algorithm       | Ratio | Compress (MiB/s) | Decompress (MiB/s) |
| :-------------- | ----: | --------------: | ----------------: |
| memcpy          | 1.00  | N/A             | N/A               |
| flate2 (gzip)   | 33.82 |      305.12      |       4498.42      |
| snap (snappy)   | 23.89 |     7086.81      |      11585.72      |
| lz4             | 24.76 |     7789.13      |      17650.88      |
| zstd (level 1)  | 32.65 |     5187.22      |       7832.87      |
| zstd (level 3)  | 36.34 |     3274.57      |       7594.50      |
| zstd (level 10) | 40.75 |      728.73      |       7780.21      |
| xz2 (lzma)      | 48.37 |       39.23      |       1011.02      |
| lzma-rs         | 21.13 |      334.20      |        303.34      |
| miniz_oxide     | 33.82 |      261.05      |       4580.23      |
| lz4_flex        | 24.72 |     6910.32      |      13606.10      |
| libdeflate      | 34.31 |     1192.53      |       8025.13      |

Results using Intel Xeon W-2295

| Algorithm       | Ratio | Compress (MiB/s) | Decompress (MiB/s) |
| :-------------- | ----: | --------------: | ----------------: |
| memcpy          | 1.00  | N/A             | N/A               |
| flate2 (gzip)   | 33.82 |      349.79      |       4342.96      |
| snap (snappy)   | 23.89 |     7353.47      |      13418.70      |
| lz4             | 24.76 |     7792.10      |      20300.40      |
| zstd (level 1)  | 32.65 |     4717.49      |      10620.90      |
| zstd (level 3)  | 36.34 |     2555.76      |       9950.12      |
| zstd (level 10) | 40.75 |      561.21      |      10072.45      |
| xz2 (lzma)      | 48.37 |       37.17      |        917.39      |
| lzma-rs         | 21.13 |      365.13      |        250.63      |
| miniz_oxide     | 33.82 |      276.41      |       4497.77      |
| lz4_flex        | 24.72 |     5973.75      |      15590.63      |
| libdeflate      | 34.31 |     1143.33      |       9034.51      |
