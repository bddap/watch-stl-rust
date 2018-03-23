
PHONY: watch

watch:
	ag -l | grep -v samples | entr -rc cargo run -- samples/bunny_99.stl
