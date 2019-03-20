# blink

Lチカをするだけのコードです。

Blue pill に書き込んで実行することで、ボード上の LED が点滅します。

組み込み Rust の練習用です。

## ビルド

ビルドし、バイナリを `app.bin` として出力:
```
cargo objcopy --bin kb789-blink --release -- -O binary app.bin
```

出力したバイナリを DFU Util でダウンロード:
```
dfu-util -d 1209:db42 -D app.bin
```
このとき、コマンド実行前にリセットボタンを押し、それから数秒以内に上記コマンドを実行してください。

なお、`1209:db42` は[ブートローダ dapboot の VID/PID](https://github.com/koba789/dapboot#usb-vidpid) です。
