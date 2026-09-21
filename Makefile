.PHONY: mdbook_assets serve build check

mdbook_assets:
	mdbook-mermaid install .

serve: mdbook_assets
	mdbook serve --port 3001

build: mdbook_assets
	mdbook build

check:
	cargo rpg check
