# For changes that touch the Rust backend, use this script to redploy
#
#
cd ..
cargo +stable build --locked --release --bin partkeepr-api
systemctl --user stop partkeepr-ng
cp target/release/partkeepr-api local-install/native/bin/
systemctl --user start partkeepr-ng
systemctl --user status partkeepr-ng
