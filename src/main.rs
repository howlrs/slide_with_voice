use std::{
    io::Write,
    path::{Path, PathBuf},
};

use log::{error, info};

use crate::slide::video;

mod slide;

#[tokio::main]
async fn main() {
    init();
    // 必要なファイル群の存在を確認
    // なければ作成して終了
    if !exists_resource_dir() {
        create_resource_dir();
        return;
    }

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

fn exists_resource_dir() -> bool {
    let resource_text = PathBuf::from(std::env::var("DEFAULT_RESOURCE_FILE_PATH").unwrap());
    let resource_dir = Path::new(&resource_text).parent().unwrap();
    if resource_dir.exists() {
        return true;
    }

    false
}

fn create_resource_dir() {
    let resource_text = PathBuf::from(std::env::var("DEFAULT_RESOURCE_FILE_PATH").unwrap());
    let resource_dir = Path::new(&resource_text).parent().unwrap();
    std::fs::create_dir_all(resource_dir).unwrap();

    // mkdir resource/, resource/slides/, resource/fonts/
    let resource_dirs = vec![
        resource_dir.join("slides"),
        resource_dir.join("fonts"),
        resource_dir.join("videos"),
    ];

    for dir in resource_dirs {
        std::fs::create_dir_all(dir).unwrap();
    }

    // make resource.txt
    let mut file = std::fs::File::create(&resource_text).unwrap();
    let text = "当ファイルを更新して実行してください。";
    file.write_all(text.as_bytes()).unwrap();
    file.flush().unwrap();
}
