#!/usr/bin/env bash
set -euo pipefail

# 🍊 Orange PC Monitor — Automação de Build & Release com Versionamento
# Suporta: ./scripts/release.sh [versao | patch | minor | major]
# Exemplo 1: ./scripts/release.sh (usa a versão atual do Cargo.toml)
# Exemplo 2: ./scripts/release.sh patch (0.1.0 -> 0.1.1)
# Exemplo 3: ./scripts/release.sh 0.2.0 (define diretamente)

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

CURRENT_VERSION=$(grep -m1 '^version' Cargo.toml | sed -E 's/version = "([^"]+)"/\1/')
TARGET_ARG="${1:-}"

if [ -z "$TARGET_ARG" ]; then
    TARGET_VERSION="$CURRENT_VERSION"
elif [ "$TARGET_ARG" = "patch" ] || [ "$TARGET_ARG" = "minor" ] || [ "$TARGET_ARG" = "major" ]; then
    IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
    case "$TARGET_ARG" in
        patch) patch=$((patch + 1)) ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        major) major=$((major + 1)); minor=0; patch=0 ;;
    esac
    TARGET_VERSION="$major.$minor.$patch"
else
    TARGET_VERSION="${TARGET_ARG#v}"
fi

TAG="v$TARGET_VERSION"

echo "=========================================================="
echo "🍊 ORANGE PC MONITOR — RELEASE $TAG"
echo "=========================================================="

# 1. Atualizar Cargo.toml se a versão foi alterada
if [ "$TARGET_VERSION" != "$CURRENT_VERSION" ]; then
    echo "📝 Atualizando versão em Cargo.toml ($CURRENT_VERSION -> $TARGET_VERSION)..."
    sed -i "0,/^version = .*/s//version = \"$TARGET_VERSION\"/" Cargo.toml
    cargo check --quiet
    git add Cargo.toml Cargo.lock
    git commit -m "chore(release): bump version to $TAG" || true
else
    echo "📌 Mantendo versão atual: $TARGET_VERSION"
fi

# 2. Executar testes
echo "🧪 Executando testes unitários..."
cargo test --quiet

# 3. Compilação Multiplataforma
echo "🐧 Compilando versão nativa Linux (release)..."
cargo build --release --locked

echo "🪟 Compilando versão nativa Windows via cargo-xwin (release)..."
cargo xwin build --release --locked --target x86_64-pc-windows-msvc

# 4. Empacotamento
echo "📦 Empacotando binários em dist/..."
rm -rf dist
mkdir -p dist

LINUX_BIN="target/release/orange-pc-monitor"
WIN_BIN="target/x86_64-pc-windows-msvc/release/orange-pc-monitor.exe"

if [ ! -f "$LINUX_BIN" ]; then
    echo "❌ Erro: $LINUX_BIN não foi gerado!"
    exit 1
fi

if [ ! -f "$WIN_BIN" ]; then
    echo "❌ Erro: $WIN_BIN não foi gerado!"
    exit 1
fi

LINUX_ARCHIVE="dist/orange-pc-monitor-${TAG}-linux-x86_64.tar.gz"
WIN_ARCHIVE="dist/orange-pc-monitor-${TAG}-windows-x86_64.zip"

tar -czf "$LINUX_ARCHIVE" -C target/release orange-pc-monitor -C "$PROJECT_DIR" README.md

python3 -c "
import zipfile
with zipfile.ZipFile('$WIN_ARCHIVE', 'w', zipfile.ZIP_DEFLATED) as z:
    z.write('$WIN_BIN', 'orange-pc-monitor.exe')
    z.write('README.md', 'README.md')
"

# 5. Gerar Hashes SHA-256
echo "🔒 Gerando hashes SHA-256 de integridade..."
(
    cd dist
    for f in *.tar.gz *.zip; do
        sha256sum "$f" > "$f.sha256"
    done
)

# 6. Criar e Enviar Tag Git
echo "🏷️ Verificando Tag Git $TAG..."
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag $TAG já existe localmente."
else
    git tag -a "$TAG" -m "Release $TAG"
fi

echo "🚀 Enviando alterações e tags para o GitHub..."
git push origin HEAD --tags

# 7. Criar Release no GitHub via gh CLI
echo "🌐 Publicando Release no GitHub..."
if gh release view "$TAG" >/dev/null 2>&1; then
    echo "Atualizando ativos da release existente $TAG..."
    gh release upload "$TAG" dist/* --clobber
else
    echo "Criando nova release $TAG no GitHub..."
    gh release create "$TAG" dist/* --title "Orange PC Monitor $TAG" --generate-notes
fi

echo ""
echo "=========================================================="
echo "🎉 Release $TAG publicada com sucesso!"
echo "🔗 URL: $(gh release view "$TAG" --json url -q .url)"
echo "=========================================================="
