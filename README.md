# Slide with voice(vox)

## 1. プロジェクト概要

### 1.1 プロジェクトの目的

Slide with voice(vox) は、テキストファイルから動画を生成するツールです。特にプレゼンテーションやサービス紹介など、プログラマブルなテンプレートライクな動画を素早く生成することに特化しています。

### 1.2 特徴

*   **テキストから動画生成:** テキストファイルとスライド画像から、ナレーション付きの動画を自動生成します。
*   **プログラマブルなテンプレート:** スライドとテキストを組み合わせて、一定のフォーマットで動画を生成できます。
*   **Voicevox連携:** 音声合成にVoicevoxを使用し、多様なキャラクターによるナレーションを作成できます。
*   **AivisSpeech連携** 音声合成にAivisSpeechを使用いただけます。

### 1.3 デモ

[紹介動画](https://youtu.be/MiVK6Sxf-vQ)

[![Slide with voice(vox)](https://github.com/howlrs/slide_with_voice/blob/release/images/icon.png?raw=true)](https://www.youtube.com/watch?v=MiVK6Sxf-vQ)

## 2. 使い方

### 2.1 依存関係

*   **Voicevox:** 起動している必要があります。
*   **ffmpeg:** システムにインストールされている必要があります。

### 2.2 準備

1.  スライドに使用する画像および動画ファイル群を作業ディレクトリに用意します。
2.  テキストファイル（`resource.txt`）を編集し、スライドに対応するテキスト、タイトルなどを記述します。

### 2.3 記述ルール

*   ファイル名に対して、単一タイトル・複数テキストが紐づきます。
*   ファイル名・テキスト郡は必須です。
*   タイトルは任意です。
*   行単位でのボイス出力に対応しています。

`resource.txt`の例:

```txt
[C:\path\to\slide1.png]
# スライド1のタイトル
@2 これはスライド1の説明です。
@3 これはスライド1の別の説明です。
[C:\path\to\slide2.png]
# スライド2のタイトル
@2 これはスライド2の説明です。
```

*   `[スライドのファイルパス]` : スライドとして使用する画像または動画ファイルのパスを記述します。
*   `# タイトル` : スライドのタイトルを記述します（任意）。
*   `@番号 テキスト` : スライドに表示するテキストを記述します。`@`に続く数字はVoicevoxのボイスIDを指定します。

### 2.4 実行

1. ターミナルでプロジェクトのディレクトリに移動し、以下のコマンドを実行します。
   1.  または、githubリリースから環境に応じたバイナリをDLしご利用ください。

```bash
cargo run
```

## 3. ファイル構成

```
├── Cargo.toml
├── README.md
├── resource
│   ├── fonts
│   │   ├── OFL.txt
│   │   ├── README.txt
│   │   └── NotoSansJP-Bold.ttf  # デフォルトフォント
│   ├── resource.txt             # スライド、テキスト定義ファイル
│   └── voice_ids.json           # VoicevoxのボイスID情報
├── src
│   ├── main.rs                  # エントリーポイント
│   ├── slide
│   │   ├── file.rs              # (未使用)
│   │   ├── mod.rs
│   │   ├── section.rs           # スライドセクションの定義、処理
│   │   ├── utils.rs             # ユーティリティ関数
│   │   ├── video.rs             # 動画生成処理
│   │   ├── video_option.rs      # 動画オプション
│   │   └── voice.rs             # 音声生成処理
└── .env.sample
```

### 3.1 主要ファイルの説明

*   **`Cargo.toml`**: Rustプロジェクトの設定ファイル。
*   **`README.md`**: プロジェクトの概要や使い方を説明するファイル（このファイル）。
*   **`resource/`**: リソースファイルが格納されるディレクトリ。
    *   **`resource.txt`**: スライド、テキスト、ボイスIDの対応を記述するファイル。
    *   **`voice_ids.json`**: Voicevoxで使用できるボイスIDの一覧。
    *   **`fonts/NotoSansJP-Bold.ttf`**:  デフォルトで使用されるフォントファイル。
*   **`src/main.rs`**: プログラムのエントリーポイント。
*   **`src/slide/`**: スライド生成に関する処理を記述したモジュール。
    *   **`section.rs`**: スライドのセクション（画像、テキスト、音声）を定義し、処理する。
    *   **`voice.rs`**: Voicevox APIを呼び出して音声ファイルを生成する。
    *   **`video.rs`**: ffmpegを呼び出して動画ファイルを生成する。
    *   **`video_option.rs`**: 動画生成オプションを定義する。
    *   **`utils.rs`**: 汎用的なユーティリティ関数を提供する。
*   **`.env.sample`**: 環境変数のサンプルファイル。

## 4. 環境変数

`.env`ファイルに以下の環境変数を設定します。

*   **`DEFAULT_RESOURCE_FILE_PATH`**: `resource.txt`ファイルのパス。
*   **`DEFAULT_OUTPUT_VOICE_FILE_DIR`**: 音声ファイルの出力先ディレクトリ。
*   **`DEFAULT_OUTPUT_VIDEO_FILE_DIR`**: 動画ファイルの出力先ディレクトリ。
*   **`DEFAULT_VOICEVOX_SERVER_URL`**: VoicevoxのサーバーURL。
*   **`DEFAULT_VOICEVOX_VOICE_ID`**: デフォルトのVoicevoxボイスID。

## 5. 仕組み

1.  `src/main.rs` がエントリーポイントとなり、`resource.txt` ファイルを読み込みます。
2.  `src/slide/section.rs` で、スライド、テキスト、ボイスIDの対応関係を解析し、セクションを作成します。
3.  `src/slide/voice.rs` で、Voicevox APIを呼び出してテキストから音声ファイルを生成します。
4.  `src/slide/video.rs` で、ffmpegを呼び出してスライド画像、テキスト、音声ファイルを組み合わせて動画ファイルを生成します。
5.  最後に、生成された動画ファイルを連結して最終的な動画ファイルを生成します。

## 6. 開発

### 6.1 依存クレート

*   `chrono`: 日時処理
*   `dotenv`: 環境変数
*   `env_logger`: ログ
*   `log`: ログ
*   `reqwest`: HTTPクライアント
*   `serde`: シリアライズ、デシリアライズ
*   `serde_json`: JSON
*   `tokio`: 非同期処理
*   `uuid`: UUID生成
*   `voicevox-client`: Voicevox APIクライアント

### 6.2 今後の開発

*   **口パクキャラクタ:** ゆっくりボイスのような口パクキャラクターの追加。
*   **スライドのデザインカスタマイズ:** サイズ、位置、フォントなどを設定ファイルで変更できるようにする。

## 7. ライセンス

このプロジェクトはMITライセンスで提供されています。

## 8. 貢献

バグ報告、機能提案、プルリクエストなど、どのような貢献も歓迎します。

1.  Issueを作成する。
2.  ブランチを作成する。
3.  コードを実装する。
4.  テストを実行する。
5.  プルリクエストを作成する。

## 9. 連絡先
[@xhowlrs](https://x.com/xhowlrs)