use std::path::PathBuf;

use log::{error, info};

use crate::slide::video;

mod slide;

#[tokio::main]
async fn main() {
    init();

    // リソースとなるファイルから動画のセクションを生成
    let text_filename = PathBuf::from(std::env::var("DEFAULT_RESOURCE_FILE_PATH").unwrap());
    let mut sections = slide::section::Section::create_vec(text_filename).unwrap();
    info!("resource data for section: {:?}", sections);

    // セクションのテキストを音声に変換
    // sections.voicesにはコンテンツkey対応の音声ファイルが格納される
    for section in sections.iter_mut() {
        // 画像及び動画一つに対して、複数の音声が出力される
        // 段落ごとに音声を生成
        match section.create_voices().await {
            Ok(_) => {
                info!("to_voices: {:?}", section);
            }
            Err(e) => {
                error!("failed to_voices error: {}", e);
                return;
            }
        };

        // 段落ごとに動画を生成
        // - テキスト・音声ファイル群を画像に焼き付け
        // - セッションVideoに動画ファイルパスを格納
        match section.create_video().await {
            Ok(_) => {
                info!("inner create_video: {:?}", section);
            }
            Err(e) => {
                error!("failed create_video error: {}", e);
                return;
            }
        };
    }

    // Videoのパスを出力
    let concated_videos = sections
        .iter()
        .filter_map(|section| section.video.clone())
        .collect::<Vec<String>>();

    // 動画連結のためのファイルを作成
    // 出力先ファイルを作成
    let (concat_file, output_file) = video::create_output_files(concated_videos);

    // 動画を連結
    match video::concat(concat_file, output_file.clone()).await {
        Ok(output_video_filepath) => {
            info!("last video concated: {:?}", output_video_filepath);
        }
        Err(e) => {
            error!("last concat Error: {}", e);
            return;
        }
    };

    info!("success: {:?}", output_file);
}

fn init() {
    dotenv::from_filename(".env.sample").ok();
    env_logger::init();
}
