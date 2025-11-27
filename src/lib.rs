//! kovi-plugin-media-transfer
//!
//! 一个便捷的媒体与链接互转工具。
//! 功能 1: 提取图片/视频消息的直链 (URL)。
//! 功能 2: 将文本链接解析并以图片/视频形式发送 (预览)。

// =============================
//          Modules
// =============================

mod config {
    use kovi::toml;
    use kovi::utils::{load_toml_data, save_toml_data};
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    pub static CONFIG: std::sync::OnceLock<Arc<RwLock<Config>>> = std::sync::OnceLock::new();

    pub fn get() -> Arc<RwLock<Config>> {
        CONFIG.get().cloned().expect("Config not initialized")
    }

    const DEFAULT_CONFIG: &str = r#"
# 插件开关
enabled = true

# 指令前缀 (留空则不需要前缀)
prefixes = []

# 【转链接】指令：提取图片/视频的 URL
# 触发方式：发送指令并引用消息，或指令与图片同条发送
cmd_to_url = ["转链接", "看链接", "提取地址", "url"]

# 【转媒体】指令：将 URL 解析为图片/视频发送
# 触发方式：指令 + URL，或 指令 + 引用包含URL的消息
cmd_to_media = ["转图片", "转视频", "预览", "看看"]
"#;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Config {
        pub enabled: bool,
        pub prefixes: Vec<String>,
        pub cmd_to_url: Vec<String>,
        pub cmd_to_media: Vec<String>,

        #[serde(skip)]
        config_path: PathBuf,
    }

    impl Config {
        pub fn load(data_dir: PathBuf) -> Arc<RwLock<Self>> {
            if !data_dir.exists() {
                std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
            }
            let config_path = data_dir.join("config.toml");

            let default: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
            let mut config = load_toml_data(default, config_path.clone()).unwrap_or_else(|_| {
                let c: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
                c
            });

            config.config_path = config_path;

            Arc::new(RwLock::new(config))
        }

        pub fn save(&self) {
            let _ = save_toml_data(self, &self.config_path);
        }
    }
}

mod utils {
    use kovi::MsgEvent;
    use regex::Regex;
    use std::sync::{Arc, OnceLock};

    pub static URL_REGEX: OnceLock<Regex> = OnceLock::new();

    /// 解析指令，返回 (是否匹配, 剩余参数, 匹配到的原始指令)
    pub fn parse_command(
        text: &str,
        prefixes: &[String],
        commands: &[String],
    ) -> (bool, String, String) {
        let text = text.trim();

        // 1. 处理前缀
        let clean_text = if !prefixes.is_empty() {
            let mut found = None;
            let mut sorted_prefixes = prefixes.to_vec();
            sorted_prefixes.sort_by_key(|b| std::cmp::Reverse(b.len()));

            for p in sorted_prefixes {
                if text.starts_with(&p) {
                    found = Some(&text[p.len()..]);
                    break;
                }
            }
            match found {
                Some(t) => t.trim(),
                None => return (false, String::new(), String::new()),
            }
        } else {
            text
        };

        // 2. 匹配指令
        // 优先匹配长指令，防止包含关系导致误判
        let mut sorted_cmds = commands.to_vec();
        sorted_cmds.sort_by_key(|b| std::cmp::Reverse(b.len()));

        for cmd in sorted_cmds {
            if clean_text.starts_with(&cmd) {
                let args = clean_text[cmd.len()..].trim().to_string();
                return (true, args, cmd);
            }
        }

        (false, String::new(), String::new())
    }

    /// 提取文本中的第一个 HTTP 链接
    pub fn extract_url(text: &str) -> Option<String> {
        let re = URL_REGEX
            .get_or_init(|| Regex::new(r"https?://[^\s\u4e00-\u9fa5]+").expect("Invalid Regex"));
        re.find(text).map(|m| m.as_str().to_string())
    }

    /// 从引用消息中获取纯文本内容
    pub async fn get_reply_text(
        event: &Arc<MsgEvent>,
        bot: &Arc<kovi::RuntimeBot>,
    ) -> Option<String> {
        let reply_id = event.message.iter().find_map(|seg| {
            if seg.type_ == "reply" {
                seg.data.get("id").and_then(|v| v.as_str())
            } else {
                None
            }
        })?;

        if let Ok(reply_id_int) = reply_id.parse::<i32>()
            && let Ok(res) = bot.get_msg(reply_id_int).await
            && let Some(segments) = res.data.get("message").and_then(|v| v.as_array())
        {
            let mut text_content = String::new();
            for seg in segments {
                if let Some(type_) = seg.get("type").and_then(|t| t.as_str()) {
                    if type_ == "text" {
                        if let Some(t) = seg
                            .get("data")
                            .and_then(|d| d.get("text"))
                            .and_then(|s| s.as_str())
                        {
                            text_content.push_str(t);
                        }
                    }
                }
            }
            if !text_content.is_empty() {
                return Some(text_content);
            }
        }
        None
    }

