use std::path::PathBuf;

pub fn target_path_from_env(target: &str) -> PathBuf {
    let target = std::env::var(target).unwrap_or("./".to_string());
    let target_path = PathBuf::from(target.clone());
    if target_path.is_absolute() {
        target_path
    } else {
        let current_dir = std::env::current_dir().unwrap();
        // 相対パスのコンポーネントを処理
        let mut result_path = current_dir;
        for component in target_path.components() {
            match component {
                std::path::Component::CurDir => {
                    // ./ は無視
                    continue;
                }
                std::path::Component::ParentDir => {
                    // ../ は親ディレクトリに移動
                    result_path.pop();
                }
                _ => {
                    // それ以外は追加
                    result_path.push(component);
                }
            }
        }
        result_path
    }
}
