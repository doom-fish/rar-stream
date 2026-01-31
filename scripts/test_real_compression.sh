#!/bin/bash
# Real-world RAR compression/decompression test
# Tests that our Rust implementation correctly decompresses files created by WinRAR/rar

set -e

cd "$(dirname "$0")/.."

TESTDIR=$(mktemp -d)
trap "rm -rf $TESTDIR" EXIT

echo "🧪 rar-stream Real Compression Tests"
echo "====================================="
echo "Test directory: $TESTDIR"
echo ""

# Generate test data
echo "📝 Generating test files..."

# 1. Text file (good for PPMd)
cat > "$TESTDIR/lorem.txt" << 'EOF'
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor 
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud 
exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute 
irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla 
pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia 
deserunt mollit anim id est laborum.

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor 
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud 
exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
EOF

# 2. Binary file (random data)
dd if=/dev/urandom of="$TESTDIR/random.bin" bs=1024 count=10 2>/dev/null

# 3. Repetitive data (good for LZSS)
python3 -c "print('ABCD' * 1000)" > "$TESTDIR/repeat.txt"

# Save original checksums
echo "📊 Original file checksums:"
md5sum "$TESTDIR/lorem.txt" "$TESTDIR/random.bin" "$TESTDIR/repeat.txt" | tee "$TESTDIR/original.md5"
echo ""

# Test function
test_compression() {
    local name="$1"
    local rar_opts="$2"
    local rarfile="$TESTDIR/test_${name}.rar"
    
    echo "📦 Testing: $name ($rar_opts)"
    
    # Create RAR archive
    (cd "$TESTDIR" && rar a $rar_opts "test_${name}.rar" lorem.txt random.bin repeat.txt >/dev/null 2>&1) || {
        echo "  ❌ FAIL: RAR creation failed"
        return 1
    }
    
    if [ ! -f "$rarfile" ]; then
        echo "  ❌ FAIL: RAR file not created"
        return 1
    fi
    
    local size=$(stat -c%s "$rarfile")
    echo "  Archive size: $size bytes"
    
    # Test with our library (Node.js)
    local result
    result=$(node -e "
    const { LocalFileMedia, RarFilesPackage } = require('.');
    const crypto = require('crypto');
    
    async function test() {
        const media = new LocalFileMedia('$rarfile');
        const pkg = new RarFilesPackage([media]);
        const files = await pkg.parse();
        
        const results = [];
        for (const file of files) {
            try {
                const data = await file.readToEnd();
                const hash = crypto.createHash('md5').update(data).digest('hex');
                results.push({ name: file.name, size: data.length, hash });
            } catch (e) {
                results.push({ name: file.name, error: e.message });
            }
        }
        console.log(JSON.stringify(results));
    }
    test().catch(e => { console.error(e); process.exit(1); });
    " 2>&1) || {
        echo "  ❌ FAIL: Decompression error: $result"
        return 1
    }
    
    echo "$result" > "$TESTDIR/result_${name}.json"
    
    # Verify results
    local failed=0
    for f in lorem.txt random.bin repeat.txt; do
        local expected_hash=$(grep "$f" "$TESTDIR/original.md5" | awk '{print $1}')
        local actual_hash=$(echo "$result" | node -e "
            let d=''; process.stdin.on('data',c=>d+=c); process.stdin.on('end',()=>{
                const r = JSON.parse(d);
                const f = r.find(x => x.name === '$f');
                console.log(f ? (f.hash || 'ERROR:' + f.error) : 'NOT_FOUND');
            });
        ")
        
        if [ "$expected_hash" = "$actual_hash" ]; then
            echo "  ✅ $f: OK"
        else
            echo "  ❌ $f: MISMATCH (expected $expected_hash, got $actual_hash)"
            failed=1
        fi
    done
    
    return $failed
}

# Run tests for different compression methods
echo ""
echo "🔬 Running compression tests..."
echo ""

PASS=0
FAIL=0

# Store (no compression)
if test_compression "store" "-m0"; then
    ((PASS++))
else
    ((FAIL++))
fi
echo ""

# LZSS fastest
if test_compression "lzss_fast" "-m1"; then
    ((PASS++))
else
    ((FAIL++))
fi
echo ""

# LZSS normal (default)
if test_compression "lzss_normal" "-m3"; then
    ((PASS++))
else
    ((FAIL++))
fi
echo ""

# LZSS best
if test_compression "lzss_best" "-m5"; then
    ((PASS++))
else
    ((FAIL++))
fi
echo ""

# Summary
echo "====================================="
echo "📊 Results: $PASS passed, $FAIL failed"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed!"
    exit 1
fi
