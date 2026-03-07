use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Testimonial {
    pub quote: String,
    pub author: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct TestimonialsData {
    pub testimonials: Vec<Testimonial>,
}

pub fn is_image_extension(ext: &str) -> bool {
    matches!(ext, "png" | "jpg" | "jpeg" | "PNG" | "JPG" | "JPEG")
}

pub fn url_encode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '+' => "%2B".to_string(),
            '#' => "%23".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '?' => "%3F".to_string(),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => {
                c.to_string()
            }
            _ => {
                let mut buf = [0u8; 4];
                let len = c.len_utf8();
                c.encode_utf8(&mut buf);
                buf[..len].iter().map(|b| format!("%{:02X}", b)).collect()
            }
        })
        .collect()
}

pub fn read_testimonials() -> Vec<Testimonial> {
    let json_path = Path::new("templates").join("reviews").join("reviews.json");

    if json_path.exists()
        && let Ok(content) = fs::read_to_string(&json_path)
        && let Ok(data) = serde_json::from_str::<TestimonialsData>(&content)
    {
        return data.testimonials;
    }

    Vec::new()
}

pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn generate_testimonials_html(testimonials: &[Testimonial]) -> String {
    testimonials
        .iter()
        .map(|t| {
            format!(
                r#"<div class="testimonial-card">
                <div class="testimonial-text">
                    <p>{}</p>
                </div>
                <div class="testimonial-author">
                    <span class="author-name">{}</span>
                    <span class="author-title">{}</span>
                </div>
            </div>"#,
                html_escape(&t.quote),
                html_escape(&t.author),
                html_escape(&t.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n            ")
}

pub fn extract_youtube_id(url: &str) -> Option<String> {
    if url.contains("youtu.be/") {
        url.split("youtu.be/")
            .nth(1)
            .map(|s| s.split('?').next().unwrap_or(s).to_string())
    } else if url.contains("youtube.com/watch") {
        url.split("v=")
            .nth(1)
            .map(|s| s.split('&').next().unwrap_or(s).to_string())
    } else {
        None
    }
}

pub fn read_youtube_links(folder: &str) -> Vec<String> {
    let links_file = Path::new("templates").join(folder).join("youtubeLinks.txt");
    let mut video_ids = Vec::new();

    if links_file.exists()
        && let Ok(content) = fs::read_to_string(&links_file)
    {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(id) = extract_youtube_id(line) {
                video_ids.push(id);
            }
        }
    }

    video_ids
}

pub fn generate_youtube_embeds(video_ids: &[String]) -> String {
    video_ids
		.iter()
		.map(|id| {
			format!(
				r#"<div class="youtube-video-wrapper">
                    <iframe src="https://www.youtube.com/embed/{}" frameborder="0" allowfullscreen></iframe>
                </div>"#,
				id
			)
		})
		.collect::<Vec<_>>()
		.join("\n")
}

pub fn read_links_file(images_dir: &Path) -> HashMap<String, String> {
    let mut links = HashMap::new();
    let links_file = images_dir.join("Links.txt");

    if links_file.exists()
        && let Ok(content) = fs::read_to_string(&links_file)
    {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((name, url)) = line.split_once(',') {
                links.insert(name.trim().to_string(), url.trim().to_string());
            }
        }
    }

    links
}
