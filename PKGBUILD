# Maintainer: Scqxd <scqxd@aur.archlinux.org>
# Generator: ArchForge v0.2.5

pkgname=archforge
pkgver=0.2.5
pkgrel=1
pkgdesc="AI-powered TUI for PKGBUILD generation and AUR management"
arch=('x86_64' 'aarch64')
url="https://github.com/Scqxd/archforge"
license=('MIT')
depends=('glibc' 'gcc-libs')
makedepends=('cargo' 'rust' 'git')
optdepends=(
    'paru: AUR helper integration'
    'yay: AUR helper integration'
    'sudo: Privilege escalation for system-wide operations'
    'arch-install-scripts: Enhanced installation features'
    'git: VCS integration and repository management'
)
provides=('aur-manager' 'pkgbuild-generator')
conflicts=('archforge-git')
replaces=('archforge-bin')
backup=('etc/archforge/config.toml')
source=("https://github.com/Scqxd/archforge/archive/refs/tags/v${pkgver}.tar.gz")
sha256sums=('e7dfe769aaed1776eeb4bb471af3363c722c6c51314a95104340607b6a84e929')

export CARGO_TARGET_DIR=target

prepare() {
    cd "$pkgname-$pkgver"
    cargo fetch --locked
}

build() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo build --release --offline
}

check() {
    cd "$pkgname-$pkgver"
    cargo test --release --offline
}

package() {
    cd "$pkgname-$pkgver"
    
    # Install main binary
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    
    # Install completions with fallback
    for shell in bash fish zsh; do
        local comp_file="completions/$pkgname.$shell"
        [[ "$shell" == "bash" ]] && local comp_dir="$pkgdir/usr/share/bash-completion/completions"
        [[ "$shell" == "fish" ]] && local comp_dir="$pkgdir/usr/share/fish/completions"
        [[ "$shell" == "zsh" ]] && local comp_dir="$pkgdir/usr/share/zsh/site-functions"
        
        if [[ -f "$comp_file" ]]; then
            install -Dm644 "$comp_file" "$comp_dir/$pkgname"
        fi
    done
    
    # Install man pages
    for man_file in target/release/*.1; do
        [[ -f "$man_file" ]] || continue
        local man_section="${man_file##*.}"
        install -Dm644 "$man_file" "$pkgdir/usr/share/man/man${man_section}/$(basename "$man_file")"
    done
    
    # Install license
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE" 2>/dev/null || true
    
    # Install default config (optional)
    if [[ -f "config.example.toml" ]]; then
        install -Dm644 "config.example.toml" "$pkgdir/etc/$pkgname/config.toml"
    fi
}