


build:
	cargo build --release && cp target/release/flowrs .
	cp flowrs /usr/local/bin/flowrs


logo:
	@ascii-image-converter image/README/1683789045509.png -C  -W 101 -c

rotating_logo:
	@tiff2png -force -destdir image/rotation/png/ image/rotation/tiff/*;
	for file in ./image/rotation/png/*; do \
		set -e; \
		file_name=$$(basename $$file .png); \
		echo "Processing $$file_name"; \
		ascii-image-converter $$file -C  -W 101 -c > image/rotation/ascii/$${file_name}.ascii; \
	done

run:
	FLOWRS_LOG=debug cargo run