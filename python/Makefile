
PYTHON ?= ../venv/bin/python
MATURIN ?= ../venv/bin/maturin
VIRTUAL_ENV ?= ../venv

.PHONY: install-dev develop build test

install-dev:
	$(PYTHON) -m pip install -U pip
	$(PYTHON) -m pip install -r requirements.txt

develop:
	VIRTUAL_ENV=$(VIRTUAL_ENV) $(MATURIN) develop

build:
	$(MATURIN) build --release

test:
	cd .. && cargo test
	cd .. && ./venv/bin/python -m pytest -q
