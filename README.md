# Slide with voice(vox)
テキストファイルから動画を生成します。
プログラマブルなテンプレートライクな動画をすばやく生成できます。特にプレゼンテーションやサービス紹介などに有用です。

## 記述ルール
- ファイル名に対して、単一タイトル・複数テキストが紐づく
- ファイル名・テキスト郡は必須
- タイトルは任意
- 行単位のボイズ出力
  
※ ファイル一つに対して、複数行のテキスト及び音声を動画生成される

![紹介動画](https://youtu.be/MiVK6Sxf-vQ)

<iframe width="560" height="315" src="https://www.youtube.com/embed/MiVK6Sxf-vQ?si=" title="Rust x Voicevoxプログラマブル動画生成" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>


## 依存関係
- Voicevox（起動
- ffmpeg
 
## Supported
- AivisSpeech with Voicevox

## Soon
- タイトル


## Features
- スライドを読み込む
  - 発展：一部動画も対応する
- スライドに対応するテキストを読み込む
- テキストから音声生成する
- 音声の再生時間を取得する
- スライドに対して音声とテキストを焼き付ける
- スライド単位の動画を連結する
- ファイナライズする

## Not implemented
- 自動改行
  - 一行毎音声と字幕を出力しています。適宜分割してください。
  - 分割により句読点のような大きな待機時間が増えることはありません。
- ゆっくりによく見られる口パクキャラクタの追加


## 想定使用手順
1. スライドに使用する画像及び動画ファイル群を作業ディレクトリに置く
2. 作業ディレクトリ内にあるリソースファイルを更新する
3. Option: 設定ファイルを見直す


## あるといい機能
- 作業ディレクトリ群を生成
  - 画像ディレクトリから画像を読み込みソートし、テキストファイルを生成