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

| Algorithm            |  Ratio | Compress (MiB/s) | Decompress (MiB/s) |
| -------------------- | ------ | ---------------- | ------------------ |
| memcpy               |   1.00 |          2112.84 |            2053.73 |
| flate2 (gzip)        |   3.38 |            30.52 |             451.51 |
| snap (snappy)        |   2.39 |           706.05 |            1170.72 |
| lz4                  |   2.48 |           775.08 |            1797.08 |
| zstd (level 1)       |   3.27 |           516.08 |             972.41 |
| zstd (level 3)       |   3.63 |           328.66 |             935.42 |
| zstd (level 10)      |   4.07 |            72.33 |             958.77 |
| xz2 (lzma)           |   4.84 |             3.92 |             101.09 |
| lzma-rs              |   2.11 |            40.63 |              29.86 |
| miniz_oxide          |   3.38 |            26.08 |             459.38 |
| lz4_flex             |   2.47 |           693.85 |            1357.06 |
| libdeflate           |   3.43 |           118.99 |             801.36 |

Results using Intel Xeon W-2295

| Algorithm            |  Ratio | Compress (MiB/s) | Decompress (MiB/s) |
| -------------------- | ------ | ---------------- | ------------------ |
| memcpy               |   1.00 |          2285.42 |            2305.44 |
| flate2 (gzip)        |   3.38 |            35.08 |             434.34 |
| snap (snappy)        |   2.39 |           737.22 |            1349.21 |
| lz4                  |   2.48 |           776.17 |            2021.04 |
| zstd (level 1)       |   3.27 |           474.24 |            1057.87 |
| zstd (level 3)       |   3.63 |           254.12 |             991.87 |
| zstd (level 10)      |   4.07 |            56.38 |            1000.06 |
| xz2 (lzma)           |   4.84 |             3.72 |              92.01 |
| lzma-rs              |   2.11 |            34.82 |              25.38 |
| miniz_oxide          |   3.38 |            27.69 |             450.02 |
| lz4_flex             |   2.47 |           592.77 |            1528.90 |
| libdeflate           |   3.43 |           114.04 |             903.50 |
