EXE = Cherry

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

native:
ifndef EVALFILE
	python3 ./scripts/download_net.py
endif
	cargo rustc --release --features tune -p cherry -- --emit link=$(NAME)
