use axum::{Router, extract::Form, http::header, response::Html, routing::get};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use website_test::shared::{
    generate_testimonials_html, generate_youtube_embeds, inject_seo_head, is_image_extension,
    read_links_file, read_testimonials, read_youtube_links, url_encode,
};

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

#[derive(Deserialize)]
struct ContactForm {
    name: String,
    email: Option<String>,
    subject: String,
    message: String,
}

fn generate_page(title: &str, content: &str, page_url: &str) -> String {
    let base_template = include_str!("../templates/base.html");
    let html = base_template
        .replace("{{TITLE}}", title)
        .replace("{{CONTENT}}", content);
    inject_seo_head(&html, title, page_url, "http://127.0.0.1:3000")
}

fn get_image_list(images_dir: &Path, category: &str) -> Vec<String> {
    let mut images = Vec::new();

    if images_dir.exists()
        && let Ok(entries) = fs::read_dir(images_dir)
    {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension()
                && extension.to_str().is_some_and(is_image_extension)
                && let Some(filename) = entry.file_name().to_str()
            {
                let url_encoded_filename = url_encode(filename);
                images.push(format!(
                    "/templates/modeling/{}/images/{}",
                    category, url_encoded_filename
                ));
            }
        }
    }

    images.sort();
    images
}

fn discover_templates() -> Result<HashMap<String, PageTemplate>, Box<dyn std::error::Error>> {
    let mut templates = HashMap::new();
    let templates_dir = Path::new("templates");

    // Handle index.html as root page
    let index_path = templates_dir.join("index.html");
    if index_path.exists() {
        let content = fs::read_to_string(&index_path)?;
        templates.insert(
            "/".to_string(),
            PageTemplate {
                title: "Amber Techel | Actress, Singer-Songwriter & Model".to_string(),
                content,
            },
        );
    }

    let contact_path = templates_dir.join("contact").join("contact.html");
    if contact_path.exists() {
        let content = fs::read_to_string(&contact_path)?;
        templates.insert(
            "/contact/".to_string(),
            PageTemplate {
                title: "Contact | Amber Techel — Bookings & Inquiries".to_string(),
                content,
            },
        );
    }

    // Handle unified modeling page
    let modeling_path = templates_dir.join("modeling").join("modeling.html");
    if modeling_path.exists() {
        let content = fs::read_to_string(&modeling_path)?;
        templates.insert(
            "/modeling/".to_string(),
            PageTemplate {
                title: "Modeling Portfolio | Amber Techel — Headshots & Editorial".to_string(),
                content,
            },
        );
    }

    // Handle bio page
    let bio_path = templates_dir.join("bio").join("bio.html");
    if bio_path.exists() {
        let content = fs::read_to_string(&bio_path)?;
        templates.insert(
            "/bio/".to_string(),
            PageTemplate {
                title: "Bio | Amber Techel — Actress, Singer-Songwriter & Model".to_string(),
                content,
            },
        );
    }

    // Handle music page
    let music_path = templates_dir.join("music").join("music.html");
    if music_path.exists() {
        let content = fs::read_to_string(&music_path)?;
        templates.insert(
            "/music/".to_string(),
            PageTemplate {
                title: "Music | Amber Techel — Singer-Songwriter & Performer".to_string(),
                content,
            },
        );
    }

    // Handle acting page
    let acting_path = templates_dir.join("acting").join("acting.html");
    if acting_path.exists() {
        let content = fs::read_to_string(&acting_path)?;
        templates.insert(
            "/acting/".to_string(),
            PageTemplate {
                title: "Acting | Amber Techel — Film, TV & Theater Since 2013".to_string(),
                content,
            },
        );
    }

    // Handle reviews page
    let reviews_path = templates_dir.join("reviews").join("reviews.html");
    if reviews_path.exists() {
        let content = fs::read_to_string(&reviews_path)?;
        templates.insert(
            "/reviews/".to_string(),
            PageTemplate {
                title: "Reviews | Amber Techel — Testimonials & Feedback".to_string(),
                content,
            },
        );
    }

    // Handle behind-the-scenes page
    let bts_path = templates_dir
        .join("Behind the scenes")
        .join("behind-the-scenes.html");
    if bts_path.exists() {
        let content = fs::read_to_string(&bts_path)?;
        templates.insert(
            "/behind-the-scenes/".to_string(),
            PageTemplate {
                title: "Behind the Scenes | Amber Techel — Film & Shoots".to_string(),
                content,
            },
        );
    }

    // Dance - under construction
    let dance_path = templates_dir
        .join("dance")
        .join("dance-under-construction.html");
    if dance_path.exists() {
        let content = fs::read_to_string(&dance_path)?;
        templates.insert(
            "/dance/".to_string(),
            PageTemplate {
                title: "Dance | Amber Techel".to_string(),
                content,
            },
        );
    }

    // Arts - under construction
    let arts_path = templates_dir
        .join("arts")
        .join("arts-under-construction.html");
    if arts_path.exists() {
        let content = fs::read_to_string(&arts_path)?;
        templates.insert(
            "/arts/".to_string(),
            PageTemplate {
                title: "Art & Jewelry | Amber Techel — Original Works".to_string(),
                content,
            },
        );
    }

    Ok(templates)
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
                let background = [
                    "bkgrnd.png",
                    "bkgrnd.jpg",
                    "bkgrnd.jpeg",
                    "bkgrnd.PNG",
                    "bkgrnd.JPG",
                ]
                .iter()
                .find(|f| background_dir.join(f).exists())
                .map(|f| format!("/templates/modeling/{}/Background/{}", category_name, f));

                categories.insert(
                    category_name,
                    CategoryData {
                        title,
                        subtitle,
                        images,
                        links,
                        background,
                    },
                );
            }
        }
    }

    categories
}

