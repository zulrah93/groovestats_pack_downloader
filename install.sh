git clone https://github.com/zulrah93/groovestats_pack_downloader
cd groovestats_pack_downloader
cargo build --release
echo "Installing groovestats_pack_downloader in sbin!"
cp /target/release/groovestats_pack_downloader /sbin/groovestats_pack_downloader
echo "Restart terminal console for the changes to take effect!"
