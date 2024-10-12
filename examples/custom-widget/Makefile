DOS_JSON_TARGET=i386-pc-dos-msvc
DOS_TARGET=i386-pc-dos-hxrt
TARGET ?= $(shell rustc -Vv | awk -F ': ' '{ if($$1 == "host") print $$2 }')
EXE_SUFFIX := $(shell \
	rustc +nightly -Z unstable-options --print target-spec-json --target $(TARGET) | jq -r '."exe-suffix" // empty' \
)

BIN=cstmwdgt

SRC=\
	Cargo.toml Cargo.lock src/main.rs src/ui.xaml

.PHONY: debug release rund runr clippy dosdebug dosrelease dosrund dosrunr clean dosclippy

release debug: %: target/$(TARGET)/%/$(BIN)$(EXE_SUFFIX)

dosrelease: target/$(DOS_TARGET)/release/$(BIN).exe \
	target/$(DOS_TARGET)/release/CODEPAGE target/$(DOS_TARGET)/release/HDPMI32.EXE target/$(DOS_TARGET)/release/DPMILD32.EXE

dosdebug: target/$(DOS_TARGET)/debug/$(BIN).exe \
	target/$(DOS_TARGET)/debug/CODEPAGE target/$(DOS_TARGET)/debug/HDPMI32.EXE target/$(DOS_TARGET)/debug/DPMILD32.EXE

rund: debug
	target/$(TARGET)/debug/$(BIN)$(EXE_SUFFIX)

runr: release
	target/$(TARGET)/release/$(BIN)$(EXE_SUFFIX)

dosrund: dosdebug
	dosbox target/$(DOS_TARGET)/debug/$(BIN).exe

dosrunr: dosrelease
	dosbox target/$(DOS_TARGET)/release/$(BIN).exe

clean:
	$(RM) -r HXRT216
	$(RM) -r target

target/$(DOS_TARGET)/%/CODEPAGE: target/$(DOS_JSON_TARGET)/%/$(BIN).exe
	mkdir -p target/$(DOS_TARGET)/$*
	find target/$(DOS_JSON_TARGET)/$*/build -name '$(BIN)-*' -print0 | xargs -0 -I '{}' cp -rf '{}'/out/CODEPAGE target/$(DOS_TARGET)/$*
	touch target/$(DOS_TARGET)/$*/CODEPAGE

target/$(DOS_TARGET)/%/$(BIN).exe: target/$(DOS_JSON_TARGET)/%/$(BIN).exe HXRT216/BIN/PESTUB.EXE HXRT216/BIN/DPMIST32.BIN
	mkdir -p target/$(DOS_TARGET)/$*
	cp -f target/$(DOS_JSON_TARGET)/$*/$(BIN).exe target/$(DOS_TARGET)/$*/$(BIN).exe
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
	wget -4 https://www.japheth.de/Download/HX/HXRT216.zip
	touch -t 200801011952 HXRT216.zip

target/$(TARGET)/debug/$(BIN)$(EXE_SUFFIX): $(SRC)
	cargo +nightly build \
		--verbose -Z build-std=alloc,core,panic_abort --target $(TARGET)

target/$(TARGET)/release/$(BIN)$(EXE_SUFFIX): $(SRC)
	cargo +nightly build \
		--verbose -Z build-std=alloc,core,panic_abort -Z build-std-features=panic_immediate_abort \
		--target $(TARGET) --release

target/$(DOS_JSON_TARGET)/debug/$(BIN).exe: $(SRC)
	cargo +nightly build \
		--verbose -Z build-std=alloc,core,panic_abort --target $(DOS_JSON_TARGET).json

target/$(DOS_JSON_TARGET)/release/$(BIN).exe: $(SRC)
	cargo +nightly build \
		--verbose -Z build-std=alloc,core,panic_abort -Z build-std-features=panic_immediate_abort \
		--target $(DOS_JSON_TARGET).json --release

clippy:
	cargo +nightly clippy \
		--verbose -Z build-std=alloc,core,panic_abort -Z build-std-features=panic_immediate_abort \
		--target $(TARGET)

dosclippy:
	cargo +nightly clippy \
		--verbose -Z build-std=alloc,core,panic_abort -Z build-std-features=panic_immediate_abort \
		--target $(DOS_JSON_TARGET).json

Cargo.lock: Cargo.toml
	cargo update
	touch Cargo.lock
