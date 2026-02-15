use std::fs;
use std::path::Path;
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Testimonial {
    quote: String,
    author: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct TestimonialsData {
    testimonials: Vec<Testimonial>,
}

fn url_encode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '+' => "%2B".to_string(),
            '#' => "%23".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '?' => "%3F".to_string(),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn read_testimonials() -> Vec<Testimonial> {
    let yaml_path = Path::new("templates").join("reviews").join("reviews.yaml");

    if yaml_path.exists() {
        if let Ok(content) = fs::read_to_string(&yaml_path) {
            if let Ok(data) = serde_yaml::from_str::<TestimonialsData>(&content) {
                return data.testimonials;
            }
        }
    }

    Vec::new()
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn generate_testimonials_html(testimonials: &[Testimonial]) -> String {
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

fn get_git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "dev".to_string())
}

fn generate_page(title: &str, content: &str, version: &str) -> String {
    let base_template = include_str!("../templates/base.html");
    let mut final_html = base_template
        .replace("{{TITLE}}", title)
        .replace("{{CONTENT}}", content);

    // Update navigation links for GitHub Pages (static generation)
    final_html = final_html.replace(
        r#"<a href="/" class="nav-item">Home</a>"#,
        r#"<a href="/Website-test/index.html" class="nav-item">Home</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/bio/" class="nav-item">Bio</a>"#,
        r#"<a href="/Website-test/bio/index.html" class="nav-item">Bio</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/acting/" class="nav-item">Acting</a>"#,
        r#"<a href="/Website-test/acting/index.html" class="nav-item">Acting</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/music/" class="nav-item">Music</a>"#,
        r#"<a href="/Website-test/music/index.html" class="nav-item">Music</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/modeling/" class="nav-item">Modeling</a>"#,
        r#"<a href="/Website-test/modeling/index.html" class="nav-item">Modeling</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/reviews/" class="nav-item">Reviews</a>"#,
        r#"<a href="/Website-test/reviews/index.html" class="nav-item">Reviews</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/behind-the-scenes/" class="nav-item">Behind the Scenes</a>"#,
        r#"<a href="/Website-test/behind-the-scenes/index.html" class="nav-item">Behind the Scenes</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/contact/" class="nav-item">Contact</a>"#,
        r#"<a href="/Website-test/contact/index.html" class="nav-item">Contact</a>"#
    );

    // Update image paths for GitHub Pages deployment
    final_html = final_html.replace(
        r#"src="/templates/global-images/"#,
        r#"src="/Website-test/global-images/"#
    );

    // Update background image paths for GitHub Pages deployment
    final_html = final_html.replace(
        r#"url('/templates/global-images/"#,
        r#"url('/Website-test/global-images/"#
    );

    // Update CSS path for GitHub Pages deployment with cache busting
    final_html = final_html.replace(
        r#"href="/templates/styles.css""#,
        &format!(r#"href="/Website-test/styles.css?v={}""#, version)
    );

    final_html
}

fn get_image_list_for_web(images_dir: &Path, category: &str) -> Vec<String> {
    let mut images = Vec::new();

    if images_dir.exists() {
        if let Ok(entries) = fs::read_dir(images_dir) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if matches!(extension.to_str(), Some("png") | Some("jpg") | Some("jpeg")) {
                        if let Some(filename) = entry.file_name().to_str() {
                            let url_encoded_filename = url_encode(filename);
                            images.push(format!("./{}/images/{}", category, url_encoded_filename));
                        }
                    }
                }
            }
        }
    }

    images.sort();
    images
}

fn copy_images(source_dir: &Path, dest_dir: &Path) {
    if !source_dir.exists() {
        return;
    }

    if fs::create_dir_all(dest_dir).is_err() {
        println!("Failed to create directory: {:?}", dest_dir);
        return;
    }

    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if matches!(extension.to_str(), Some("png") | Some("jpg") | Some("jpeg")) {
                    if let Some(filename) = entry.file_name().to_str() {
                        let source_file = source_dir.join(filename);
                        let dest_file = dest_dir.join(filename);

                        if let Err(e) = fs::copy(&source_file, &dest_file) {
                            println!("Failed to copy {:?} to {:?}: {}", source_file, dest_file, e);
                        } else {
                            println!("Copied image: {}", filename);
                        }
                    }
                }
            }
        }
    }
}