    /// 从消息段中获取图片或视频的 URL
    /// 支持递归查找引用消息
    pub async fn find_media_url(
        event: &Arc<MsgEvent>,
        bot: &Arc<kovi::RuntimeBot>,
    ) -> Option<(String, String)> {
        // 返回 (URL, 类型: image/video)

        // 1. 检查当前消息
        for seg in event.message.iter() {
            if seg.type_ == "image" {
                if let Some(url) = seg.data.get("url").and_then(|u| u.as_str()) {
                    return Some((url.to_string(), "图片".to_string()));
                }
            } else if seg.type_ == "video"
                && let Some(url) = seg
                    .data
                    .get("url")
                    .or(seg.data.get("file"))
                    .and_then(|u| u.as_str())
            {
                return Some((url.to_string(), "视频".to_string()));
            }
        }

        // 2. 检查引用消息
        let reply_id = event.message.iter().find_map(|seg| {
            if seg.type_ == "reply" {
                seg.data.get("id").and_then(|v| v.as_str())
            } else {
                None
            }
        })?;

        if let Ok(reply_id_int) = reply_id.parse::<i32>()
            && let Ok(res) = bot.get_msg(reply_id_int).await
            && let Some(segments) = res.data.get("message").and_then(|v| v.as_array())
        {
            for seg in segments {
                if let Some(type_) = seg.get("type").and_then(|t| t.as_str()) {
                    if type_ == "image" {
                        if let Some(url) = seg
                            .get("data")
                            .and_then(|d| d.get("url"))
                            .and_then(|u| u.as_str())
                        {
                            return Some((url.to_string(), "图片".to_string()));
                        }
                    } else if type_ == "video" {
                        // 视频有时在 file 字段，有时在 url 字段
                        if let Some(url) = seg
                            .get("data")
                            .and_then(|d| d.get("url").or(d.get("file")))
                            .and_then(|u| u.as_str())
                        {
                            return Some((url.to_string(), "视频".to_string()));
                        }
                    }
                }
            }
        }

        None
    }
}

// =============================
//      Main Plugin Logic
// =============================

use kovi::{Message, PluginBuilder};

#[kovi::plugin]
async fn main() {
    let bot = PluginBuilder::get_runtime_bot();
    let data_dir = bot.get_data_path();

    // 加载配置
    let config_lock = config::Config::load(data_dir.clone());
    config::CONFIG.set(config_lock.clone()).ok();

    PluginBuilder::on_msg(move |event| {
        let bot = bot.clone();
        let config_lock = config_lock.clone();

        async move {
            let text = match event.borrow_text() {
                Some(t) => t,
                None => return,
            };

            let (enabled, prefixes, cmd_to_url, cmd_to_media) = {
                let cfg = config_lock.read().unwrap();
                (
                    cfg.enabled,
                    cfg.prefixes.clone(),
                    cfg.cmd_to_url.clone(),
                    cfg.cmd_to_media.clone(),
                )
            };

            if !enabled {
                return;
            }

            // ----------------------------------------------------
            // 功能 1: 转链接 (Media -> URL)
            // ----------------------------------------------------
            let (is_match, _, _) = utils::parse_command(text, &prefixes, &cmd_to_url);
            if is_match {
                match utils::find_media_url(&event, &bot).await {
                    Some((url, type_name)) => {
                        let msg = Message::new()
                            .add_reply(event.message_id)
                            .add_text(format!("🔗 已提取{}:\n{}", type_name, url));
                        event.reply(msg);
                    }
                    None => {
                        event.reply("⚠️ 未检测到媒体文件。\n请【引用】一条包含图片或视频的消息，或在发送指令时附带图片。");
                    }
                }
                return; // 命中指令后直接返回
            }

            // ----------------------------------------------------
            // 功能 2: 转媒体 (URL -> Media)
            // ----------------------------------------------------
            let (is_match, args, raw_cmd) = utils::parse_command(text, &prefixes, &cmd_to_media);
            if is_match {
                // 1. 尝试从指令参数中提取 URL
                let mut target_url = utils::extract_url(&args);

                // 2. 如果参数没有 URL，尝试从引用消息的文本中提取
                if target_url.is_none() {
                    if let Some(reply_text) = utils::get_reply_text(&event, &bot).await {
                        target_url = utils::extract_url(&reply_text);
                    }
                }

                let url = match target_url {
                    Some(u) => u,
                    None => {
                        event.reply(
                            "⚠️ 未检测到有效链接。\n请在指令后附带 URL，或【引用】一条包含 URL 的消息。",
                        );
                        return;
                    }
                };

                // 判断是否发送为视频
                // 1. 指令中包含 "视频" 二字 (如 "转视频")
                // 2. 链接以常见视频后缀结尾
                let is_video_cmd = raw_cmd.contains("视频");
                let is_video_ext = url.ends_with(".mp4") || url.ends_with(".mov");

                let msg = Message::new().add_reply(event.message_id);

                if is_video_cmd || is_video_ext {
                    // 构建视频消息
                    let mut vec = Vec::new();
                    let segment = kovi::bot::message::Segment::new(
                        "video",
                        kovi::serde_json::json!({
                            "file": url
                        }),
                    );
                    vec.push(segment);
                    let video_msg = kovi::bot::message::Message::from(vec);
                    event.reply(video_msg);
                } else {
                    // 默认为图片
                    event.reply(msg.add_image(&url));
                }
            }
        }
    });

    // 插件卸载/退出时的清理与保存
    PluginBuilder::drop(move || {
        let config_lock = config::get();
        async move {
            let config = {
                let guard = config_lock.read().unwrap();
                guard.clone()
            };
            config.save();
        }
    });
}
