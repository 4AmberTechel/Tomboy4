use axum::{response::Html, routing::get, Router, http::header}; // TODO: Re-add contact page - Form removed
use serde::Deserialize;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

fn generate_page(title: &str, content: &str) -> String {
    let base_template = include_str!("../templates/base.html");
    base_template
        .replace("{{TITLE}}", title)
        .replace("{{CONTENT}}", content)
}

fn get_image_list(images_dir: &Path, category: &str) -> Vec<String> {
    let mut images = Vec::new();

    if images_dir.exists() {
        if let Ok(entries) = fs::read_dir(images_dir) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension.to_str() == Some("png") || extension.to_str() == Some("jpg") || extension.to_str() == Some("jpeg") {
                        if let Some(filename) = entry.file_name().to_str() {
                            let url_encoded_filename = url_encode(filename);
                            images.push(format!("/templates/modeling/{}/images/{}", category, url_encoded_filename));
                        }
                    }
                }
            }
        }
    }

    images.sort();
    images
}

#[derive(Clone, Debug)]
struct PageTemplate {
    title: String,
    content: String,
}

#[derive(Clone, Debug)]
struct CategoryData {
    title: String,
    subtitle: String,
    images: Vec<String>,
    links: HashMap<String, String>,
    background: Option<String>,
}

// TODO: Re-add contact page - ContactForm struct for form submissions
/*
#[derive(Deserialize)]
struct ContactForm {
    name: String,
    email: Option<String>,
    subject: String,
    message: String,
}
*/

fn discover_templates() -> Result<HashMap<String, PageTemplate>, Box<dyn std::error::Error>> {
    let mut templates = HashMap::new();
    let templates_dir = Path::new("templates");

    // Handle index.html as root page
    let index_path = templates_dir.join("index.html");
    if index_path.exists() {
        let content = fs::read_to_string(&index_path)?;
        templates.insert("/".to_string(), PageTemplate {
            title: "Home".to_string(),
            content,
        });
    }

    // TODO: Re-add contact page - Currently showing under construction
    // Original contact page implementation commented out below:
    /*
    let contact_path = templates_dir.join("contact").join("contact.html");
    if contact_path.exists() {
        let content = fs::read_to_string(&contact_path)?;
        templates.insert("/contact/".to_string(), PageTemplate {
            title: "Contact".to_string(),
            content,
        });
    }
    */

    // Temporary under construction contact page
    let contact_construction_path = templates_dir.join("contact").join("contact-under-construction.html");
    if contact_construction_path.exists() {
        let content = fs::read_to_string(&contact_construction_path)?;
        templates.insert("/contact/".to_string(), PageTemplate {
            title: "Contact - Under Construction".to_string(),
            content,
        });
    }

    // Handle unified modeling page
    let modeling_path = templates_dir.join("modeling").join("modeling.html");
    if modeling_path.exists() {
        let content = fs::read_to_string(&modeling_path)?;
        templates.insert("/modeling/".to_string(), PageTemplate {
            title: "Modeling Portfolio".to_string(),
            content,
        });
    }

    // Handle bio page
    let bio_path = templates_dir.join("bio").join("bio.html");
    if bio_path.exists() {
        let content = fs::read_to_string(&bio_path)?;
        templates.insert("/bio/".to_string(), PageTemplate {
            title: "Bio".to_string(),
            content,
        });
    }

    // Handle music page
    let music_path = templates_dir.join("music").join("music.html");
    if music_path.exists() {
        let content = fs::read_to_string(&music_path)?;
        templates.insert("/music/".to_string(), PageTemplate {
            title: "Music".to_string(),
            content,
        });
    }

    // Handle acting page
    let acting_path = templates_dir.join("acting").join("acting.html");
    if acting_path.exists() {
        let content = fs::read_to_string(&acting_path)?;
        templates.insert("/acting/".to_string(), PageTemplate {
            title: "Acting".to_string(),
            content,
        });
    }

    // Handle reviews page
    let reviews_path = templates_dir.join("reviews").join("reviews.html");
    if reviews_path.exists() {
        let content = fs::read_to_string(&reviews_path)?;
        templates.insert("/reviews/".to_string(), PageTemplate {
            title: "Reviews".to_string(),
            content,
        });
    }

    // Handle behind-the-scenes page
    let bts_path = templates_dir.join("Behind the scenes").join("behind-the-scenes.html");
    if bts_path.exists() {
        let content = fs::read_to_string(&bts_path)?;
        templates.insert("/behind-the-scenes/".to_string(), PageTemplate {
            title: "Behind the Scenes".to_string(),
            content,
        });
    }

    Ok(templates)
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
                // Extract video ID from youtu.be/ID or youtube.com/watch?v=ID
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

fn discover_modeling_categories() -> HashMap<String, CategoryData> {
    let mut categories = HashMap::new();
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

                let images = get_image_list(&images_dir, &category_name);
                let links = read_links_file(&images_dir);

                // Check for background image
                let background_dir = entry.path().join("Background");
                let background = if background_dir.join("bkgrnd.png").exists() {
                    Some(format!("/templates/modeling/{}/Background/bkgrnd.png", category_name))
                } else {
                    None
                };

                categories.insert(category_name, CategoryData {
                    title,
                    subtitle,
                    images,
                    links,
                    background,
                });
            }
        }
    }

    categories
}

