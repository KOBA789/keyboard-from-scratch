# 環境構築

※これは bootloader をビルドするための環境構築手順を含んでいません

## プログラミング言語 Rust による組み込みハードウェア向け開発環境

### Rust ツールチェーン
バージョン1.33(stable)のツールチェーンが必要です。

未インストールの方は、下記に示す公式のインストール手順に従い、インストールしてください。`rustc`, `cargo`,`rustup` などのコマンドがインストールされます。

[Install - Rust programming language](https://www.rust-lang.org/tools/install)

上記ページにもある通り、シェルをカスタマイズしていると `$HOME/.cargo/bin/` への PATH が自動で通らないことが多いです。 **いい感じに** してください。

既にインストール済みの場合は改めてインストールし直す必要はありませんが、バージョンが古い場合はアップデートしておいてください。

### ARM Cortex-M3 ターゲット
ARM Cortex-M3 をターゲットとしたバイナリをビルドするため、`thumbv7m-none-eabi` ターゲットがツールチェーンに必要です。

未インストールの場合は次のコマンドを実行し、ターゲットを追加してください。

```
rustup target add thumbv7m-none-eabi
```

### cargo-binutils
プレーンバイナリをビルドするため、 `cargo-binutils` が必要です。

[GitHub - rust-embedded/cargo-binutils: Cargo subcommands to invoke the LLVM tools shipped with the Rust toolchain](https://github.com/rust-embedded/cargo-binutils)

未インストールの場合は上記ページの `Installation` の手順に従い、インストールしてください。

## DFU ダウンロードツール
ビルドしたファームウェアをターゲットボードにダウンロードするため、DFU のホスト側のツールが必要です。

特にこだわりがない場合は `dfu-util` をインストールするとよいでしょう。

[dfu-util Homepage](http://dfu-util.sourceforge.net/)

### macOS でのインストール手順

macOS では Homebrew を用いてインストールできます。

```
brew install dfu-util
```


### Ubuntu 18.04 でのインストール手順

Ubuntu 18.04 では apt を用いてインストールできます。

必要に応じて sudo などを利用してください。

```
apt install dfu-util
```

## テキストエディタ
**テキストエディタは任意のものを利用してください。**

なお、筆者(KOBA789)は [Visual Studio Code](https://code.visualstudio.com/) に [Rust (rls)](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust) をインストールして使っています。

Rust を書くためのエディタに迷っている場合は参考にしてください。

## KiCAD

このコースでは、KiCAD を用いて回路図の作成とプリント基板のパターン設計をします。

バージョン 5.1 系の KiCAD を想定しています。

未インストールの場合は、以下に示す公式の手順に従ってインストールをしてください。

[Download | KiCad EDA](http://kicad-pcb.org/download/)

