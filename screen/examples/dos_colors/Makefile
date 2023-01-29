WIN_TARGET=i386-pc-dos-msvc
DOS_TARGET=i386-pc-dos-hxrt

BIN=colors

.PHONY: debug release rund runr clean clippy

release debug: %: target/$(DOS_TARGET)/%/$(BIN).exe target/$(DOS_TARGET)/%/CODEPAGE target/$(DOS_TARGET)/%/HDPMI32.EXE target/$(DOS_TARGET)/%/DPMILD32.EXE

rund: debug
	dosbox target/$(DOS_TARGET)/debug/$(BIN).exe

runr: release
	dosbox target/$(DOS_TARGET)/release/$(BIN).exe

clean:
	$(RM) -r HXRT216
	$(RM) -r target

target/$(DOS_TARGET)/%/CODEPAGE: target/$(WIN_TARGET)/%/$(BIN).exe
	mkdir -p target/$(DOS_TARGET)/$*
	find target/$(WIN_TARGET)/$*/build -name '$(BIN)-*' -print0 | xargs -0 -I '{}' cp -rf '{}'/out/CODEPAGE target/$(DOS_TARGET)/$*
	touch target/$(DOS_TARGET)/$*/CODEPAGE

target/$(DOS_TARGET)/%/$(BIN).exe: target/$(WIN_TARGET)/%/$(BIN).exe HXRT216/BIN/PESTUB.EXE HXRT216/BIN/DPMIST32.BIN
	mkdir -p target/$(DOS_TARGET)/$*
	cp -f target/$(WIN_TARGET)/$*/$(BIN).exe target/$(DOS_TARGET)/$*/$(BIN).exe
	wine HXRT216/BIN/PESTUB.EXE -v -n -x -s target/$(DOS_TARGET)/$*/$(BIN).exe HXRT216/BIN/DPMIST32.BIN
	touch target/$(DOS_TARGET)/$*/$(BIN).exe

target/$(DOS_TARGET)/%/HDPMI32.EXE: HXRT216/BIN/HDPMI32.EXE
	mkdir -p target/$(DOS_TARGET)/$*
	cp -f HXRT216/BIN/HDPMI32.EXE target/$(DOS_TARGET)/$*/HDPMI32.EXE

target/$(DOS_TARGET)/%/DPMILD32.EXE: HXRT216/BIN/DPMILD32.EXE
	mkdir -p target/$(DOS_TARGET)/$*
	cp -f HXRT216/BIN/DPMILD32.EXE target/$(DOS_TARGET)/$*/DPMILD32.EXE

HXRT216/BIN/HDPMI32.EXE HXRT216/BIN/DPMILD32.EXE HXRT216/BIN/PESTUB.EXE HXRT216/BIN/DPMIST32.BIN: HXRT216.zip
	$(RM) -r HXRT216
	mkdir HXRT216
	unzip -d HXRT216 HXRT216.zip

HXRT216.zip:
	wget https://www.japheth.de/Download/HX/HXRT216.zip
	touch -t 200801011952 HXRT216.zip

target/$(WIN_TARGET)/debug/$(BIN).exe: Cargo.toml Cargo.lock src/main.rs build.rs
	RUSTFLAGS="--cfg pc_atomics" \
		cargo +nightly build --verbose -Z build-std=alloc,core,panic_abort --target $(WIN_TARGET).json

target/$(WIN_TARGET)/release/$(BIN).exe: Cargo.toml Cargo.lock src/main.rs build.rs
	RUSTFLAGS="--cfg pc_atomics" \
		cargo +nightly build --verbose -Z build-std=alloc,core,panic_abort -Z build-std-features=panic_immediate_abort --target $(WIN_TARGET).json --release

clippy:
	RUSTFLAGS="--cfg pc_atomics" \
		cargo +nightly clippy --verbose -Z build-std=alloc,core,panic_abort -Z build-std-features=panic_immediate_abort --target $(WIN_TARGET).json

Cargo.lock: Cargo.toml
	cargo update