fn generate_categories_json(categories: &HashMap<String, CategoryData>) -> String {
    let mut json_parts = Vec::new();

    let mut sorted_keys: Vec<_> = categories.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        let data = &categories[key];
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

fn generate_modeling_page(content: &str, categories: &HashMap<String, CategoryData>) -> String {
    let categories_json = generate_categories_json(categories);
    let updated_content = content.replace("{{CATEGORIES_JSON}}", &categories_json);

    let base_template = include_str!("../templates/base.html");
    base_template
        .replace("{{TITLE}}", "Modeling Portfolio")
        .replace("{{CONTENT}}", &updated_content)
}

// Home page handler
async fn home_page_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Html<String> {
    if let Some(template) = templates.get("/") {
        Html(generate_page(&template.title, &template.content))
    } else {
        Html(generate_page("Error", "<h1>Home page template not found</h1>"))
    }
}

// TODO: Re-add contact page - Contact page GET handler (serves under-construction template)
// Contact page handler
async fn contact_page_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/contact/") {
        let html_content = generate_page(&template.title, &template.content);
        Ok(Html(html_content))
    } else {
        let not_found_html = generate_page("404 - Page Not Found",
            "<div style='text-align: center; padding: 50px;'>
                <h1>404 - Page Not Found</h1>
                <p>The contact page template was not found.</p>
                <a href='/'>Return to Home</a>
             </div>");

        Err(axum::response::Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(not_found_html.into())
            .unwrap())
    }
}

// Bio page handler
async fn bio_page_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/bio/") {
        let html_content = generate_page(&template.title, &template.content);
        Ok(Html(html_content))
    } else {
        let not_found_html = generate_page("404 - Page Not Found",
            "<div style='text-align: center; padding: 50px;'>
                <h1>404 - Page Not Found</h1>
                <p>The bio page template was not found.</p>
                <a href='/'>Return to Home</a>
             </div>");

        Err(axum::response::Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(not_found_html.into())
            .unwrap())
    }
}

// Music page handler
async fn music_page_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/music/") {
        let video_ids = read_youtube_links("music");
        let embeds_html = generate_youtube_embeds(&video_ids);
        let content = template.content.replace("{{YOUTUBE_EMBEDS}}", &embeds_html);
        let html_content = generate_page(&template.title, &content);
        Ok(Html(html_content))
    } else {
        let not_found_html = generate_page("404 - Page Not Found",
            "<div style='text-align: center; padding: 50px;'>
                <h1>404 - Page Not Found</h1>
                <p>The music page template was not found.</p>
                <a href='/'>Return to Home</a>
             </div>");

        Err(axum::response::Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(not_found_html.into())
            .unwrap())
    }
}

// Acting page handler
async fn acting_page_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/acting/") {
        let video_ids = read_youtube_links("acting");
        let embeds_html = generate_youtube_embeds(&video_ids);
        let content = template.content.replace("{{ACTING_YOUTUBE_EMBEDS}}", &embeds_html);
        let html_content = generate_page(&template.title, &content);
        Ok(Html(html_content))
    } else {
        let not_found_html = generate_page("404 - Page Not Found",
            "<div style='text-align: center; padding: 50px;'>
                <h1>404 - Page Not Found</h1>
                <p>The acting page template was not found.</p>
                <a href='/'>Return to Home</a>
             </div>");

        Err(axum::response::Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(not_found_html.into())
            .unwrap())
    }
}

// Reviews page handler
async fn reviews_page_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/reviews/") {
        let testimonials = read_testimonials();
        let testimonials_html = generate_testimonials_html(&testimonials);
        let content = template.content.replace("{{TESTIMONIALS_HTML}}", &testimonials_html);
        let html_content = generate_page(&template.title, &content);
        Ok(Html(html_content))
    } else {
        let not_found_html = generate_page("404 - Page Not Found",
            "<div style='text-align: center; padding: 50px;'>
                <h1>404 - Page Not Found</h1>
                <p>The reviews page template was not found.</p>
                <a href='/'>Return to Home</a>
             </div>");

        Err(axum::response::Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(not_found_html.into())
            .unwrap())
    }
}