fn discover_arts_categories() -> HashMap<String, CategoryData> {
    let mut categories = HashMap::new();
    let arts_dir = Path::new("templates").join("arts");

    if !arts_dir.exists() {
        return categories;
    }

    if let Ok(entries) = fs::read_dir(&arts_dir) {
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
                        .unwrap_or_else(|_| format!("{} collection", category_name))
                } else {
                    format!("{} collection", category_name)
                };

                let mut images = Vec::new();
                if let Ok(img_entries) = fs::read_dir(&images_dir) {
                    for img_entry in img_entries.flatten() {
                        if let Some(ext) = img_entry.path().extension()
                            && ext.to_str().is_some_and(is_image_extension)
                            && let Some(filename) = img_entry.file_name().to_str()
                        {
                            let url_encoded = url_encode(filename);
                            images.push(format!(
                                "/templates/arts/{}/images/{}",
                                category_name, url_encoded
                            ));
                        }
                    }
                }
                images.sort();

                let links = HashMap::new();
                let background_dir = entry.path().join("Background");
                let background = [
                    "bkgrnd.png", "bkgrnd.jpg", "bkgrnd.jpeg", "bkgrnd.PNG", "bkgrnd.JPG",
                ]
                .iter()
                .find(|f| background_dir.join(f).exists())
                .map(|f| format!("/templates/arts/{}/Background/{}", category_name, f));

                categories.insert(
                    category_name,
                    CategoryData {
                        title,
                        subtitle,
                        images,
                        links,
                        background,
                    },
                );
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
        let images_json: Vec<String> = data
            .images
            .iter()
            .map(|img| format!("\"{}\"", img))
            .collect();
        let escaped_subtitle = data
            .subtitle
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", " ")
            .replace("\r", "");

        let links_json: Vec<String> = data
            .links
            .iter()
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
    let html = base_template
        .replace("{{TITLE}}", "Modeling Portfolio | Amber Techel — Headshots & Editorial")
        .replace("{{CONTENT}}", &updated_content);
    inject_seo_head(
        &html,
        "Modeling Portfolio | Amber Techel — Headshots & Editorial",
        "/modeling/",
        "http://127.0.0.1:3000",
    )
}

fn not_found_response(page_name: &str) -> axum::response::Response {
    let html = generate_page(
        "404 - Page Not Found",
        &format!(
            "<div style='text-align: center; padding: 50px;'>\
                <h1>404 - Page Not Found</h1>\
                <p>The {} page template was not found.</p>\
                <a href='/'>Return to Home</a>\
             </div>",
            page_name
        ),
        "",
    );
    axum::response::Response::builder()
        .status(404)
        .header("content-type", "text/html")
        .body(html.into())
        .unwrap()
}

// Home page handler
async fn home_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Html<String> {
    if let Some(template) = templates.get("/") {
        Html(generate_page(&template.title, &template.content, "/"))
    } else {
        Html(generate_page(
            "Error",
            "<h1>Home page template not found</h1>",
            "/",
        ))
    }
}

// Contact page handler
async fn contact_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/contact/") {
        let html_content = generate_page(&template.title, &template.content, "/contact/");
        Ok(Html(html_content))
    } else {
        Err(not_found_response("contact"))
    }
}

// Bio page handler
async fn bio_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/bio/") {
        let html_content = generate_page(&template.title, &template.content, "/bio/");
        Ok(Html(html_content))
    } else {
        Err(not_found_response("bio"))
    }
}

