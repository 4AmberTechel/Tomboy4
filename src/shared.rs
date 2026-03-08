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

const STRUCTURED_DATA_JSON: &str = r#"<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "Person",
  "name": "Amber Techel",
  "jobTitle": "Actress, Singer-Songwriter, Model",
  "url": "https://4ambertechel.com",
  "sameAs": ["https://www.imdb.com/name/nm7720188/"]
}
</script>"#;

pub fn inject_seo_head(html: &str, title: &str, page_url: &str, base_url: &str) -> String {
    const SEO_TAG: &str = "<!-- SEO";
    let mut description = String::new();
    let mut og_image_path = "/global-images/homebackground.webp".to_string();
    let mut html = html.to_string();

    if let Some(start) = html.find(SEO_TAG) {
        if let Some(rel_end) = html[start..].find("-->") {
            let end = start + rel_end + 3;
            let inner_start = start + SEO_TAG.len();
            let inner = html[inner_start..start + rel_end].to_string();
            for line in inner.lines() {
                let line = line.trim();
                if let Some((key, val)) = line.split_once(": ") {
                    match key.trim() {
                        "description" => description = val.trim().to_string(),
                        "og:image" => og_image_path = val.trim().to_string(),
                        _ => {}
                    }
                }
            }
            html.replace_range(start..end, "");
        }
    }

    let og_image_url = if og_image_path.starts_with("http") {
        og_image_path.clone()
    } else {
        format!("{}{}", base_url, og_image_path)
    };

    let canonical_url = format!("{}{}", base_url, page_url);
    let escaped_desc = html_escape(&description);
    let escaped_title = html_escape(&format!("{} - 4AmberTechel", title));

    let meta_description = if description.is_empty() {
        String::new()
    } else {
        format!(r#"<meta name="description" content="{}">"#, escaped_desc)
    };

    let og_tags = format!(
        r#"<meta property="og:type" content="website">
    <meta property="og:url" content="{canonical_url}">
    <meta property="og:title" content="{escaped_title}">
    <meta property="og:description" content="{escaped_desc}">
    <meta property="og:image" content="{og_image_url}">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{escaped_title}">
    <meta name="twitter:description" content="{escaped_desc}">
    <meta name="twitter:image" content="{og_image_url}">"#
    );

    let canonical_tag = if page_url.is_empty() {
        String::new()
    } else {
        format!(r#"<link rel="canonical" href="{}">"#, canonical_url)
    };

    html.replace("{{META_DESCRIPTION}}", &meta_description)
        .replace("{{OG_TAGS}}", &og_tags)
        .replace("{{STRUCTURED_DATA}}", STRUCTURED_DATA_JSON)
        .replace("{{CANONICAL_URL}}", &canonical_tag)
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
