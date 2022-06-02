WIN_TARGET=i586-pc-windows-msvc
DOS_TARGET=i586-pc-dmpi-hxrt

bin=tstdos32

.PHONY: debug release cargo.debug cargo.release

release debug: %: target/$(DOS_TARGET)/%/$(bin).exe target/$(DOS_TARGET)/%/CODEPAGE target/$(DOS_TARGET)/%/HDPMI32.EXE target/$(DOS_TARGET)/%/DPMILD32.EXE

target/$(DOS_TARGET)/%/CODEPAGE: cargo.%
	mkdir -p target/$(DOS_TARGET)/$*
	find target/$(WIN_TARGET)/$*/build -name '$(bin)-*' -print0 | \
	    xargs -0 -I '{}' \
	    cp -rf '{}'/out/CODEPAGE target/$(DOS_TARGET)/$*

target/$(DOS_TARGET)/%/$(bin).exe: cargo.% \
    HXRT216/BIN/PESTUB.EXE \
    HXRT216/BIN/DPMIST32.BIN
	mkdir -p target/$(DOS_TARGET)/$*
	cp -f target/$(WIN_TARGET)/$*/$(bin).exe \
	    target/$(DOS_TARGET)/$*/$(bin).exe
	wine HXRT216/BIN/PESTUB.EXE -v -n -x -s \
	    target/$(DOS_TARGET)/$*/$(bin).exe \
	    HXRT216/BIN/DPMIST32.BIN

target/$(DOS_TARGET)/%/HDPMI32.EXE: HXRT216/BIN/HDPMI32.EXE
	mkdir -p target/$(DOS_TARGET)/$*
	cp -f HXRT216/BIN/HDPMI32.EXE \
	    target/$(DOS_TARGET)/$*/HDPMI32.EXE

target/$(DOS_TARGET)/%/DPMILD32.EXE: HXRT216/BIN/DPMILD32.EXE
	mkdir -p target/$(DOS_TARGET)/$*
	cp -f HXRT216/BIN/DPMILD32.EXE \
	    target/$(DOS_TARGET)/$*/DPMILD32.EXE

HXRT216/BIN/HDPMI32.EXE \
HXRT216/BIN/DPMILD32.EXE \
HXRT216/BIN/PESTUB.EXE \
HXRT216/BIN/DPMIST32.BIN: \
    HXRT216.zip
	rm -rf HXRT216
	mkdir HXRT216
	unzip -d HXRT216 HXRT216.zip
	find HXRT216 -print0 | \
	    xargs -0 -I '{}' \
	    touch '{}'

HXRT216.zip:
	wget https://www.japheth.de/Download/HX/HXRT216.zip

cargo.debug: Cargo.toml Cargo.lock src/main.rs build.rs
	cargo +nightly build \
	    --verbose \
	    -Z build-std=core,panic_abort \
	    --target $(WIN_TARGET)

cargo.release: Cargo.toml Cargo.lock src/main.rs build.rs
	cargo +nightly build \
	    --verbose \
	    -Z build-std=core,panic_abort \
	    -Z build-std-features=panic_immediate_abort \
	    --target $(WIN_TARGET) \
	    --release
