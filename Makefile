build:
	cargo build --release

install:
	cargo install --path .
	mkdir -p ~/.local/share/kservices5/
	mkdir -p ~/.local/share/dbus-1/services/

	cp plasma-runner-KRunnerLutris.desktop ~/.local/share/kservices5/
	sed "s,BIN_PATH,$(shell which lutris-runner-dbus)," org.kde.KRunnerLutris.service > ~/.local/share/dbus-1/services/org.kde.KRunnerLutris.service
