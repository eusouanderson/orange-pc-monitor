.PHONY: release build test run clean

# Atalho principal: compila Linux + Windows, versiona e publica no GitHub
# Uso:
#   make release            -> Publica versão atual do Cargo.toml
#   make release V=patch    -> Incrementa versão patch (ex: 0.1.0 -> 0.1.1)
#   make release V=0.2.0    -> Define versão específica
release:
	@chmod +x scripts/release.sh
	@./scripts/release.sh $(V)

# Compila para ambas as plataformas localmente
build:
	cargo build --release --locked
	cargo xwin build --release --locked --target x86_64-pc-windows-msvc

# Roda os testes do sistema
test:
	cargo test

# Executa o HUD com interface gráfica
run:
	./target/release/orange-pc-monitor --gui

# Limpa artefatos temporários
clean:
	cargo clean
	rm -rf dist/
