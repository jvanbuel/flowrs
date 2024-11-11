


build:
	cargo build --release && cp target/debug/flowrs .


logo:
	@ascii-image-converter image/README/1683789045509.png -C  -W 101 -c
