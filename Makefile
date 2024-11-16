


build:
	cargo build --release && cp target/release/flowrs .


logo:
	@ascii-image-converter image/README/1683789045509.png -C  -W 101 -c

run:
	FLOWRS_LOG=debug cargo run