// Music page handler
async fn music_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/music/") {
        let video_ids = read_youtube_links("music");
        let embeds_html = generate_youtube_embeds(&video_ids);
        let content = template.content.replace("{{YOUTUBE_EMBEDS}}", &embeds_html);
        let html_content = generate_page(&template.title, &content, "/music/");
        Ok(Html(html_content))
    } else {
        Err(not_found_response("music"))
    }
}

// Acting page handler
async fn acting_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/acting/") {
        let video_ids = read_youtube_links("acting");
        let embeds_html = generate_youtube_embeds(&video_ids);
        let content = template
            .content
            .replace("{{ACTING_YOUTUBE_EMBEDS}}", &embeds_html);
        let html_content = generate_page(&template.title, &content, "/acting/");
        Ok(Html(html_content))
    } else {
        Err(not_found_response("acting"))
    }
}

// Reviews page handler
async fn reviews_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/reviews/") {
        let testimonials = read_testimonials();
        let testimonials_html = generate_testimonials_html(&testimonials);
        let content = template
            .content
            .replace("{{TESTIMONIALS_HTML}}", &testimonials_html);
        let html_content = generate_page(&template.title, &content, "/reviews/");
        Ok(Html(html_content))
    } else {
        Err(not_found_response("reviews"))
    }
}

// Dance page handler
async fn dance_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/dance/") {
        Ok(Html(generate_page(
            &template.title,
            &template.content,
            "/dance/",
        )))
    } else {
        Err(not_found_response("dance"))
    }
}

// Arts page handler
async fn arts_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/arts/") {
        let categories = discover_arts_categories();
        let categories_json = generate_categories_json(&categories);
        let content = template.content.replace("{{CATEGORIES_JSON}}", &categories_json);
        let html_content = generate_page(&template.title, &content, "/arts/");
        Ok(Html(html_content))
    } else {
        Err(not_found_response("arts"))
    }
}

// Behind-the-scenes page handler
async fn bts_page_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/behind-the-scenes/") {
        let images_dir = Path::new("templates")
            .join("Behind the scenes")
            .join("images");
        let mut images = Vec::new();

        if images_dir.exists()
            && let Ok(entries) = fs::read_dir(&images_dir)
        {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension()
                    && extension.to_str().is_some_and(is_image_extension)
                    && let Some(filename) = entry.file_name().to_str()
                {
                    let url_encoded_filename = url_encode(filename);
                    images.push(format!(
                        "/templates/Behind the scenes/images/{}",
                        url_encoded_filename
                    ));
                }
            }
        }
        images.sort();

        let images_json: Vec<String> = images.iter().map(|img| format!("\"{}\"", img)).collect();
        let images_json_str = format!("[{}]", images_json.join(", "));

        let subtitle_file = Path::new("templates")
            .join("Behind the scenes")
            .join("subtitle.txt");
        let subtitle = if subtitle_file.exists() {
            fs::read_to_string(&subtitle_file)
                .unwrap_or_else(|_| "Behind the scenes photography".to_string())
                .trim()
                .to_string()
        } else {
            "Behind the scenes photography".to_string()
        };

        let content = template
            .content
            .replace("{{BTS_IMAGES_JSON}}", &images_json_str)
            .replace("{{BTS_SUBTITLE}}", &subtitle);

        let html_content = generate_page(&template.title, &content, "/behind-the-scenes/");
        Ok(Html(html_content))
    } else {
        Err(not_found_response("behind-the-scenes"))
    }
}

// Unified modeling page handler
async fn unified_modeling_handler(
    templates: axum::extract::State<HashMap<String, PageTemplate>>,
) -> Result<Html<String>, axum::response::Response> {
    if let Some(template) = templates.get("/modeling/") {
        let categories = discover_modeling_categories();
        let html_content = generate_modeling_page(&template.content, &categories);
        Ok(Html(html_content))
    } else {
        Err(not_found_response("modeling"))
    }
}

async fn contact_form_handler(Form(form): Form<ContactForm>) -> Html<String> {
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
    match fs::OpenOptions::new()
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
        }
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

fn get_lan_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}

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
        .route("/dance/", get(dance_page_handler))
        .route("/arts/", get(arts_page_handler))
        .route("/contact/", get(contact_page_handler).post(contact_form_handler))
        .nest_service("/docs", ServeDir::new("docs"))
        .nest_service("/templates", ServeDir::new("templates"))
        .with_state(templates.clone())
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("\nServer running on http://0.0.0.0:3000 (all interfaces)");
    println!("Local:   http://127.0.0.1:3000");
    if let Some(ip) = get_lan_ip() {
        println!("Network: http://{}:3000", ip);
    }
    println!("Available pages:");
    println!(
        "  /  /bio  /acting  /music  /modeling  /reviews  /behind-the-scenes  /dance  /arts  /contact"
    );

    axum::serve(listener, app).await.unwrap();
}