use std::collections::HashMap;

struct CategoryData {
    title: String,
    subtitle: String,
    images: Vec<String>,
    links: HashMap<String, String>,
    background: Option<String>,
}

fn read_youtube_links(folder: &str) -> Vec<String> {
    let links_file = Path::new("templates").join(folder).join("youtubeLinks.txt");
    let mut video_ids = Vec::new();

    if links_file.exists() {
        if let Ok(content) = fs::read_to_string(&links_file) {
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
    }

    video_ids
}

fn extract_youtube_id(url: &str) -> Option<String> {
    if url.contains("youtu.be/") {
        url.split("youtu.be/").nth(1).map(|s| s.split('?').next().unwrap_or(s).to_string())
    } else if url.contains("youtube.com/watch") {
        url.split("v=").nth(1).map(|s| s.split('&').next().unwrap_or(s).to_string())
    } else {
        None
    }
}

fn generate_youtube_embeds(video_ids: &[String]) -> String {
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

fn read_links_file(images_dir: &Path) -> HashMap<String, String> {
    let mut links = HashMap::new();
    let links_file = images_dir.join("Links.txt");

    if links_file.exists() {
        if let Ok(content) = fs::read_to_string(&links_file) {
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
    }

    links
}

fn discover_modeling_categories(docs_dir: &Path) -> Vec<(String, CategoryData)> {
    let mut categories = Vec::new();
    let modeling_dir = Path::new("templates").join("modeling");

    if !modeling_dir.exists() {
        return categories;
    }

    if let Ok(entries) = fs::read_dir(&modeling_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let category_name = entry.file_name().to_str().unwrap_or("").to_string();

                let images_dir = entry.path().join("images");
                if !images_dir.exists() {
                    continue;
                }

                let title = {
                    let mut chars = category_name.chars();
                    match chars.next() {
                        None => continue,
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                };

                let subtitle_file = entry.path().join("subtitle.txt");
                let subtitle = if subtitle_file.exists() {
                    fs::read_to_string(&subtitle_file)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|_| format!("Professional {} photography", category_name))
                } else {
                    format!("Professional {} photography", category_name)
                };

                let docs_images_dir = docs_dir.join("modeling").join(&category_name).join("images");
                copy_images(&images_dir, &docs_images_dir);

                let images = get_image_list_for_web(&images_dir, &category_name);
                let links = read_links_file(&images_dir);

                // Check for and copy background image
                let background_dir = entry.path().join("Background");
                let background = if background_dir.join("bkgrnd.png").exists() {
                    let docs_bg_dir = docs_dir.join("modeling").join(&category_name).join("Background");
                    create_dir_if_not_exists(&docs_bg_dir);
                    copy_images(&background_dir, &docs_bg_dir);
                    Some(format!("./{}/Background/bkgrnd.png", category_name))
                } else {
                    None
                };

                categories.push((category_name, CategoryData {
                    title,
                    subtitle,
                    images,
                    links,
                    background,
                }));
            }
        }
    }

    categories.sort_by(|a, b| a.0.cmp(&b.0));
    categories
}

fn generate_categories_json(categories: &[(String, CategoryData)]) -> String {
    let mut json_parts = Vec::new();

    for (key, data) in categories {
        let images_json: Vec<String> = data.images.iter().map(|img| format!("\"{}\"", img)).collect();
        let escaped_subtitle = data.subtitle
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", " ")
            .replace("\r", "");

        let links_json: Vec<String> = data.links.iter()
            .map(|(k, v)| format!("\"{}\": \"{}\"", k, v.replace("\"", "\\\"")))
            .collect();

        let background_json = match &data.background {
            Some(bg) => format!(", \"background\": \"{}\"", bg),
            None => String::new(),
        };

        json_parts.push(format!(
            "\"{}\": {{\"title\": \"{}\", \"subtitle\": \"{}\", \"images\": [{}], \"links\": {{{}}}{}}}",
            key,
            data.title,
            escaped_subtitle,
            images_json.join(", "),
            links_json.join(", "),
            background_json
        ));
    }

    format!("{{{}}}", json_parts.join(", "))
}

fn generate_modeling_page(content: &str, categories: &[(String, CategoryData)], version: &str) -> String {
    let categories_json = generate_categories_json(categories);
    let updated_content = content.replace("{{CATEGORIES_JSON}}", &categories_json);

    let base_template = include_str!("../templates/base.html");
    let mut final_html = base_template
        .replace("{{TITLE}}", "Modeling Portfolio")
        .replace("{{CONTENT}}", &updated_content);

    // Update navigation links for GitHub Pages (modeling page)
    final_html = final_html.replace(
        r#"<a href="/" class="nav-item">Home</a>"#,
        r#"<a href="/Website-test/index.html" class="nav-item">Home</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/bio/" class="nav-item">Bio</a>"#,
        r#"<a href="/Website-test/bio/index.html" class="nav-item">Bio</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/acting/" class="nav-item">Acting</a>"#,
        r#"<a href="/Website-test/acting/index.html" class="nav-item">Acting</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/music/" class="nav-item">Music</a>"#,
        r#"<a href="/Website-test/music/index.html" class="nav-item">Music</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/modeling/" class="nav-item">Modeling</a>"#,
        r#"<a href="/Website-test/modeling/index.html" class="nav-item">Modeling</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/reviews/" class="nav-item">Reviews</a>"#,
        r#"<a href="/Website-test/reviews/index.html" class="nav-item">Reviews</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/behind-the-scenes/" class="nav-item">Behind the Scenes</a>"#,
        r#"<a href="/Website-test/behind-the-scenes/index.html" class="nav-item">Behind the Scenes</a>"#
    );
    final_html = final_html.replace(
        r#"<a href="/contact/" class="nav-item">Contact</a>"#,
        r#"<a href="/Website-test/contact/index.html" class="nav-item">Contact</a>"#
    );

    // Update CSS path for GitHub Pages deployment with cache busting
    final_html = final_html.replace(
        r#"href="/templates/styles.css""#,
        &format!(r#"href="/Website-test/styles.css?v={}""#, version)
    );

    final_html
}

fn create_dir_if_not_exists(path: &Path) {
    if !path.exists() {
        fs::create_dir_all(path).unwrap_or_else(|_| panic!("Failed to create directory: {:?}", path));
    }
}

fn main() {
    let docs_dir = Path::new("docs");
    let version = get_git_hash();
    println!("Building with version: {}", version);

    // Clean and rebuild the entire docs directory structure
    if docs_dir.exists() {
        fs::remove_dir_all(docs_dir).expect("Failed to remove existing docs directory");
        println!("Cleaned existing docs directory");
    }

    create_dir_if_not_exists(docs_dir);

    // Copy CSS file from templates to docs
    let templates_css = Path::new("templates").join("styles.css");
    let docs_css = docs_dir.join("styles.css");

    if templates_css.exists() {
        if let Err(e) = fs::copy(&templates_css, &docs_css) {
            println!("Failed to copy CSS file: {}", e);
        } else {
            println!("Copied styles.css to docs directory");
        }
    }

    // Copy global images folder from templates to docs
    let templates_global_images = Path::new("templates").join("global-images");
    let docs_global_images = docs_dir.join("global-images");

    if templates_global_images.exists() {
        create_dir_if_not_exists(&docs_global_images);
        copy_images(&templates_global_images, &docs_global_images);
    }

    // Create modeling subdirectory
    create_dir_if_not_exists(&docs_dir.join("modeling"));

    // Discover modeling categories and copy their images
    let categories = discover_modeling_categories(docs_dir);
    println!("\nModeling categories discovered:");
    for (name, data) in &categories {
        println!("  - {} ({} images)", name, data.images.len());
    }

    // Generate home page
    let home_content = include_str!("../templates/index.html");
    let home_html = generate_page("Home", home_content, &version);
    let home_file_path = docs_dir.join("index.html");
    fs::write(&home_file_path, home_html).expect("Failed to write index.html");
    println!("Generated index.html");

    // Generate unified modeling page
    let modeling_content = include_str!("../templates/modeling/modeling.html");
    let modeling_html = generate_modeling_page(modeling_content, &categories, &version);
    let modeling_path = docs_dir.join("modeling").join("index.html");
    fs::write(&modeling_path, modeling_html).expect("Failed to write modeling/index.html");
    println!("Generated modeling/index.html");

    // Generate bio page
    let bio_dir = docs_dir.join("bio");
    create_dir_if_not_exists(&bio_dir);

    // Copy bio background image
    let bio_bg_src = Path::new("templates").join("bio").join("background");
    let bio_bg_dest = bio_dir.join("background");
    if bio_bg_src.exists() {
        create_dir_if_not_exists(&bio_bg_dest);
        copy_images(&bio_bg_src, &bio_bg_dest);
    }

    let bio_path = Path::new("templates").join("bio").join("bio.html");
    if bio_path.exists() {
        match fs::read_to_string(&bio_path) {
            Ok(content) => {
                // Update background image path for GitHub Pages
                let updated_content = content.replace(
                    "url('/templates/bio/background/bkgrnd.png')",
                    "url('./background/bkgrnd.png')"
                );
                let html = generate_page("Bio", &updated_content, &version);
                let file_path = bio_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write bio/index.html");
                println!("Generated bio/index.html");
            },
            Err(e) => {
                println!("Failed to read bio template: {}", e);
            }
        }
    }

    // Generate music page
    let music_dir = docs_dir.join("music");
    create_dir_if_not_exists(&music_dir);

    // Copy music background image
    let music_bg_src = Path::new("templates").join("music").join("background");
    let music_bg_dest = music_dir.join("background");
    if music_bg_src.exists() {
        create_dir_if_not_exists(&music_bg_dest);
        copy_images(&music_bg_src, &music_bg_dest);
    }

    let music_path = Path::new("templates").join("music").join("music.html");
    if music_path.exists() {
        match fs::read_to_string(&music_path) {
            Ok(content) => {
                // Generate YouTube embeds
                let video_ids = read_youtube_links("music");
                let embeds_html = generate_youtube_embeds(&video_ids);
                let content = content.replace("{{YOUTUBE_EMBEDS}}", &embeds_html);

                // Update background image path for GitHub Pages
                let updated_content = content.replace(
                    "url('/templates/music/background/bkgrnd.png')",
                    "url('./background/bkgrnd.png')"
                );
                let html = generate_page("Music", &updated_content, &version);
                let file_path = music_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write music/index.html");
                println!("Generated music/index.html");
            },
            Err(e) => {
                println!("Failed to read music template: {}", e);
            }
        }
    }

    // Generate contact page
    let contact_dir = docs_dir.join("contact");
    create_dir_if_not_exists(&contact_dir);

    let contact_path = Path::new("templates").join("contact").join("contact.html");
    if contact_path.exists() {
        match fs::read_to_string(&contact_path) {
            Ok(content) => {
                let html = generate_page("Contact", &content, &version);
                let file_path = contact_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write contact/index.html");
                println!("Generated contact/index.html");
            },
            Err(e) => {
                println!("Failed to read contact template: {}", e);
            }
        }
    }

    // Generate acting page
    let acting_dir = docs_dir.join("acting");
    create_dir_if_not_exists(&acting_dir);

    // Copy acting background image
    let acting_bg_src = Path::new("templates").join("acting").join("Background");
    let acting_bg_dest = acting_dir.join("Background");
    if acting_bg_src.exists() {
        create_dir_if_not_exists(&acting_bg_dest);
        copy_images(&acting_bg_src, &acting_bg_dest);
    }

    let acting_path = Path::new("templates").join("acting").join("acting.html");
    if acting_path.exists() {
        match fs::read_to_string(&acting_path) {
            Ok(content) => {
                // Generate YouTube embeds
                let video_ids = read_youtube_links("acting");
                let embeds_html = generate_youtube_embeds(&video_ids);
                let content = content.replace("{{ACTING_YOUTUBE_EMBEDS}}", &embeds_html);

                // Update background image path for GitHub Pages
                let updated_content = content.replace(
                    "url('/templates/acting/Background/bckgrnd.png')",
                    "url('./Background/bckgrnd.png')"
                );
                let html = generate_page("Acting", &updated_content, &version);
                let file_path = acting_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write acting/index.html");
                println!("Generated acting/index.html");
            },
            Err(e) => {
                println!("Failed to read acting template: {}", e);
            }
        }
    }

    // Generate reviews page
    let reviews_dir = docs_dir.join("reviews");
    create_dir_if_not_exists(&reviews_dir);

    let reviews_path = Path::new("templates").join("reviews").join("reviews.html");
    if reviews_path.exists() {
        match fs::read_to_string(&reviews_path) {
            Ok(mut content) => {
                let testimonials = read_testimonials();
                let testimonials_html = generate_testimonials_html(&testimonials);
                content = content.replace("{{TESTIMONIALS_HTML}}", &testimonials_html);
                let html = generate_page("Reviews", &content, &version);
                let file_path = reviews_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write reviews/index.html");
                println!("Generated reviews/index.html");
            },
            Err(e) => {
                println!("Failed to read reviews template: {}", e);
            }
        }
    }

    // Generate behind-the-scenes page
    let bts_dir = docs_dir.join("behind-the-scenes");
    create_dir_if_not_exists(&bts_dir);

    // Copy BTS images
    let bts_images_src = Path::new("templates").join("Behind the scenes").join("images");
    let bts_images_dest = bts_dir.join("images");
    if bts_images_src.exists() {
        create_dir_if_not_exists(&bts_images_dest);
        copy_images(&bts_images_src, &bts_images_dest);
    }

    // Copy BTS background image
    let bts_bg_src = Path::new("templates").join("Behind the scenes").join("background");
    let bts_bg_dest = bts_dir.join("background");
    if bts_bg_src.exists() {
        create_dir_if_not_exists(&bts_bg_dest);
        copy_images(&bts_bg_src, &bts_bg_dest);
    }

    let bts_path = Path::new("templates").join("Behind the scenes").join("behind-the-scenes.html");
    if bts_path.exists() {
        match fs::read_to_string(&bts_path) {
            Ok(content) => {
                // Read subtitle
                let subtitle_file = Path::new("templates").join("Behind the scenes").join("subtitle.txt");
                let subtitle = if subtitle_file.exists() {
                    fs::read_to_string(&subtitle_file)
                        .unwrap_or_else(|_| "Behind the scenes photography".to_string())
                        .trim()
                        .to_string()
                } else {
                    "Behind the scenes photography".to_string()
                };

                // Get images list
                let mut images = Vec::new();
                if bts_images_src.exists() {
                    if let Ok(entries) = fs::read_dir(&bts_images_src) {
                        for entry in entries.flatten() {
                            if let Some(extension) = entry.path().extension() {
                                if matches!(extension.to_str(), Some("png") | Some("jpg") | Some("jpeg")) {
                                    if let Some(filename) = entry.file_name().to_str() {
                                        let url_encoded_filename = url_encode(filename);
                                        images.push(format!("./images/{}", url_encoded_filename));
                                    }
                                }
                            }
                        }
                    }
                }
                images.sort();

                let images_json: Vec<String> = images.iter().map(|img| format!("\"{}\"", img)).collect();
                let images_json_str = format!("[{}]", images_json.join(", "));

                // Update background image path for GitHub Pages
                let updated_content = content
                    .replace("{{BTS_IMAGES_JSON}}", &images_json_str)
                    .replace("{{BTS_SUBTITLE}}", &subtitle)
                    .replace(
                        "url('/templates/Behind the scenes/background/bkgrnd.png')",
                        "url('./background/bkgrnd.png')"
                    );

                let html = generate_page("Behind the Scenes", &updated_content, &version);
                let file_path = bts_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write behind-the-scenes/index.html");
                println!("Generated behind-the-scenes/index.html");
            },
            Err(e) => {
                println!("Failed to read behind-the-scenes template: {}", e);
            }
        }
    }

    println!("\nStatic files generated successfully!");
}
