# Maintainer: Scqxd <scqxd@aur.archlinux.org>
# Generator: ArchForge v0.2.3

pkgname=archforge
pkgver=0.2.3
pkgrel=1
pkgdesc="AI-powered TUI for PKGBUILD generation and AUR management"
arch=('x86_64')
url="https://github.com/Scqxd/archforge"
license=('MIT')
depends=('glibc' 'gcc-libs')
makedepends=('cargo' 'rust')
optdepends=(
    'paru: AUR helper integration'
    'yay: AUR helper integration'
)
provides=('aur-manager' 'pkgbuild-generator')
conflicts=('archforge-git')
source=("https://github.com/Scqxd/archforge/archive/refs/tags/v${pkgver}.tar.gz")
sha256sums=('6bdb4334e9be334e556e88bf4a646c22ff77b85c03a2872191d72edd608d1943')

prepare() {
    cd "$pkgname-$pkgver"
    cargo fetch --locked
}

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

check() {
    cd "$pkgname-$pkgver"
    cargo test --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 target/release/archforge "$pkgdir/usr/bin/archforge"

    # Install bash completion
    install -Dm644 archforge/src/cli.rs \
        "$pkgdir/usr/share/bash_completion/completions/archforge" 2>/dev/null || true

    # Install man page (if exists)
    if [ -f target/release/archforge.1 ]; then
        install -Dm644 target/release/archforge.1 \
            "$pkgdir/usr/share/man/man1/archforge.1"
    fi
}