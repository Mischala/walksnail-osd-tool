#!/bin/bash
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo "Creating Linux AppImage..."

mkdir -p walksnail-osd-tool/usr/bin

cp target/release/walksnail-osd-tool walksnail-osd-tool/usr/bin/
cp ext/ffmpeg/linux64/ffmpeg walksnail-osd-tool/usr/bin/
cp resources/icons/512x512.png walksnail-osd-tool/walksnail-osd-tool.png
cp resources/icons/512x512.png walksnail-osd-tool/.DirIcon

cat > walksnail-osd-tool/AppRun << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/walksnail-osd-tool" "$@"
EOF
chmod +x walksnail-osd-tool/AppRun

cp resources/walksnail-osd-tool.desktop walksnail-osd-tool/

cd walksnail-osd-tool
LD_LIBRARY_PATH=/tmp/squashfs-root/usr/lib ARCH=x86_64 /tmp/squashfs-root/usr/bin/appimagetool .

echo "AppImage created successfully!"