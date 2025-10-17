# TROPIC01 USB example
A small example to interact with the USB dongle with libtropic-rs.


## Development Environment
The `flake.nix` file under project's root, can be used a manage a
development environment. To enter the development environment:

```bash
nix develop
```

## Quick start

```bash
cargo run --package tropic01-example-usb -- /dev/ttyACM0 115200
```

> [!NOTE]
> Note the two arguments: `/dev/ttyACM0` and `115200`. These may differ
> on your system. Modify accordingly if necessary.

**Example output:**

```bash
Opening TS1302 dongle on /dev/ttyACM0 @ 115200 baud
CHIP_ID ver: 0x01000000 (v1.0.0.0)
FL_PROD_DATA: 0x00000000000000000000000000000000 (N/A)
MAN_FUNC_TEST: 0x01000000000000ff (PASSED)
Silicon rev: 0x41434142 (ACAB)
Package ID: 0x80aa (QFN32, 4x4mm)
Prov info ver: 0x01 (v1)
Fab ID: 0x001 (EPS Global - Brno)
P/N ID (short P/N): 0x101
Prov date: 0x085b
HSM HW/FW/SW ver: 0x00060501 (v0.6.5.1)
Programmer ver: 0x00000000 (v0.0.0.0)
S/N: 0x02001101085b1905090d0000000003eb
  SN: 0x02
  Fab ID: 0x001
  P/N ID: 0x101
  Fabrication Date: 0x085b
  Lot ID: 0x1905090d00
  Wafer ID: 0x00
  X-Coordinate: 0x0000
  Y-Coordinate: 0x03eb
P/N (long) = 0x0D545230312D4332502D54313031FFFF (TR01-C2P-T101)
Prov template ver: 0x0104 (v1.4)
Prov template tag: 0xd8966128
Prov specification ver: 0x000c (v0.12)
Prov specification tag: 0x7deda870
Batch ID: 0x1905090d00
Cert: X509Certificate { data: [1, 4, 1, d3, 2, 62, 2, 8f, 2, 5c, 30, 82, 1, cf, 30, 82, 1, 55, a0, 3, 2, 1, 2, 2, 10, 2, 0, 11, 1, 8, 5b, 19, 5, 9, d, 0, 0, 0, 0, 3, eb, 30, a, 6, 8, 2a, 86, 48, ce, 3d, 4, 3, 3, 30, 47, 31, b, 30, 9, 6, 3, 55, 4, 6, 13, 2, 43, 5a, 31, 1d, 30, 1b, 6, 3, 55, 4, a, c, 14, 54, 72, 6f, 70, 69, 63, 20, 53, 71, 75, 61, 72, 65, 20, 73, 2e, 72, 2e, 6f, 2e, 31, 19, 30, 17, 6, 3, 55, 4, 3, c, 10, 54, 52, 4f, 50, 49, 43, 30, 31, 2d, 54, 20, 43, 41, 20, 76, 31, 30, 1e, 17, d, 32, 35, 30, 35, 31, 39, 31, 32, 31, 37, 31, 35, 5a, 17, d, 34, 35, 30, 35, 31, 39, 31, 32, 31, 37, 31, 35, 5a, 30, 17, 31, 15, 30, 13, 6, 3, 55, 4, 3, c, c, 54, 52, 4f, 50, 49, 43, 30, 31, 20, 65, 53, 45, 30, 2a, 30, 5, 6, 3, 2b, 65, 6e, 3, 21, 0, dd, b4, cf, 37, 7b, d3, e5, 1, d1, 4a, 27, 84, 73, ea, dc, a5, 96, ed, 79, dd, 9e, 4, aa, 44, c7, 66, 5a, 5c, 78, 92, 8d, 5b, a3, 81, 81, 30, 7f, 30, c, 6, 3, 55, 1d, 13, 1, 1, ff, 4, 2, 30, 0, 30, e, 6, 3, 55, 1d, f, 1, 1, ff, 4, 4, 3, 2, 3, 8, 30, 1f, 6, 3, 55, 1d, 23, 4, 18, 30, 16, 80, 14, 33, c7, 11, 6, c, e8, 5, 13, b5, 67, 7b, 1, 96, 50, 64, 4e, 3b, 43, fa, e7, 30, 3e, 6, 3, 55, 1d, 1f, 1, 1, 0, 4, 34, 30, 32, 30, 30, a0, 2e, a0, 2c, 86, 2a, 68, 74, 74, 70, 3a, 2f, 2f, 70, 6b, 69, 2e, 74, 72, 6f, 70, 69, 63, 73, 71, 75, 61, 72, 65, 2e, 63, 6f, 6d, 2f, 6c, 33, 2f, 74, 30, 31, 2d, 54, 76, 31, 2e, 63, 72, 6c, 30, a, 6, 8, 2a, 86, 48, ce, 3d, 4, 3, 3, 3, 68, 0, 30, 65, 2, 30, 12, d4, a2, 71, c8, e9, 82, 1d, 3d, 68, 4e, e5, 9a, 20, 41, cc, 52, df, c4, 22, 71, 5d, 9e, a8, d8, a0, 9b, 96, bd, fc, 4e, eb, 43, 34, 14, 82, 54, 90, 83, 2a, 35, a9, 40, 72, 6a, d0, 99, 29, 2, 31, 0, e1, ff, 3d, c8, 53, 1, 3a, 2f, 64, 92, d9, 46, 9d, 60, 21, bd, 18, 1b, 23, dd, 9a, ec, 1b, 55, d4, 35, c6, ea, 24, 68, ad, 8d, 3f, ba, 8d, a, 48, a6, 4c, f7, 1d, f0, d3, ad, 9b, aa, eb, 7c, 30, 82, 2, 5e, 30, 82, 1, e4, a0, 3, 2, 1, 2, 2, 2, 75, 31, 30, a, 6, 8, 2a, 86, 48, ce, 3d, 4, 3, 3, 30, 45, 31, b, 30, 9] }
Example completed successfully!
```

## Build & Run
To build the example, (from the libtropic-rs project root):


```bash
cargo build --package tropic01-example-usb
```

To run:

```bash
./target/debug/tropic01-example-usb
```
