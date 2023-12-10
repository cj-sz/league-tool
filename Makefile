# TODO add examples and help message
all:
	cargo build
clean:
	cargo clean
ex: all
	@echo 'Without some previous seeding statistics:'
	target/debug/league-tool examples/teams.txt examples/games.txt
	@echo
	@echo 'And some more descriptive statistics if we had given it a previous seeding beforehand:'
	target/debug/league-tool examples/teams.txt examples/games.txt examples/seeding.txt
help:
	@echo 'Usage:'
	@echo
	@echo '  > make                          # builds the tool'
	@echo '  > make clean                    # removes build files'
	@echo '  > make ex             			 # runs the examples'
	@echo
	@echo 'Once built, do:'
	@echo 'target/debug/league-tool <teamfile> <gamefile> <optional: seeding file>'
	@echo
	@echo 'See https://github.com/cj-sz/league-tool for more detailed instructions.'