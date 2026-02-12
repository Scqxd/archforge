//! Template builder for generating PKGBUILD content
//!
//! Contains the logic for building PKGBUILD templates for each
/// TemplateKind.

/// Builder for generating PKGBUILD templates
#[derive(Debug, Clone)]
pub struct TemplateBuilder<'a> {
    pkgname: &'a str,
    pkgver: &'a str,
    description: &'a str,
}

impl<'a> TemplateBuilder<'a> {
    /// Create a new template builder
    pub fn new(pkgname: &'a str, pkgver: &'a str, description: &'a str) -> Self {
        Self {
            pkgname,
            pkgver,
            description,
        }
    }

    /// Generate a simple C template
    pub fn build_c_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://example.com"
license=('MIT')
depends=('glibc')
makedepends=('gcc' 'make')
source=("https://example.com/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    gcc -o {pkgname} main.c -Wall
}}

package() {{
    install -Dm755 "$pkgname" "$pkgdir/usr/bin/{pkgname}"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a C++ template
    pub fn build_cpp_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://example.com"
license=('MIT')
depends=('glibcxx')
makedepends=('gcc' 'make' 'cmake')
source=("https://example.com/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    g++ -o {pkgname} main.cpp -std=c++17 -Wall
}}

package() {{
    install -Dm755 "$pkgname" "$pkgdir/usr/bin/{pkgname}"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Go template
    pub fn build_go_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://github.com/example/{pkgname}"
license=('MIT')
depends=('glibc')
makedepends=('go')
source=("https://github.com/example/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    go build -o {pkgname} -ldflags="-s -w"
}}

package() {{
    install -Dm755 "$pkgname" "$pkgdir/usr/bin/{pkgname}"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Python template
    pub fn build_python_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://pypi.org/project/{pkgname}"
license=('MIT')
depends=('python')
makedepends=('python-pip' 'python-setuptools' 'python-wheel')
source=("https://files.pythonhosted.org/packages/$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    python setup.py build
}}

package() {{
    cd "$pkgname-$pkgver"
    python setup.py install --root="$pkgdir" --optimize=1 --skip-build
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Rust template
    pub fn build_rust_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://crates.io/crates/{pkgname}"
license=('MIT')
depends=('glibc')
makedepends=('cargo' 'rustc')
source=("https://github.com/example/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}}

package() {{
    cd "$pkgname-$pkgver/target/release"
    install -Dm755 {pkgname} "$pkgdir/usr/bin/{pkgname}"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Node.js template
    pub fn build_nodejs_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://npmjs.com/package/{pkgname}"
license=('MIT')
depends=('nodejs')
makedepends=('nodejs' 'npm')
source=("https://registry.npmjs.org/{pkgname}/-/{pkgname}-$pkgver.tgz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    npm install --ignore-scripts
}}

package() {{
    cd "$pkgname-$pkgver"
    npm install --production --prefix "$pkgdir/usr/lib/node_modules/{pkgname}"
    mkdir -p "$pkgdir/usr/bin"
    ln -s "/usr/lib/node_modules/{pkgname}/bin/{pkgname}" "$pkgdir/usr/bin/{pkgname}"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Haskell template
    pub fn build_haskell_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://hackage.haskell.org/package/{pkgname}"
license=('MIT')
depends=('ghc-libs')
makedepends=('ghc' 'cabal')
source=("https://hackage.haskell.org/package/$pkgver/$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    cabal configure --enable-optimization=2
    cabal build
}}

package() {{
    cd "$pkgname-$pkgver"
    cabal install --prefix="$pkgdir" --disable-documentation
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a CMake template
    pub fn build_cmake_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://example.com"
license=('MIT')
depends=('glibc')
makedepends=('cmake' 'make' 'gcc')
source=("https://example.com/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    cmake -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr
    cmake --build build
}}

package() {{
    cd "$pkgname-$pkgver"
    cmake --install build --prefix "$pkgdir"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Meson template
    pub fn build_meson_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://example.com"
license=('MIT')
depends=('glibc')
makedepends=('meson' 'ninja' 'gcc')
source=("https://example.com/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    meson setup build --prefix=/usr --buildtype=plain
    meson compile -C build
}}

package() {{
    cd "$pkgname-$pkgver"
    meson install -C build --destdir "$pkgdir"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Perl template
    pub fn build_perl_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://metacpan.org/pod/{pkgname}"
license=('Artistic' 'GPL')
depends=('perl')
makedepends=('perl' 'make')
source=("https://cpan.metacpan.org/authors/id/A/AB/ABUTHOR/$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    perl Makefile.PL INSTALLDIRS=vendor
    make
}}

package() {{
    cd "$pkgname-$pkgver"
    make install DESTDIR="$pkgdir"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Ruby template
    pub fn build_ruby_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://rubygems.org/gems/{pkgname}"
license=('MIT')
depends=('ruby')
makedepends=('ruby' 'ruby-bundler')
source=("https://rubygems.org/downloads/{pkgname}-$pkgver.gem")
sha256sums=('SKIP')

package() {{
    gem install --no-document --ignore-dependencies \
        --bindir "$pkgdir/usr/bin" \
        --install-dir "$pkgdir/usr/lib/ruby/gems/$ruby_engine/gems/{pkgname}-$pkgver" \
        "$pkgname-$pkgver.gem"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a .NET template
    pub fn build_dotnet_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://github.com/example/{pkgname}"
license=('MIT')
depends=('dotnet-runtime')
makedepends=('dotnet-sdk')
source=("https://github.com/example/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    dotnet restore
    dotnet publish -c Release -r linux-x64 --self-contained false -o build
}}

package() {{
    cd "$pkgname-$pkgver"
    mkdir -p "$pkgdir/usr/lib/{pkgname}"
    cp -r build/* "$pkgdir/usr/lib/{pkgname}/"
    mkdir -p "$pkgdir/usr/bin"
    ln -s "/usr/lib/{pkgname}/{pkgname}" "$pkgdir/usr/bin/{pkgname}"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Java template
    pub fn build_java_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://github.com/example/{pkgname}"
license=('Apache-2.0')
depends=('java-runtime>=11')
makedepends=('jdk-openjdk' 'maven')
source=("https://github.com/example/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    mvn package -DskipTests -q
}}

package() {{
    cd "$pkgname-$pkgver"
    mkdir -p "$pkgdir/usr/share/java/{pkgname}"
    cp target/{pkgname}-$pkgver.jar "$pkgdir/usr/share/java/{pkgname}/"
    mkdir -p "$pkgdir/usr/bin"
    cp target/{pkgname}-$pkgver.jar "$pkgdir/usr/bin/{pkgname}.jar"
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a Qt template
    pub fn build_qt_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://github.com/example/{pkgname}"
license=('LGPL3')
depends=('qt5-base')
makedepends=('qt5-base' 'make' 'gcc')
source=("https://github.com/example/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    qmake {pkgname}.pro
    make
}}

package() {{
    cd "$pkgname-$pkgver"
    make INSTALL_ROOT="$pkgdir" install
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }

    /// Generate a generic fallback template
    pub fn build_generic_template(&self) -> String {
        format!(
            r#"# Maintainer: ArchForge Generated
pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="{description}"
arch=('x86_64')
url="https://example.com"
license=('MIT')
depends=('glibc')
makedepends=('gcc' 'make')
source=("https://example.com/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {{
    cd "$pkgname-$pkgver"
    ./configure --prefix=/usr
    make
}}

package() {{
    cd "$pkgname-$pkgver"
    make DESTDIR="$pkgdir" install
}}
"#,
            pkgname = self.pkgname,
            pkgver = self.pkgver,
            description = self.description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_template() {
        let builder = TemplateBuilder::new("test-c", "1.0.0", "A C package");
        let result = builder.build_c_template();
        assert!(result.contains("pkgname=test-c"));
        assert!(result.contains("gcc -o test-c main.c"));
    }

    #[test]
    fn test_rust_template() {
        let builder = TemplateBuilder::new("test-rust", "1.0.0", "A Rust package");
        let result = builder.build_rust_template();
        assert!(result.contains("pkgname=test-rust"));
        assert!(result.contains("cargo build --release"));
    }

    #[test]
    fn test_go_template() {
        let builder = TemplateBuilder::new("test-go", "1.0.0", "A Go package");
        let result = builder.build_go_template();
        assert!(result.contains("pkgname=test-go"));
        assert!(result.contains("go build"));
    }

    #[test]
    fn test_python_template() {
        let builder = TemplateBuilder::new("test-python", "1.0.0", "A Python package");
        let result = builder.build_python_template();
        assert!(result.contains("pkgname=test-python"));
        assert!(result.contains("python setup.py install"));
    }
}