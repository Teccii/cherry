EXE = Cherry
EVALFILE = networks/default.bin

ifeq ($(OS),Windows_NT)
NAME := $(EXE).exe
else
NAME := $(EXE)
endif

native:
	cargo rustc --release -p cherry -- --emit link=$(NAME)