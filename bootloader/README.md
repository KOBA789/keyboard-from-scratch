# bootloader

コードは別リポジトリに分離しています。

https://github.com/KOBA789/dapboot

STM32F103x 向けの DFU ブートローダです。
[dapboot](https://github.com/devanlai/dapboot) の fork です。

## ビルド

ビルド方法は上記リポジトリの README に準じます。
`TARGET` としては `BLUEPILL` を指定してください。

## fork 元からの変更点

オリジナルの dapboot では、DFU モードに入るためにファームウェア側での実装または BOOT ジャンパの切り替えが必要でした。
しかし、この仕様ではファームウェア開発時のトライアンドエラーが面倒なため、RESET から数秒間(デフォルトでは5秒)は DFU モードを維持するように改造してあります。
DFU でファームウェアをダウンロードする場合は RESET して数秒以内にホスト側からダウンロードを開始してください。
