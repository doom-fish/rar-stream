#!/bin/bash
# Verify decompression of large files with MD5 checksum

set -e
cd "$(dirname "$0")/.."

FIXTURES="__fixtures__/large"
EXPECTED_MD5="c25f3f72a00b388ea20b6ee35a33e8ca"

echo "=== Testing LZSS decompression ==="
# Extract with unrar and check MD5
cd "$FIXTURES"
rm -f extracted_lzss.tar extracted_ppmd.tar

unrar x -o+ alpine_lzss.rar extracted_lzss.tar 2>/dev/null
LZSS_MD5=$(md5sum extracted_lzss.tar | cut -d' ' -f1)
echo "LZSS extracted MD5: $LZSS_MD5"
echo "Expected MD5:       $EXPECTED_MD5"
if [ "$LZSS_MD5" = "$EXPECTED_MD5" ]; then
    echo "✓ LZSS: PASS"
else
    echo "✗ LZSS: FAIL"
fi

echo ""
echo "=== Testing PPMd decompression ==="
unrar x -o+ alpine_ppmd.rar extracted_ppmd.tar 2>/dev/null
PPMD_MD5=$(md5sum extracted_ppmd.tar | cut -d' ' -f1)
echo "PPMd extracted MD5: $PPMD_MD5"
echo "Expected MD5:       $EXPECTED_MD5"
if [ "$PPMD_MD5" = "$EXPECTED_MD5" ]; then
    echo "✓ PPMd: PASS"
else
    echo "✗ PPMd: FAIL"
fi

rm -f extracted_lzss.tar extracted_ppmd.tar