// Behind-the-scenes page handler
async fn bts_page_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/behind-the-scenes/") {
        let images_dir = Path::new("templates").join("Behind the scenes").join("images");
        let mut images = Vec::new();

        if images_dir.exists() {
            if let Ok(entries) = fs::read_dir(&images_dir) {
                for entry in entries.flatten() {
                    if let Some(extension) = entry.path().extension() {
                        if matches!(extension.to_str(), Some("png") | Some("jpg") | Some("jpeg")) {
                            if let Some(filename) = entry.file_name().to_str() {
                                let url_encoded_filename = url_encode(filename);
                                images.push(format!("/templates/Behind the scenes/images/{}", url_encoded_filename));
                            }
                        }
                    }
                }
            }
        }
        images.sort();

        let images_json: Vec<String> = images.iter().map(|img| format!("\"{}\"", img)).collect();
        let images_json_str = format!("[{}]", images_json.join(", "));

        let subtitle_file = Path::new("templates").join("Behind the scenes").join("subtitle.txt");
        let subtitle = if subtitle_file.exists() {
            fs::read_to_string(&subtitle_file)
                .unwrap_or_else(|_| "Behind the scenes photography".to_string())
                .trim()
                .to_string()
        } else {
            "Behind the scenes photography".to_string()
        };

        let content = template.content
            .replace("{{BTS_IMAGES_JSON}}", &images_json_str)
            .replace("{{BTS_SUBTITLE}}", &subtitle);

        let html_content = generate_page(&template.title, &content);
        Ok(Html(html_content))
    } else {
        let not_found_html = generate_page("404 - Page Not Found",
            "<div style='text-align: center; padding: 50px;'>
                <h1>404 - Page Not Found</h1>
                <p>The behind-the-scenes page template was not found.</p>
                <a href='/'>Return to Home</a>
             </div>");

        Err(axum::response::Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(not_found_html.into())
            .unwrap())
    }
}

// Unified modeling page handler
async fn unified_modeling_handler(templates: axum::extract::State<HashMap<String, PageTemplate>>) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/modeling/") {
        let categories = discover_modeling_categories();
        let html_content = generate_modeling_page(&template.content, &categories);
        Ok(Html(html_content))
    } else {
        let not_found_html = generate_page("404 - Page Not Found",
            "<div style='text-align: center; padding: 50px;'>
                <h1>404 - Page Not Found</h1>
                <p>The modeling page template was not found.</p>
                <a href='/'>Return to Home</a>
             </div>");

        Err(axum::response::Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(not_found_html.into())
            .unwrap())
    }
}

// TODO: Re-add contact page - Contact form POST handler
// Original form submission handler commented out below:
/*
async fn contact_form_handler(Form(form): Form<ContactForm>) -> Html<String> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let email = form.email.as_deref().unwrap_or("Anonymous");
    let message_entry = format!(
        "\n=== Message received at {} ===\nFrom: {} <{}>\nSubject: {}\nMessage:\n{}\n\n",
        timestamp, form.name, email, form.subject, form.message
    );

    let messages_file = "messages.txt";
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(messages_file)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(message_entry.as_bytes()) {
                println!("Error writing to messages file: {}", e);
            } else {
                println!("New message saved from: {} - Subject: {}", form.name, form.subject);
            }
        },
        Err(e) => {
            println!("Error opening messages file: {}", e);
        }
    }

    Html(format!(
        r#"
        <div style="text-align: center; padding: 50px; background: linear-gradient(45deg, #4CAF50, #45a049); color: white; border-radius: 15px; margin: 20px;">
            <h1>Message Sent Successfully!</h1>
            <p>Thank you {}, I'll get back to you soon!</p>
            <a href="/contact/" style="color: white; text-decoration: underline;">Send another message</a>
        </div>
        "#, form.name
    ))
}
*/


#[tokio::main]
async fn main() {
    let templates = match discover_templates() {
        Ok(templates) => templates,
        Err(e) => {
            eprintln!("Error discovering templates: {}", e);
            std::process::exit(1);
        }
    };

    println!("Discovered templates:");
    let mut sorted_paths: Vec<_> = templates.keys().collect();
    sorted_paths.sort();
    for path in &sorted_paths {
        let template = &templates[*path];
        println!("  - {} - {}", path, template.title);
    }

    // Discover modeling categories
    let categories = discover_modeling_categories();
    println!("\nModeling categories:");
    for (name, data) in &categories {
        println!("  - {} ({} images)", name, data.images.len());
    }

    let app = Router::new()
        .route("/", get(home_page_handler))
        .route("/bio/", get(bio_page_handler))
        .route("/acting/", get(acting_page_handler))
        .route("/music/", get(music_page_handler))
        .route("/modeling/", get(unified_modeling_handler))
        .route("/reviews/", get(reviews_page_handler))
        .route("/behind-the-scenes/", get(bts_page_handler))
        // TODO: Re-add contact page - POST handler removed temporarily
        .route("/contact/", get(contact_page_handler)) // .post(contact_form_handler) removed
        .nest_service("/docs", ServeDir::new("docs"))
        .nest_service("/templates", ServeDir::new("templates"))
        .with_state(templates.clone())
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("\nServer running on http://127.0.0.1:3000");
    println!("Available pages:");
    println!("  - http://127.0.0.1:3000");
    println!("  - http://127.0.0.1:3000/bio");
    println!("  - http://127.0.0.1:3000/acting");
    println!("  - http://127.0.0.1:3000/music");
    println!("  - http://127.0.0.1:3000/modeling");
    println!("  - http://127.0.0.1:3000/reviews");
    println!("  - http://127.0.0.1:3000/behind-the-scenes");
    println!("  - http://127.0.0.1:3000/contact");

    axum::serve(listener, app).await.unwrap();
}
