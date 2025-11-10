# TROPIC01 USB example
A small example to interact with the USB dongle with libtropic-rs.


## Development Environment
The `flake.nix` file under project's root, can be used a manage a
development environment. To enter the development environment:

```bash
nix develop
```

## Quick start
> [!NOTE]
> Note the two optional arguments: `/dev/ttyACM0` and `115200`. These may differ
> on your system. Modify accordingly if necessary.

Current examples are under the [./examples](./examples) directory, and include:

* `get-certs` to read the certificate store and output them to der files
* `get-device-cert` to read the device certificate and show its information on screen
* `get-device-pubkey` to read the public key from the device certificate
* `show-chip-id` to read the chip identification data, such as the unique serial number
* `verify-certs` to read the certificate chain, validate it, and veritfy the signatures


### Show Chip Identification Information

```bash
cargo run --example show-chip-id
```

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
```

### Verify Certificate Chain Information
This currently shows the information of the certificate chain, validates it,
and verifies the signature of the self-signed root certificate.

```bash
cargo run --example verify-certs
```
**Example output:**

```bash
Opening TS1302 dongle on /dev/ttyACM0 @ 115200 baud
Cert store sizes: [467, 610, 655, 604]

------------------------------------------------------------------
Certificate 3, DER (604 bytes)
  Version: V3
  Serial: 01:2d
  Subject: C=CZ, O=Tropic Square s.r.o., CN=Tropic Square Root CA v1
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=Tropic Square Root CA v1
  Version: V3
  Serial: 01:2d
  Subject: C=CZ, O=Tropic Square s.r.o., CN=Tropic Square Root CA v1
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=Tropic Square Root CA v1
  Validity:
    NotBefore: Mar 31 12:08:25 2025 +00:00
    NotAfter:  Mar 31 12:08:25 2075 +00:00
    is_valid:  true
  Subject Public Key Info:
    Public Key Algorithm:
      Oid: id-ecPublicKey
      Parameter: <PRESENT> secp521r1
00000000	2b 81 04 00 23                                  	+�..#
    EC Public Key: (528 bit)
        04:01:87:cc:ea:62:83:7e:23:09:2d:8a:71:35:78:9f:
        cc:6f:bc:3d:35:e7:9f:c0:1f:4f:49:8f:c5:c2:c4:09:
        ce:77:2f:90:13:40:09:04:03:e8:ba:4d:97:e1:3f:1e:
        75:94:ac:6d:2f:51:fd:22:39:f8:d4:57:76:9f:37:84:
        40:a1:80:00:71:2b:f1:6a:48:ea:20:25:83:7b:ef:d0:
        50:2a:56:2f:d9:39:41:d5:2c:c4:0e:d9:55:3c:a7:9b:
        14:5b:a5:85:f3:24:92:bf:d7:92:eb:96:d9:49:d3:16:
        76:cd:09:9f:19:ce:88:48:69:7b:8c:34:30:af:01:6f:
        ed:98:5e:1e:b4:
  Signature Algorithm: ECDSA
  Signature Value:
      30:81:86:02:41:68:41:83:73:39:33:7c:18:2a:4e:e8:
      96:cb:fd:5d:a5:92:5f:00:26:e7:a6:fa:3d:ee:61:f4:
      9a:46:b5:d9:85:68:58:d3:d8:65:01:be:64:b0:f2:f3:
      3b:05:d8:56:de:96:f5:7b:94:7f:49:e7:20:e8:75:09:
      0b:30:c3:37:79:18:02:41:2f:dd:b6:8d:16:65:10:45:
      1f:e4:c6:2d:ba:e0:cc:d9:52:dc:34:e0:3a:e6:61:78:
      18:cc:d0:ea:28:a9:df:f0:45:aa:13:a2:48:a5:f0:66:
      b5:11:39:c9:be:f4:71:dd:00:4d:ac:4f:78:db:56:cf:
      7b:3e:8d:6f:8f:87:d0:48:d2:
  Extensions:
    [crit:false l:22] subjectKeyIdentifier: 
      X509v3 Subject Key Identifier: 3c:18:af:71:1a:66:99:b3:79:14:e3:63:96:3f:e2:5c:f3:04:b3:bf
    [crit:true l:5] basicConstraints: 
      X509v3 CA: true
    [crit:true l:4] keyUsage: 
      X509v3 Key Usage: Key Cert Sign, CRL Sign
Structure validation status: Ok
  [W] year >= 2050 should use GeneralizedTime (notAfter)

Signature verification: OK

