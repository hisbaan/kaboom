# VERSION=0.0.0
# ARCH=x84_64-linux-gnu
rm target/release
cargo build --release
cp target/release/kaboom .
strip kaboom
tar -czvf kaboom-${VERSION}-${ARCH}.tar.gz kaboom
