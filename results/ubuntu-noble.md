Image was created using the following instructions. We truncated stripes with all zeros, since we skip compressing & uploading those in ubiblk.

```
wget https://cloud-images.ubuntu.com/noble/20260108/noble-server-cloudimg-amd64.img
qemu-img convert -f qcow2 -O raw ./noble-server-cloudimg-amd64.img noble-server-cloudimg-amd64.raw
cat noble-server-cloudimg-amd64.raw | python3 -c "import sys; r, w = sys.stdin.buffer, sys.stdout.buffer; [w.write(b) for b in iter(lambda: r.read(1048576), b'') if any(b)]" > noble-server-cloudimg-amd64.raw.truncated
```

```
hadi@w2295:~$ ls -lh noble-server-cloudimg-amd64.raw.truncated
-rw-rw-r-- 1 hadi hadi 1.9G Feb  5 00:10 noble-server-cloudimg-amd64.raw.truncated
```

Results on Hetzner AX162-R (AMD EPYC 9454P)

| Algorithm            |  Ratio | Compress (MiB/s) | Decompress (MiB/s) |
| -------------------- | ------ | ---------------- | ------------------ |
| memcpy               |   1.00 |         16713.15 |           16078.12 |
| flate2 (gzip)        |   3.38 |            30.71 |             547.92 |
| snap (snappy)        |   2.39 |           793.94 |            2011.19 |
| lz4                  |   2.48 |           888.62 |            5111.76 |
| zstd (level 1)       |   3.27 |           628.23 |            1824.17 |
| zstd (level 3)       |   3.66 |           391.41 |            1666.28 |
| zstd (level 10)      |   4.08 |            81.92 |            1802.45 |
| xz2 (lzma)           |   4.82 |             4.02 |             104.80 |
| lzma-rs              |   2.11 |            33.35 |              30.10 |
| miniz_oxide          |   3.38 |            26.42 |             632.38 |
| lz4_flex             |   2.47 |           782.56 |            2749.76 |
| libdeflate           |   3.43 |           120.33 |            1151.95 |

Results on an auction Hetzner server with Intel Xeon W-2295

| Algorithm            |  Ratio | Compress (MiB/s) | Decompress (MiB/s) |
| -------------------- | ------ | ---------------- | ------------------ |
| memcpy               |   1.00 |          9934.62 |           10129.61 |
| flate2 (gzip)        |   3.38 |            35.07 |             497.44 |
| snap (snappy)        |   2.39 |           806.31 |            2011.72 |
| lz4                  |   2.48 |           858.44 |            4015.51 |
| zstd (level 1)       |   3.27 |           596.11 |            1674.48 |
| zstd (level 3)       |   3.66 |           301.66 |            1444.81 |
| zstd (level 10)      |   4.08 |            66.27 |            1408.33 |
| xz2 (lzma)           |   4.82 |             3.73 |              94.66 |
| lzma-rs              |   2.11 |            32.87 |              23.14 |
| miniz_oxide          |   3.38 |            27.61 |             531.36 |
| lz4_flex             |   2.47 |           611.43 |            2539.84 |
| libdeflate           |   3.43 |           114.93 |            1167.58 |

Result on my laptop (12th Gen Intel Core i5-1240P)

| Algorithm            |  Ratio | Compress (MiB/s) | Decompress (MiB/s) |
| -------------------- | ------ | ---------------- | ------------------ |
| memcpy               |   1.00 |         19184.69 |           19515.49 |
| flate2 (gzip)        |   3.38 |            39.58 |             547.72 |
| snap (snappy)        |   2.39 |           895.79 |            2086.26 |
| lz4                  |   2.48 |           893.36 |            5063.36 |
| zstd (level 1)       |   3.27 |           726.56 |            1933.35 |
| zstd (level 3)       |   3.66 |           427.02 |            1837.06 |
| zstd (level 10)      |   4.08 |            75.85 |            1908.78 |
| xz2 (lzma)           |   4.82 |             4.48 |             111.74 |
| lzma-rs              |   2.11 |            35.60 |              31.16 |
| miniz_oxide          |   3.38 |            34.33 |             626.95 |
| lz4_flex             |   2.47 |           803.08 |            3293.02 |
| libdeflate           |   3.43 |           142.25 |            1208.51 |
