use std::{collections::HashMap, path::PathBuf, vec};

use log::{info, warn};

use crate::slide::{utils, video, video_option, voice};

/* 例）
[適用ファイル名（絶対パス名OK、suffix照合）]
# タイトル
本文(行区切り)
本文(行区切り)
本文(行区切り)
本文(行区切り)
[適用ファイル名（絶対パス名OK、suffix照合）]
# タイトル
本文(行区切り)
本文(行区切り)
本文(行区切り)
本文(行区切り)
*/
//
#[derive(Debug, Clone)]
pub struct Section {
    // 対象とするファイル
    pub filename: String,
    pub title: Option<String>,
    pub contents: Vec<Content>,

    // 以下は、音声化のための情報
    // voices keyはcontenst keyと対になり、数が一致する
    pub voices: HashMap<String, voice::Data>,

    // 以下は、動画化のための情報
    pub video: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Content {
    pub key: String,
    pub voice_id: Option<i32>,
    pub text: String,
}

impl Content {
    pub fn new(voice_id: Option<i32>, text: String) -> Content {
        let key = uuid::Uuid::new_v4().to_string();
        Content {
            key,
            voice_id,
            text,
        }
    }
}

impl Section {
    pub fn blanc() -> Section {
        Section {
            filename: "".to_string(),
            title: None,
            contents: vec![],
            voices: HashMap::new(),
            video: None,
        }
    }

    pub fn create_vec(resource_text: PathBuf) -> Result<Vec<Section>, String> {
        let mut f = std::fs::File::open(resource_text).map_err(|e| e.to_string())?;
        // まずは行区切りに分割
        let mut all_text = String::new();
        std::io::Read::read_to_string(&mut f, &mut all_text).map_err(|e| e.to_string())?;

        // Handle various BOMs (Byte Order Marks) that might be present in files from different platforms
        // UTF-8 BOM: \u{feff}, UTF-16 LE: 0xFF 0xFE, UTF-16 BE: 0xFE 0xFF, etc.
        all_text = all_text.trim_start_matches("\u{feff}").to_string();

        // Platform-independent line splitting (handles both \r\n and \n)
        let split_text = all_text.lines().collect::<Vec<&str>>();
        let mut texts = vec![];
        let mut inner_text = Section::blanc();

        for text in split_text {
            let target_text = text.trim();
            if target_text.is_empty() {
                // Keep empty lines in content if we're already in a section with content
                if !inner_text.filename.is_empty() && !inner_text.contents.is_empty() {
                    inner_text.contents.push(Content::new(None, "".to_string()));
                }
                continue;
            }

            if target_text.starts_with("[") && target_text.ends_with("]") {
                // 新規: タイトルがあり、かつ、コンテンツがある場合は、新しいセクションとする
                // セクションを追加し、新しいテキストを作成
                if !inner_text.filename.is_empty() {
                    // Even if content is empty, we still want to preserve the section
                    texts.push(inner_text);
                    inner_text = Section::blanc();
                }

                let filename = target_text[1..target_text.len() - 1].trim();
                let path_filename = PathBuf::from(filename).to_string_lossy().into_owned();

                inner_text.filename = path_filename;
            } else if target_text.starts_with("#") {
                // このセクションの以前のタイトルを上書きする可能性がある
                // タイトルが複数ある場合は、最後のタイトルが採用される
                let title = target_text.trim_start_matches("#").trim();
                inner_text.title = Some(title.to_string());
            } else {
                // ファイル名が既に定義されている場合にのみコンテンツを追加します
                if !inner_text.filename.is_empty() {
                    // コンテンツの音声ID指定識別子がある場合、音声ID、テキストに分割
                    // ```@number コンテンツテキスト``` の形式
                    let (voice_id, text) = if target_text.starts_with("@") {
                        let mut split_text = target_text.splitn(2, " ");
                        println!("split_text: {:?}", split_text);
                        let voice_id = split_text
                            .next()
                            .unwrap()
                            .trim_start_matches("@")
                            .parse::<i32>()
                            .ok();
                        let text = split_text.next().unwrap_or("").to_string();
                        (voice_id, text)
                    } else {
                        (None, target_text.to_string())
                    };

                    warn!("voice_id: {:?}, text: {:?}", voice_id, text);

                    inner_text.contents.push(Content::new(voice_id, text));
                }
            }
        }

        // Add the last section if it has a filename (even if content might be empty)
        if !inner_text.filename.is_empty() {
            texts.push(inner_text);
        }

        Ok(texts)
    }

