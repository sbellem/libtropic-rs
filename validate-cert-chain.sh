#!/bin/bash

xxxx_ca_cert_sn_30001="tropic01_xxxx_ca_certificate_sn_30001.pem"
ca_cert_sn_3001="tropic01_ca_certificate_sn_3001.pem"
root_ca_cert_sn_301="tropicsquare_root_ca_certificate_sn_301.pem"

# Download certificate authorities from Tropic Square PKI web
if [ ! -f "${xxxx_ca_cert_sn_30001}" ]; then
    curl -O "http://pki.tropicsquare.com/l0/${xxxx_ca_cert_sn_30001}"
fi
if [ ! -f "${ca_cert_sn_3001}" ]; then
    curl -O "http://pki.tropicsquare.com/l0/${ca_cert_sn_3001}"
fi
if [ ! -f "${root_ca_cert_sn_301}" ]; then
    curl -O "http://pki.tropicsquare.com/l0/${root_ca_cert_sn_301}"
fi

# Parse CRLs from certificates read from device in previous example
L3=$(openssl x509 -in t01_ese_cert.der -inform DER -text | grep URI | cut -d ':' -f 2-)
L2=$(openssl x509 -in t01_xxxx_ca_cert.der -inform DER -text | grep URI | cut -d ':' -f 2-)
L1=$(openssl x509 -in t01_ca_cert.der -inform DER -text | grep URI | cut -d ':' -f 2-)

echo "$L3, $L2, $L1"

# Download CRLs
if [ -n "${L3}" ]; then curl -O "${L3}"; fi        # Downloads t01-Tv1.crl
if [ -n "${L2}" ]; then curl -O "${L2}"; fi        # Downloads t01v1.crl
if [ -n "${L1}" ]; then curl -O "${L1}"; fi        # Downloads tsrv1.crl

# Validate (chip) device certificate
cat "${xxxx_ca_cert_sn_30001}" t01-Tv1.crl \
    "${ca_cert_sn_3001}" t01v1.crl \
    "${root_ca_cert_sn_301}" tsrv1.crl > chain.pem
 
openssl verify -verbose -crl_check -CAfile chain.pem t01_ese_cert.der

# Validate the "Part Number (group)" certificate
cat "${ca_cert_sn_3001}" t01v1.crl \
    "${root_ca_cert_sn_301}" tsrv1.crl > chain.pem

openssl verify -verbose -crl_check -CAfile chain.pem t01_xxxx_ca_cert.der

# Validate the "Product (\PartName{})" certificate
cat "${ca_cert_sn_3001}" t01v1.crl \
    "${root_ca_cert_sn_301}" tsrv1.crl > chain.pem

openssl verify -verbose -crl_check -CAfile chain.pem t01_ca_cert.der

# Validate Tropic Square Root Certificate
# Out-of-band validation of Root Certificate is not included
openssl verify -verbose -CAfile "${root_ca_cert_sn_301}" "${root_ca_cert_sn_301}"
