use std::{fs::File, io::Write};

use chrono::TimeDelta;
use voicevox_client::Client;

#[derive(Debug, Clone)]
pub struct Data {
    pub voice_id: Option<i32>,
    pub filepath: String,
    pub duration: TimeDelta,
}

impl Data {
    pub fn new(voice_id: Option<i32>, outputpath: &str) -> Data {
        Data {
            voice_id,
            filepath: outputpath.to_string(),
            duration: TimeDelta::zero(),
        }
    }

    pub async fn create_voice(&mut self, text: &str) -> Result<Self, String> {
        // ここで音声化処理を行う
        // 例）音声化処理を行い、durationをセットする
        let duration_delta = match self.voicebox(text).await {
            Ok(duration) => duration,
            Err(e) => {
                return Err(format!("Error: {}", e));
            }
        };
        println!("duration: {:?}", duration_delta);
        self.duration = duration_delta;
        Ok(self.clone())
    }

    // 出力ファイルに対して音声を生成する
    // voicevox_clientを使用して音声を生成する
    async fn voicebox(&self, text: &str) -> Result<TimeDelta, String> {
        let base_path = std::env::var("DEFAULT_VOICEVOX_SERVER_URL").unwrap();
        let client = Client::new(base_path);

        let default_voice_id = std::env::var("DEFAULT_VOICEVOX_VOICE_ID")
            .unwrap()
            .parse::<i32>()
            .unwrap();
        let voice_id = match self.voice_id {
            Some(voice_id) => voice_id,
            None => default_voice_id,
        };

        // クエリ生成
        let audio_query = match client.create_audio_query(text, voice_id, None).await {
            Ok(audio_query) => audio_query,
            Err(e) => {
                return Err(format!("create query: {}", e));
            }
        };

        // 音声生成
        let audio = match audio_query.synthesis(voice_id).await {
            Ok(audio) => audio,
            Err(e) => {
                return Err(format!("create audio: {}", e));
            }
        };

        // self.convert_pcm_to_wav(&audio, &outputpath, 24000, 1)?; // サンプルレートとチャンネル数はVoicevoxのデフォルトに合わせる

        // save file
        let mut file = File::create(&self.filepath).map_err(|e| e.to_string())?;
        file.write_all(&audio).map_err(|e| e.to_string())?;

        // get audio play time
        let duration = audio.len() as f32 / 48000.0;
        let time_delta = chrono::Duration::milliseconds((duration * 1000.0) as i64);

        Ok(time_delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_voice_ids() {
        let current = std::env::current_dir().unwrap();
        let env_path = current.join(".env.sample");
        dotenv::from_filename(env_path).ok();

        let base_path = std::env::var("DEFAULT_VOICEVOX_SERVER_URL").unwrap();
        let client = reqwest::Client::new();

        let result = match client.get(format!("{}/speakers", base_path)).send().await {
            Ok(result) => match result.json::<serde_json::Value>().await {
                Ok(result) => result,
                Err(e) => {
                    println!("Error: {}", e);
                    return;
                }
            },
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        };

        // jsonをファイルに保存
        let output_path = current.join("resource/voice_ids.json");
        let mut file = std::fs::File::create(output_path).unwrap();
        file.write_all(result.to_string().as_bytes()).unwrap();
        file.flush().unwrap();
    }

    #[tokio::test]
    async fn test_to_voice() {
        let project_dir = env!("CARGO_MANIFEST_DIR");
        dotenv::from_filename(format!("{}/.env.sample", project_dir)).ok();

        let target_file_string = format!("{}/results/output/voice/test_voice.wav", project_dir);
        let target_file = target_file_string.as_str();
        println!("project_dir: {}", target_file);

        let mut data = Data::new(Some(14), target_file);
        let result = data.create_voice("テストしています、いかがですか？").await;
        assert!(result.is_ok(), "Error: {:?}", result);
    }
}