    // テキストコンテンツの音声化
    // Supported:
    // - VoicevoxAPIを使って音声化
    pub async fn create_voices(&mut self) -> Result<(), String> {
        if self.contents.is_empty() {
            return Err("No content to  to voice".to_string());
        }

        let output_dir = utils::target_path_from_env("DEFAULT_OUTPUT_VOICE_FILE_DIR");
        for content in self.contents.iter() {
            let output_filepath = output_dir
                .join(format!("{}.wav", content.key))
                .to_string_lossy()
                .into_owned();
            let voice_data = match voice::Data::new(content.voice_id, &output_filepath)
                .create_voice(content.text.as_str())
                .await
            {
                Ok(voice_data) => voice_data,
                Err(e) => {
                    return Err(format!("Error: {}", e));
                }
            };
            self.voices.insert(content.key.clone(), voice_data);
        }

        Ok(())
    }

    // ffmpegを使って動画化
    // セッションの音声ごとに動画を生成
    // 画像とテキストと音声を組み合わせて動画を生成
    // 音声ごとに生成した動画を連結し、セッションの動画を生成する
    pub async fn create_video(&mut self) -> Result<(), String> {
        let mut parts = vec![];
        for content in self.contents.iter() {
            let voice_data = match self.voices.get(&content.key) {
                Some(voice_data) => voice_data,
                None => {
                    warn!("voice data not found: {:?}", content.key);
                    continue;
                }
            };

            // 動画生成のためのオプション
            let mut op = video_option::Op::default();
            op.set_word(content.text.as_str());

            // 動画生成のためのパラメータ
            let (video_args, output_filepath) = video::create_args(
                content.key.to_string(),
                self.filename.clone(),
                voice_data.clone(),
                Some(op),
            );

            // 動画生成コマンド
            match video::create_part(video_args).await {
                Ok(result) => {
                    // 動画生成成功したら、パスを保存
                    info!("video created: {:?}", result);
                    parts.push(output_filepath);
                }
                Err(e) => {
                    return Err(format!("part create Error: {}", e));
                }
            };
        }

        let (concat_file, output_file) = video::create_output_files(parts);

        match video::concat(concat_file, output_file).await {
            Ok(output_video_filepath) => {
                info!("video concated: {:?}", output_video_filepath);
                self.video = Some(output_video_filepath);
            }
            Err(e) => {
                return Err(format!("concat Error: {}", e));
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use log::error;

    use crate::slide;

    use super::*;

    // 対象ディレクトリ内を内包したファイルを削除
    fn remove_files(to: &str) {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        let target_dir = std::path::PathBuf::from(format!("{}/results/output/{}", project_dir, to));

        let paths = std::fs::read_dir(target_dir).unwrap();
        for path in paths {
            let path = path.unwrap().path();
            if path.is_file() {
                std::fs::remove_file(path).unwrap();
            }
        }
    }

    #[test]
    fn test_remove_file_all() {
        remove_files("video");
        remove_files("voice");
    }

    #[tokio::test]
    async fn test_convert_voices() {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        dotenv::from_filename(".env.sample").ok();

        let target_file =
            std::path::PathBuf::from(project_dir).join("results/output/voice/test_client.wav");
        println!("project_dir: {}", target_file.display());

        let mut section = Section::blanc();
        section.filename = target_file.to_string_lossy().into_owned();

        section
            .contents
            .push(Content::new(None, "テスト".to_string()));
        let result = section.create_voices().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_video() {
        remove_files("video");
        remove_files("voice");

        let project_dir = env!("CARGO_MANIFEST_DIR");
        let env_file = PathBuf::from(project_dir).join(".env.sample");
        dotenv::from_filename(env_file).ok();

        env_logger::init();

        let text_filename = PathBuf::from(std::env::var("DEFAULT_RESOURCE_FILE_PATH").unwrap());
        let mut sections = slide::section::Section::create_vec(text_filename).unwrap();
        info!("resource data for section: {:?}", sections);

        for section in sections.iter_mut() {
            match section.create_voices().await {
                Ok(_) => {
                    info!("to_voices: {:?}", section);
                }
                Err(e) => {
                    error!("failed to_voices error: {}", e);
                }
            };

            match section.create_video().await {
                Ok(_) => {
                    info!("inner create_video: {:?}", section);
                }
                Err(e) => {
                    error!("failed create_video error: {}", e);
                }
            };
        }

        for section in sections.iter() {
            info!("section video: {:?}", section.video);
        }

        let concated_videos = sections
            .iter()
            .filter_map(|section| section.video.clone())
            .collect::<Vec<String>>();

        println!("concated_videos: {:?}", concated_videos);

        let (concat_file, output_file) = video::create_output_files(concated_videos);

        match video::concat(concat_file, output_file).await {
            Ok(output_video_filepath) => {
                info!("last video concated: {:?}", output_video_filepath);
            }
            Err(e) => {
                error!("last concat Error: {}", e);
            }
        };
    }
}