------------------------------------------------------------------
Certificate 2, DER (655 bytes)
  Version: V3
  Serial: 0b:b9
  Subject: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01 CA v1
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=Tropic Square Root CA v1
  Version: V3
  Serial: 0b:b9
  Subject: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01 CA v1
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=Tropic Square Root CA v1
  Validity:
    NotBefore: Mar 31 12:08:29 2025 +00:00
    NotAfter:  Mar 31 12:08:29 2065 +00:00
    is_valid:  true
  Subject Public Key Info:
    Public Key Algorithm:
      Oid: id-ecPublicKey
      Parameter: <PRESENT> secp384r1
00000000	2b 81 04 00 22                                  	+�.."
    EC Public Key: (384 bit)
        04:23:01:be:5b:6e:d9:a8:58:15:3f:57:c6:be:bc:9f:
        37:b8:58:bc:28:74:dd:c9:0c:10:41:be:6d:04:e7:bb:
        f2:4a:79:68:f2:e5:11:73:d0:ac:ac:89:2e:65:e4:fc:
        03:ea:5b:c4:38:1a:60:15:4d:7c:d7:cc:6d:f9:45:91:
        65:0f:5f:dc:00:89:19:15:73:14:fc:1f:8d:82:95:f1:
        a1:05:71:dd:15:73:e8:68:bf:ec:a9:6c:92:cc:bb:81:
        6f:
  Signature Algorithm: ECDSA
  Signature Value:
      30:81:88:02:42:00:bc:d0:2d:46:43:29:f3:fc:7d:c8:
      17:23:b0:c2:64:37:e3:5c:2b:49:78:2b:ae:97:43:27:
      89:f5:08:b5:a2:20:24:0e:6e:3e:4d:12:c1:5c:3b:db:
      15:a8:d3:f9:0c:dd:19:07:1e:22:27:c4:89:82:20:b2:
      be:f5:84:b2:c2:0f:8f:02:42:01:eb:85:4f:05:f9:a2:
      c5:b4:66:d7:98:fe:62:7c:53:9b:98:70:35:31:73:5f:
      7a:b4:95:46:fe:5c:fb:9d:f0:bf:3b:69:85:d7:00:ef:
      bc:36:df:3f:f0:16:92:f0:ec:e9:8b:b8:db:2f:bb:9b:
      f4:09:13:ea:87:ea:12:1a:7a:d2:e7:
  Extensions:
    [crit:false l:22] subjectKeyIdentifier: 
      X509v3 Subject Key Identifier: 43:ba:b7:bd:a7:cd:e7:28:94:5c:f1:42:cb:d2:f9:cd:55:88:a9:3f
    [crit:true l:8] basicConstraints: 
      X509v3 CA: true
    [crit:true l:4] keyUsage: 
      X509v3 Key Usage: Key Cert Sign, CRL Sign
    [crit:false l:24] authorityKeyIdentifier: 
      X509v3 Authority Key Identifier
        Key Identifier: 3c:18:af:71:1a:66:99:b3:79:14:e3:63:96:3f:e2:5c:f3:04:b3:bf
    [crit:false l:50] crlDistributionPoints: 
      X509v3 CRL Distribution Points:
        Full Name: FullName([URI("http://pki.tropicsquare.com/l1/tsrv1.crl")])

Structure validation status: Ok
  [W] year >= 2050 should use GeneralizedTime (notAfter)

Signature verification: OK

------------------------------------------------------------------
Certificate 1, DER (610 bytes)
  Version: V3
  Serial: 75:31
  Subject: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01-T CA v1
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01 CA v1
  Version: V3
  Serial: 75:31
  Subject: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01-T CA v1
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01 CA v1
  Validity:
    NotBefore: Mar 31 12:08:30 2025 +00:00
    NotAfter:  Mar 31 12:08:30 2060 +00:00
    is_valid:  true
  Subject Public Key Info:
    Public Key Algorithm:
      Oid: id-ecPublicKey
      Parameter: <PRESENT> secp384r1
