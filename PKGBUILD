# Maintainer: Your Name <your.email@example.com>
pkgname=hypridle_audio_listener
pkgver=0.1.0
pkgrel=1
pkgdesc="A hypridle tool to turn off monitors when audio is not playing."
arch=('x86_64')
url="https://github.com/msandoval/hypridle_audio_listener"
license=('BSD-3-Clause') # Replace with your app's license
depends=()
makedepends=('rust' 'cargo' 'pipewire' 'clang' 'llvm')
source=("$pkgname-$pkgver.tar.gz::https://github.com/msandoval/hypridle_audio_listener/archive/refs/tags/$pkgver.tar.gz")
sha256sums=('fc0e1c78b68c84b9b204ae8d6d44ccaeb5d4683af4315aa5fb121d0821cf107d')

build() {
    export LIBCLANG_PATH="/usr/lib/libclang.so"
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 "target/release/hypridle_audio_listener" "$pkgdir/usr/bin/hypridle_audio_listener"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
