use std::io::Write;

use log::info;

use crate::slide::{utils, video_option::Op, voice};

pub fn create_args(
    key: String,
    input_filepath: String,
    input_voice: voice::Data,
    op: Option<Op>,
) -> (Vec<String>, String) {
    let op = op.unwrap_or_default();
    let width = op.w.unwrap_or_default();
    let height = op.h.unwrap_or_default();

    // フィルターを生成
    let filter = op.create_filter_complex();

    let duration = input_voice.duration.num_milliseconds() as f64 / 1000.0;

    let output_filename = {
        let output_dir = utils::target_path_from_env("DEFAULT_OUTPUT_VIDEO_FILE_DIR");

        let mut row = output_dir.join(format!("{}.mp4", key));
        if row.is_relative() {
            // ./
            row = std::env::current_dir().unwrap().join(row);
        }

        row.to_string_lossy().into_owned()
    };

    (
        vec![
            // 画像を繰り返しフレームに表示する
            "-loop".to_string(), // 画像をループ再生するオプション
            "1".to_string(),     // ループ回数(1で無限ループ)
            // インプット画像または動画ファイル
            "-i".to_string(), // 画像または動画ファイルを入力として指定
            input_filepath,   // 入力ファイルパス
            // インプット音声ファイル
            "-i".to_string(),     // 音声ファイルを入力として指定
            input_voice.filepath, // 音声ファイルパス
            // キャラクターなどを出力する場合あここに[-i, input_filepath]を追加
            // フィルターを追加
            "-filter_complex".to_string(),   // 複雑なフィルタ構成を指定
            filter,                          // フィルタ内容
            "-map".to_string(),              // 映像ストリームのマッピングを指定
            "[out2]".to_string(),            // 映像出力ラベル
            "-map".to_string(),              // 音声ストリームのマッピングを指定
            "1:a".to_string(),               // 二番目の入力ファイルの音声を使用
            "-s".to_string(),                // 出力動画の解像度を指定
            format!("{}x{}", width, height), // 横×縦のフォーマット
            "-t".to_string(),                // 出力の長さを指定
            format!("{}", duration),         // 音声の長さに合わせた秒数
            "-c:v".to_string(),              // ビデオコーデックの指定
            "hevc_nvenc".to_string(),        // NVIDIAのHEVCハードウェアエンコード
            "-c:a".to_string(),              // オーディオコーデックを指定
            "aac".to_string(),               // AACを用いた音声エンコード
            "-pix_fmt".to_string(),          // ピクセルフォーマットの指定
            "yuv420p".to_string(),           // yuv420p形式
            // "-shortest".to_string(),         // 入力の中で最も短いストリームに合わせて終了
            // "-y".to_string(),                // 出力ファイルを上書き
            output_filename.clone(), // 出力ファイル名
        ],
        output_filename,
    )
}

// create video
// ffmpegで動画を生成
// 指定引数が生成指定パラメータ
// 一つの画像または動画に対して、複数の音声が焼き込まれる
// 字幕やキャラクターの表示も可能
// 生成した動画ファイルを返す
pub async fn create_part(args: Vec<String>) -> Result<(), String> {
    // ffmpegで動画を連結する
    let output = std::process::Command::new("ffmpeg")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    // error handling
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("failed create_part: {:?}", output))
    }
}

pub fn create_output_files(video_files: Vec<String>) -> (String, String) {
    let output_dir = utils::target_path_from_env("DEFAULT_OUTPUT_VIDEO_FILE_DIR");
    let filename = uuid::Uuid::new_v4().to_string();

    let concat_file = output_dir.join(format!("concat-{}.txt", filename));
    let output_file = output_dir.join(format!("concat-{}.mp4", filename));

    let mut file = std::fs::File::create(concat_file.clone()).unwrap();
    for video_file in video_files {
        writeln!(file, "file '{}'", video_file).unwrap();
    }

    file.flush().unwrap();

    // 文字列をファイルパスに変換
    (
        concat_file.to_string_lossy().into_owned(),
        output_file.to_string_lossy().into_owned(),
    )
}

pub async fn concat(concat_file: String, output_file: String) -> Result<String, String> {
    // ffmpegのパラメータ引数
    let result = {
        let args = [
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_file.as_str(),
            "-c",
            "copy",
            output_file.as_str(),
        ];
        // ffmpegで動画を連結する
        std::process::Command::new("ffmpeg")
            .args(args)
            .output()
            .map_err(|e| e.to_string())?
    };

    info!("result concat video: {:?}", result);

    // error handling
    if result.status.success() {
        Ok(output_file)
    } else {
        Err(format!("failed concat: {:?}", result))
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use log::error;

    use super::*;

    #[test]
    fn test_create_args() {
        let key = "test".to_string();
        let input_filepath = "test.mp4".to_string();
        let input_voice = voice::Data {
            voice_id: Some(14),
            filepath: "test.wav".to_string(),
            duration: TimeDelta::seconds(10),
        };

        let op = Op {
            w: Some(1920),
            h: Some(1080),
            background_color: Some("black".to_string()),
            // char_prompt: Some("".to_string()),
            font: Some("NotoSansJP-Bold.otf".to_string()),
            font_size: Some("48".to_string()),
            font_color: Some("white".to_string()),
            border_color: Some("0xBBDEFB".to_string()),
            word: Some("".to_string()),
        };

        let (args, output_filename) = create_args(key, input_filepath, input_voice, Some(op));

        assert_eq!(args.len(), 24);
        assert_eq!(output_filename, r"output\test.mp4");
    }

    #[tokio::test]
    async fn test_concat() {
        let current_dir = std::env::current_dir().unwrap();
        let concat_file = current_dir
            .join(r"results\output\video\dadf520a-0f2a-4009-b57a-0177e44b594c.txt")
            .to_string_lossy()
            .into_owned();

        // is exists concat file
        assert!(std::path::Path::new(&concat_file).exists());

        let output_file = current_dir
            .join(r"results\output\video\output.mp4")
            .to_string_lossy()
            .into_owned();

        assert!(!std::path::Path::new(&output_file).exists());

        let result = match concat(concat_file, output_file.clone()).await {
            Ok(result) => result,
            Err(e) => {
                error!("failed to concat error: {}", e);
                e
            }
        };

        assert_eq!(result, output_file);
    }
}