00000000	2b 81 04 00 22                                  	+�.."
    EC Public Key: (384 bit)
        04:a7:0c:32:73:ae:32:27:dc:76:7e:f0:29:3d:95:cc:
        10:66:91:e5:bc:9a:a6:c0:28:2b:aa:8f:d4:b3:7c:fa:
        c3:0f:ee:0d:87:9c:32:d8:d9:ce:9b:0b:d7:92:4b:5c:
        10:09:7b:8c:4a:5e:7e:d6:8d:69:01:85:e3:d1:28:16:
        12:56:c0:10:33:c0:29:3d:e3:9a:71:88:a7:2c:ff:9e:
        ea:f5:b3:b5:de:e8:98:f9:54:c4:f2:26:c2:ad:e7:0b:
        c6:
  Signature Algorithm: ECDSA
  Signature Value:
      30:65:02:30:14:ae:c5:25:e5:e8:31:1b:5d:63:12:cf:
      0e:bb:22:86:70:05:52:ee:ba:32:d6:41:67:2c:20:f0:
      2a:61:2b:77:e9:fc:37:09:c9:65:7c:ec:6d:82:d6:cd:
      bd:de:57:c4:02:31:00:bb:9b:77:cc:bb:d9:de:11:08:
      64:81:d4:ba:97:72:c3:87:43:59:1e:72:2b:9e:4d:08:
      a8:99:40:d8:79:da:24:47:a5:5c:15:f8:41:75:c4:79:
      94:63:26:e0:f4:82:af:
  Extensions:
    [crit:false l:22] subjectKeyIdentifier: 
      X509v3 Subject Key Identifier: 33:c7:11:06:0c:e8:05:13:b5:67:7b:01:96:50:64:4e:3b:43:fa:e7
    [crit:true l:8] basicConstraints: 
      X509v3 CA: true
    [crit:true l:4] keyUsage: 
      X509v3 Key Usage: Key Cert Sign, CRL Sign
    [crit:false l:24] authorityKeyIdentifier: 
      X509v3 Authority Key Identifier
        Key Identifier: 43:ba:b7:bd:a7:cd:e7:28:94:5c:f1:42:cb:d2:f9:cd:55:88:a9:3f
    [crit:false l:50] crlDistributionPoints: 
      X509v3 CRL Distribution Points:
        Full Name: FullName([URI("http://pki.tropicsquare.com/l2/t01v1.crl")])

Structure validation status: Ok
  [W] year >= 2050 should use GeneralizedTime (notAfter)

Signature verification: OK

------------------------------------------------------------------
Certificate 0, DER (467 bytes)
  Version: V3
  Serial: 02:00:11:01:08:5b:19:05:09:0d:00:00:00:00:03:eb
  Subject: CN=TROPIC01 eSE
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01-T CA v1
  Version: V3
  Serial: 02:00:11:01:08:5b:19:05:09:0d:00:00:00:00:03:eb
  Subject: CN=TROPIC01 eSE
  Issuer: C=CZ, O=Tropic Square s.r.o., CN=TROPIC01-T CA v1
  Validity:
    NotBefore: May 19 12:17:15 2025 +00:00
    NotAfter:  May 19 12:17:15 2045 +00:00
    is_valid:  true
  Subject Public Key Info:
    Public Key Algorithm:
      Oid: 1.3.101.110
      Parameter: <ABSENT>
    Unknown key type
00000000	dd b4 cf 37 7b d3 e5 01 d1 4a 27 84 73 ea dc a5 	ݴ�7{��.�J'�s�ܥ
00000010	96 ed 79 dd 9e 04 aa 44 c7 66 5a 5c 78 92 8d 5b 	��yݞ.�D�fZ\x��[
  Signature Algorithm: ECDSA
  Signature Value:
      30:65:02:30:12:d4:a2:71:c8:e9:82:1d:3d:68:4e:e5:
      9a:20:41:cc:52:df:c4:22:71:5d:9e:a8:d8:a0:9b:96:
      bd:fc:4e:eb:43:34:14:82:54:90:83:2a:35:a9:40:72:
      6a:d0:99:29:02:31:00:e1:ff:3d:c8:53:01:3a:2f:64:
      92:d9:46:9d:60:21:bd:18:1b:23:dd:9a:ec:1b:55:d4:
      35:c6:ea:24:68:ad:8d:3f:ba:8d:0a:48:a6:4c:f7:1d:
      f0:d3:ad:9b:aa:eb:7c:
  Extensions:
    [crit:true l:2] basicConstraints: 
      X509v3 CA: false
    [crit:true l:4] keyUsage: 
      X509v3 Key Usage: Key Agreement
    [crit:false l:24] authorityKeyIdentifier: 
      X509v3 Authority Key Identifier
        Key Identifier: 33:c7:11:06:0c:e8:05:13:b5:67:7b:01:96:50:64:4e:3b:43:fa:e7
    [crit:false l:52] crlDistributionPoints: 
      X509v3 CRL Distribution Points:
        Full Name: FullName([URI("http://pki.tropicsquare.com/l3/t01-Tv1.crl")])

Structure validation status: Ok
  [W] Unknown public key type

Signature verification: OK
```
