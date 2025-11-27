kovi-plugin-media-transfer
==========================

[<img alt="github" src="https://img.shields.io/badge/github-araea/kovi__plugin__media__transfer-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/araea/kovi-plugin-media-transfer)
[<img alt="crates.io" src="https://img.shields.io/crates/v/kovi-plugin-media-transfer.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/kovi-plugin-media-transfer)

Kovi 的媒体与链接互转插件。专为解决无法直接复制表情包链接或预览长链接媒体的痛点而设计。

## 特性

- 🔗 **链接提取** - 快速提取图片、视频、闪照的真实下载直链
- 🎬 **媒体预览** - 将文本 URL 解析并以图片或视频形式发送
- 🧠 **智能判断** - 自动识别引用消息，自动根据后缀或指令判断媒体类型
- ⚙️ **高度配置** - 自定义触发指令、前缀

## 前置

1. 创建 Kovi 项目
2. 执行 `cargo kovi add media-transfer`
3. 在 `src/main.rs` 中添加 `kovi_plugin_media_transfer`

## 快速开始

1. **提取链接**：引用一张别人的表情包或视频，发送 `转链接`。
2. **预览图片**：发送 `转图片 https://example.com/image.png`。
3. **预览视频**：发送 `转视频 https://example.com/video.mp4`。

## 指令速查

### 提取链接 (Media -> URL)

| 默认指令 | 触发方式 | 说明 |
|:---|:---|:---|
| `转链接`<br>`看链接`<br>`提取地址`<br>`url` | 引用含媒体的消息<br>或<br>指令与图片同条发送 | 机器人会回复该媒体文件的直链 URL |

### 媒体预览 (URL -> Media)

| 默认指令 | 触发方式 | 说明 |
|:---|:---|:---|
| `转图片`<br>`转视频`<br>`预览`<br>`看看` | 指令 + URL | 机器人会将链接内容以媒体形式发出。<br>若后缀为 `.mp4` 或指令含“视频”，将发送视频消息。 |

> 💡 指令支持自定义前缀（配置中默认留空，即不需要前缀）。

## 配置

资源目录：`data/kovi-plugin-media-transfer/config.toml`

> 首次运行时自动生成。

```toml
# 插件开关
enabled = true

# 指令前缀 (留空则不需要前缀)
prefixes = []

# 【转链接】指令：提取图片/视频的 URL
# 触发方式：发送指令并引用消息，或指令与图片同条发送
cmd_to_url = ["转链接", "看链接", "提取地址", "url"]

# 【转媒体】指令：将 URL 解析为图片/视频发送
# 触发方式：指令 + URL
cmd_to_media = ["转图片", "转视频", "预览", "看看"]
```

## 致谢

- [Kovi](https://kovi.threkork.com/)

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
