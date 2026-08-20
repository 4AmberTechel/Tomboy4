const MAX_IMAGE_DIMENSION: u32 = 2048;
const WEBP_QUALITY: f32 = 82.0;
use ab_glyph::{Font, FontArc, PxScale};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use website_test::shared::{
    generate_testimonials_html, generate_youtube_embeds, inject_seo_head, is_image_extension,
    read_links_file, read_products, read_testimonials, read_youtube_links, url_encode,
};

const BASE_URL: &str = "https://4ambertechel.com";

fn get_git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "dev".to_string())
}

fn apply_github_pages_nav_links(html: String) -> String {
    html.replace(
        r#"<a href="/" class="nav-item">Home</a>"#,
        r#"<a href="" class="nav-item">Home</a>"#,
    )
    .replace(
        r#"<a href="/bio/" class="nav-item">Bio</a>"#,
        r#"<a href="/bio" class="nav-item">Bio</a>"#,
    )
    .replace(
        r#"<a href="/acting/" class="nav-item">Acting</a>"#,
        r#"<a href="/acting" class="nav-item">Acting</a>"#,
    )
    .replace(
        r#"<a href="/music/" class="nav-item">Music</a>"#,
        r#"<a href="/music" class="nav-item">Music</a>"#,
    )
    .replace(
        r#"<a href="/modeling/" class="nav-item">Modeling</a>"#,
        r#"<a href="/modeling" class="nav-item">Modeling</a>"#,
    )
    .replace(
        r#"<a href="/reviews/" class="nav-item">Reviews</a>"#,
        r#"<a href="/reviews" class="nav-item">Reviews</a>"#,
    )
    .replace(
        r#"<a href="/behind-the-scenes/" class="nav-item">Behind the Scenes</a>"#,
        r#"<a href="/behind-the-scenes" class="nav-item">Behind the Scenes</a>"#,
    )
    .replace(
        r#"<a href="/dance/" class="nav-item">Dance</a>"#,
        r#"<a href="/dance" class="nav-item">Dance</a>"#,
    )
    .replace(
        r#"<a href="/arts/" class="nav-item">Arts</a>"#,
        r#"<a href="/arts" class="nav-item">Arts</a>"#,
    )
    .replace(
        r#"<a href="/contact/" class="nav-item">Contact</a>"#,
        r#"<a href="/contact" class="nav-item">Contact</a>"#,
    )
}

fn generate_page(title: &str, content: &str, version: &str, page_url: &str) -> String {
    let base_template = include_str!("../templates/base.html");
    let mut final_html = base_template
        .replace("{{TITLE}}", title)
        .replace("{{CONTENT}}", content);

    // Update navigation links for GitHub Pages (static generation)
    final_html = apply_github_pages_nav_links(final_html);

    // Update image paths for GitHub Pages deployment
    final_html = final_html.replace(
        r#"src="/templates/global-images/"#,
        r#"src="/global-images/"#,
    );

    // Update background image paths for GitHub Pages deployment
    final_html = final_html.replace(
        r#"url('/templates/global-images/"#,
        r#"url('/global-images/"#,
    );

    // Update global image extensions to webp
    final_html = replace_extensions_after_prefix(&final_html, "/global-images/");

    // Update CSS path for GitHub Pages deployment with cache busting
    final_html = final_html.replace(
        r#"href="/templates/styles.css""#,
        &format!(r#"href="/styles.css?v={}""#, version),
    );

    inject_seo_head(&final_html, title, page_url, BASE_URL)
}

/// After a path prefix like `/global-images/` is found, replace the first
/// image extension (.png/.jpg/.jpeg) in the following filename with .webp.
fn replace_extensions_after_prefix(html: &str, prefix: &str) -> String {
    let mut result = String::with_capacity(html.len() + 64);
    let mut remaining = html;

    while let Some(pos) = remaining.find(prefix) {
        result.push_str(&remaining[..pos + prefix.len()]);
        remaining = &remaining[pos + prefix.len()..];

        let mut replaced = false;
        for old_ext in &[".png", ".PNG", ".jpg", ".JPG", ".jpeg", ".JPEG"] {
            if let Some(ext_pos) = remaining.find(old_ext) {
                let after = remaining.get(ext_pos + old_ext.len()..ext_pos + old_ext.len() + 1);
                let is_boundary =
                    after.is_none_or(|c| !c.chars().next().is_some_and(|ch| ch.is_alphanumeric()));
                if is_boundary {
                    result.push_str(&remaining[..ext_pos]);
                    result.push_str(".webp");
                    remaining = &remaining[ext_pos + old_ext.len()..];
                    replaced = true;
                    break;
                }
            }
        }
        let _ = replaced;
    }

    result.push_str(remaining);
    result
}

fn get_image_list_for_web(images_dir: &Path, category: &str) -> Vec<String> {
    let mut images = Vec::new();

    if images_dir.exists()
        && let Ok(entries) = fs::read_dir(images_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str())
                && is_image_extension(ext)
                && let Some(stem) = path.file_stem()
            {
                let webp_name = format!("{}.webp", stem.to_string_lossy());
                let url_encoded = url_encode(&webp_name);
                images.push(format!("./{}/images/{}", category, url_encoded));
            }
        }
    }

    images.sort();
    images
}

fn exif_orientation(path: &Path) -> u16 {
    use std::io::Read;
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return 1,
    };
    let mut bytes = Vec::new();
    if file.take(131072).read_to_end(&mut bytes).is_err() {
        return 1;
    }
    let n = bytes.len();
    if n < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return 1;
    }
    let mut i = 2;
    while i + 4 <= n {
        if bytes[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = bytes[i + 1];
        if marker == 0xFF {
            i += 1;
            continue;
        }
        if marker == 0xD9 || marker == 0xDA {
            break;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            i += 2;
            continue;
        }
        let len = ((bytes[i + 2] as usize) << 8) | bytes[i + 3] as usize;
        if len < 2 || i + 2 + len > n {
            break;
        }
        if marker == 0xE1 {
            let seg = &bytes[i + 4..i + 2 + len];
            if seg.len() > 6
                && &seg[0..6] == b"Exif\0\0"
                && let Some(o) = tiff_orientation(&seg[6..])
            {
                return o;
            }
        }
        i += 2 + len;
    }
    1
}

fn tiff_orientation(t: &[u8]) -> Option<u16> {
    if t.len() < 8 {
        return None;
    }
    let le = match &t[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let read_u16 = |b: &[u8]| -> u16 {
        if le {
            (b[0] as u16) | ((b[1] as u16) << 8)
        } else {
            ((b[0] as u16) << 8) | (b[1] as u16)
        }
    };
    let read_u32 = |b: &[u8]| -> u32 {
        if le {
            (b[0] as u32) | ((b[1] as u32) << 8) | ((b[2] as u32) << 16) | ((b[3] as u32) << 24)
        } else {
            ((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | (b[3] as u32)
        }
    };
    let ifd = read_u32(&t[4..8]) as usize;
    if ifd + 2 > t.len() {
        return None;
    }
    let count = read_u16(&t[ifd..ifd + 2]) as usize;
    let mut e = ifd + 2;
    for _ in 0..count {
        if e + 12 > t.len() {
            break;
        }
        if read_u16(&t[e..e + 2]) == 0x0112 {
            let value = read_u16(&t[e + 8..e + 10]);
            if (1..=8).contains(&value) {
                return Some(value);
            }
            return None;
        }
        e += 12;
    }
    None
}

fn apply_orientation(img: image::DynamicImage, orientation: u16) -> image::DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

fn convert_and_copy_images(source_dir: &Path, dest_dir: &Path) {
    if !source_dir.exists() {
        return;
    }

    if fs::create_dir_all(dest_dir).is_err() {
        println!("Failed to create directory: {:?}", dest_dir);
        return;
    }

    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str())
                && is_image_extension(ext)
                && let Some(stem) = path.file_stem()
            {
                let webp_name = format!("{}.webp", stem.to_string_lossy());
                let dest_file = dest_dir.join(&webp_name);

                let orientation = exif_orientation(&path);

                if dest_file.exists() && orientation == 1 {
                    println!("Skipping (cached): {}", webp_name);
                    continue;
                }

                let open_result = image::ImageReader::open(&path)
                    .ok()
                    .and_then(|r| r.with_guessed_format().ok())
                    .and_then(|r| r.decode().ok());

                match open_result {
                    Some(img) => {
                        let img = apply_orientation(img, orientation);
                        let img = if img.width() > MAX_IMAGE_DIMENSION
                            || img.height() > MAX_IMAGE_DIMENSION
                        {
                            img.resize(
                                MAX_IMAGE_DIMENSION,
                                MAX_IMAGE_DIMENSION,
                                image::imageops::FilterType::Lanczos3,
                            )
                        } else {
                            img
                        };
                        let webp_data = if img.color().has_alpha() {
                            let rgba = img.to_rgba8();
                            let (w, h) = rgba.dimensions();
                            webp::Encoder::from_rgba(rgba.as_raw(), w, h).encode(WEBP_QUALITY)
                        } else {
                            let rgb = img.to_rgb8();
                            let (w, h) = rgb.dimensions();
                            webp::Encoder::from_rgb(rgb.as_raw(), w, h).encode(WEBP_QUALITY)
                        };
                        if let Err(e) = std::fs::write(&dest_file, &*webp_data) {
                            println!("Failed to write {:?}: {}", dest_file, e);
                        } else {
                            println!("Converted to WebP: {}", webp_name);
                        }
                    }
                    None => println!("Failed to open image {:?}", path),
                }
            }
        }
    }
}

struct CategoryData {
    title: String,
    subtitle: String,
    images: Vec<String>,
    links: HashMap<String, String>,
    background: Option<String>,
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

                let docs_images_dir = docs_dir
                    .join("modeling")
                    .join(&category_name)
                    .join("images");
                convert_and_copy_images(&images_dir, &docs_images_dir);

                let images = get_image_list_for_web(&images_dir, &category_name);
                let links = read_links_file(&images_dir);

                // Check for and copy background image
                let background_dir = entry.path().join("Background");
                let has_bg = [
                    "bkgrnd.png",
                    "bkgrnd.jpg",
                    "bkgrnd.jpeg",
                    "bkgrnd.PNG",
                    "bkgrnd.JPG",
                ]
                .iter()
                .any(|f| background_dir.join(f).exists());
                let background = if has_bg {
                    let docs_bg_dir = docs_dir
                        .join("modeling")
                        .join(&category_name)
                        .join("Background");
                    create_dir_if_not_exists(&docs_bg_dir);
                    convert_and_copy_images(&background_dir, &docs_bg_dir);
                    Some(format!("./{}/Background/bkgrnd.webp", category_name))
                } else {
                    None
                };

                categories.push((
                    category_name,
                    CategoryData {
                        title,
                        subtitle,
                        images,
                        links,
                        background,
                    },
                ));
            }
        }
    }

    categories.sort_by(|a, b| a.0.cmp(&b.0));
    categories
}

fn discover_arts_categories(docs_dir: &Path) -> Vec<(String, CategoryData)> {
    let mut categories = Vec::new();
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

                let docs_images_dir = docs_dir.join("arts").join(&category_name).join("images");
                convert_and_copy_images(&images_dir, &docs_images_dir);

                let images = get_image_list_for_web(&images_dir, &category_name);

                let background_dir = entry.path().join("Background");
                let has_bg = [
                    "bkgrnd.png", "bkgrnd.jpg", "bkgrnd.jpeg", "bkgrnd.PNG", "bkgrnd.JPG",
                ]
                .iter()
                .any(|f| background_dir.join(f).exists());
                let background = if has_bg {
                    let docs_bg_dir = docs_dir.join("arts").join(&category_name).join("Background");
                    create_dir_if_not_exists(&docs_bg_dir);
                    convert_and_copy_images(&background_dir, &docs_bg_dir);
                    Some(format!("./{}/Background/bkgrnd.webp", category_name))
                } else {
                    None
                };

                categories.push((
                    category_name,
                    CategoryData {
                        title,
                        subtitle,
                        images,
                        links: HashMap::new(),
                        background,
                    },
                ));
            }
        }
    }

    categories.sort_by(|a, b| a.0.cmp(&b.0));
    categories
}

fn generate_categories_json(categories: &[(String, CategoryData)]) -> String {
    let mut json_parts = Vec::new();

    for (key, data) in categories {
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

fn generate_modeling_page(
    content: &str,
    categories: &[(String, CategoryData)],
    version: &str,
) -> String {
    let categories_json = generate_categories_json(categories);
    let updated_content = content.replace("{{CATEGORIES_JSON}}", &categories_json);

    let base_template = include_str!("../templates/base.html");
    let mut final_html = base_template
        .replace("{{TITLE}}", "Modeling Portfolio | Amber Techel — Headshots & Editorial")
        .replace("{{CONTENT}}", &updated_content);

    // Update navigation links for GitHub Pages (modeling page)
    final_html = apply_github_pages_nav_links(final_html);

    // Update CSS path for GitHub Pages deployment with cache busting
    final_html = final_html.replace(
        r#"href="/templates/styles.css""#,
        &format!(r#"href="/styles.css?v={}""#, version),
    );

    inject_seo_head(&final_html, "Modeling Portfolio | Amber Techel — Headshots & Editorial", "/modeling/", BASE_URL)
}

fn generate_sitemap(output_dir: &Path) {
    let sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://4ambertechel.com/</loc><priority>1.0</priority></url>
  <url><loc>https://4ambertechel.com/bio/</loc></url>
  <url><loc>https://4ambertechel.com/acting/</loc></url>
  <url><loc>https://4ambertechel.com/music/</loc></url>
  <url><loc>https://4ambertechel.com/modeling/</loc></url>
  <url><loc>https://4ambertechel.com/reviews/</loc></url>
  <url><loc>https://4ambertechel.com/contact/</loc></url>
  <url><loc>https://4ambertechel.com/order/</loc></url>
  <url><loc>https://4ambertechel.com/behind-the-scenes/</loc></url>
</urlset>"#;

    let path = output_dir.join("sitemap.xml");
    fs::write(&path, sitemap).expect("Failed to write sitemap.xml");
    println!("Generated sitemap.xml");
}

fn generate_robots_txt(output_dir: &Path) {
    let robots = "User-agent: *\nAllow: /\nSitemap: https://4ambertechel.com/sitemap.xml\n";
    let path = output_dir.join("robots.txt");
    fs::write(&path, robots).expect("Failed to write robots.txt");
    println!("Generated robots.txt");
}

fn generate_404(output_dir: &Path, version: &str) {
    let products = read_products();
    let products_json = serde_json::to_string(&products).unwrap_or_else(|_| "[]".to_string());

    let product_links = products
        .iter()
        .map(|p| {
            format!(
                r#"<a href="/order/{}/" class="product-link">{}</a>"#,
                p.code, p.name
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    let content = format!(
        r#"<div class="page-section">
    <h1 class="page-title" id="nf-title">Page Not Found</h1>
    <p class="page-subtitle" id="nf-subtitle">Sorry, the page you're looking for doesn't exist.</p>

    <div class="nf-box" id="nf-order-box" style="display:none">
        <p id="nf-product-message"></p>
        <h3>Available products:</h3>
        <div class="product-links">{product_links}</div>
    </div>

    <div class="nf-box" id="nf-generic-box">
        <p>Try one of the pages below:</p>
        <div class="product-links">
            <a href="/" class="product-link">Home</a>
            <a href="/bio/" class="product-link">Bio</a>
            <a href="/acting/" class="product-link">Acting</a>
            <a href="/music/" class="product-link">Music</a>
            <a href="/modeling/" class="product-link">Modeling</a>
            <a href="/reviews/" class="product-link">Reviews</a>
            <a href="/contact/" class="product-link">Contact</a>
        </div>
    </div>
</div>

<style>
    .nf-box {{
        background: rgba(255, 255, 255, 0.95);
        border-radius: 20px;
        padding: 2.5rem;
        box-shadow: 0 15px 35px rgba(0, 0, 0, 0.1);
        max-width: 700px;
        margin: 2rem auto;
        text-align: center;
    }}

    .nf-box h3 {{
        color: #6b73ff;
        margin-bottom: 1rem;
    }}

    .nf-box p {{
        color: #666;
        line-height: 1.8;
        margin-bottom: 1.5rem;
    }}

    .product-links {{
        display: flex;
        flex-wrap: wrap;
        gap: 0.8rem;
        justify-content: center;
    }}

    .product-link {{
        display: inline-block;
        padding: 0.7rem 1.5rem;
        background: linear-gradient(45deg, #ff6b9d, #6b73ff);
        border-radius: 25px;
        color: white;
        text-decoration: none;
        font-weight: bold;
        transition: all 0.3s ease;
    }}

    .product-link:hover {{
        transform: translateY(-2px);
        box-shadow: 0 8px 20px rgba(107, 115, 255, 0.3);
    }}
</style>

<script>
    const PRODUCTS = {products_json};
    const segments = window.location.pathname.split('/').filter(Boolean);
    if (segments.length >= 2 && segments[0] === 'order') {{
        const code = (segments[segments.length - 1] || '').toUpperCase();
        const product = PRODUCTS.find(p => p.code.toUpperCase() === code);
        if (!product) {{
            document.getElementById('nf-title').textContent = 'Product Not Found';
            document.getElementById('nf-subtitle').textContent = 'That product link is invalid.';
            document.getElementById('nf-order-box').style.display = 'block';
            document.getElementById('nf-product-message').textContent =
                "We couldn't find a product matching code '" + code + "'. Check the URL and try again, or pick one of the available products below:";
            document.getElementById('nf-generic-box').style.display = 'none';
        }}
    }}
</script>"#
    );

    let html = generate_page("Page Not Found | Amber Techel", &content, version, "/404/");

    let path = output_dir.join("404.html");
    fs::write(&path, &html).expect("Failed to write 404.html");
    println!("Generated 404.html");
}

fn create_dir_if_not_exists(path: &Path) {
    if !path.exists() {
        fs::create_dir_all(path)
            .unwrap_or_else(|_| panic!("Failed to create directory: {:?}", path));
    }
}

fn center_x(image_width: u32, text: &str, font: &FontArc, scale: PxScale) -> i32 {
    use ab_glyph::ScaleFont;
    let scaled = font.as_scaled(scale);
    let text_width: f32 = text
        .chars()
        .map(|c| scaled.h_advance(scaled.glyph_id(c)))
        .sum();
    ((image_width as f32 - text_width) / 2.0).max(0.0) as i32
}

fn generate_og_image(docs_dir: &Path) {
    let dest_file = docs_dir.join("global-images").join("og-image.webp");

    if dest_file.exists() {
        println!("Skipping (cached): og-image.webp");
        return;
    }

    let bg_path = Path::new("templates")
        .join("global-images")
        .join("homebackground.png");
    if !bg_path.exists() {
        println!("Warning: homebackground.png not found, skipping OG image generation");
        return;
    }

    let img = match image::ImageReader::open(&bg_path)
        .ok()
        .and_then(|r| r.with_guessed_format().ok())
        .and_then(|r| r.decode().ok())
    {
        Some(img) => img,
        None => {
            println!("Failed to open homebackground.png for OG image");
            return;
        }
    };

    let img = apply_orientation(img, exif_orientation(&bg_path));

    // Resize/crop to 1200x630 OG dimensions
    let img = img.resize_to_fill(1200, 630, image::imageops::FilterType::Lanczos3);
    let mut rgba = img.to_rgba8();

    // Semi-transparent dark overlay (alpha blend with black at ~63% opacity)
    for pixel in rgba.pixels_mut() {
        let alpha = 160u32;
        pixel[0] = ((pixel[0] as u32 * (255 - alpha)) / 255) as u8;
        pixel[1] = ((pixel[1] as u32 * (255 - alpha)) / 255) as u8;
        pixel[2] = ((pixel[2] as u32 * (255 - alpha)) / 255) as u8;
    }

    let font_data = include_bytes!("../templates/global-images/og-font-bold.ttf");
    let font = match FontArc::try_from_slice(font_data) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to load OG font: {}", e);
            return;
        }
    };

    let white = image::Rgba([255u8, 255, 255, 255]);
    let accent = image::Rgba([220u8, 200, 255, 255]);

    let title_scale = PxScale::from(72.0);
    let subtitle_scale = PxScale::from(36.0);
    let cta_scale = PxScale::from(28.0);

    let title = "Amber Techel";
    let subtitle = "Actress  \u{00B7}  Singer-Songwriter  \u{00B7}  Model";
    let cta = "View Portfolio \u{2192}";

    let title_x = center_x(1200, title, &font, title_scale);
    let subtitle_x = center_x(1200, subtitle, &font, subtitle_scale);
    let cta_x = center_x(1200, cta, &font, cta_scale);

    imageproc::drawing::draw_text_mut(&mut rgba, white, title_x, 185, title_scale, &font, title);
    imageproc::drawing::draw_text_mut(
        &mut rgba,
        white,
        subtitle_x,
        295,
        subtitle_scale,
        &font,
        subtitle,
    );
    imageproc::drawing::draw_text_mut(&mut rgba, accent, cta_x, 490, cta_scale, &font, cta);

    let (w, h) = rgba.dimensions();
    let webp_data = webp::Encoder::from_rgba(rgba.as_raw(), w, h).encode(WEBP_QUALITY);
    if let Err(e) = std::fs::write(&dest_file, &*webp_data) {
        println!("Failed to write og-image.webp: {}", e);
    } else {
        println!("Generated og-image.webp");
    }
}

fn main() {
    let docs_dir = Path::new("docs");
    let version = get_git_hash();
    println!("Building with version: {}", version);

    // docs/ is preserved between runs; GitHub Actions caches it so WebP images
    // are not reconverted. Only HTML/CSS files are overwritten each run.

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
        convert_and_copy_images(&templates_global_images, &docs_global_images);
    }

    // Generate branded OG image with text overlay
    generate_og_image(docs_dir);

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
    let home_html = generate_page("Amber Techel | Actress, Singer-Songwriter & Model", home_content, &version, "/");
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
        convert_and_copy_images(&bio_bg_src, &bio_bg_dest);
    }

    let bio_path = Path::new("templates").join("bio").join("bio.html");
    if bio_path.exists() {
        match fs::read_to_string(&bio_path) {
            Ok(content) => {
                // Update background image path for GitHub Pages
                let updated_content = content.replace(
                    "url('/templates/bio/background/bkgrnd.png')",
                    "url('./background/bkgrnd.webp')",
                );
                let html = generate_page("Bio | Amber Techel — Actress, Singer-Songwriter & Model", &updated_content, &version, "/bio/");
                let file_path = bio_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write bio/index.html");
                println!("Generated bio/index.html");
            }
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
        convert_and_copy_images(&music_bg_src, &music_bg_dest);
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
                    "url('./background/bkgrnd.webp')",
                );
                let html = generate_page("Music | Amber Techel — Singer-Songwriter & Performer", &updated_content, &version, "/music/");
                let file_path = music_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write music/index.html");
                println!("Generated music/index.html");
            }
            Err(e) => {
                println!("Failed to read music template: {}", e);
            }
        }
    }

    let contact_dir = docs_dir.join("contact");
    create_dir_if_not_exists(&contact_dir);

    let contact_path = Path::new("templates").join("contact").join("contact.html");
    if contact_path.exists() {
        match fs::read_to_string(&contact_path) {
            Ok(content) => {
                let html = generate_page(
                    "Contact | Amber Techel — Bookings & Inquiries",
                    &content,
                    &version,
                    "/contact/",
                );
                let file_path = contact_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write contact/index.html");
                println!("Generated contact/index.html");
            }
            Err(e) => {
                println!("Failed to read contact template: {}", e);
            }
        }
    }

    let order_dir = docs_dir.join("order");
    create_dir_if_not_exists(&order_dir);

    // Copy product images into docs/order/images/
    let src_images = Path::new("products").join("images");
    let dst_images = order_dir.join("images");
    if src_images.exists() {
        create_dir_if_not_exists(&dst_images);
        if let Ok(entries) = fs::read_dir(&src_images) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    let dest = dst_images.join(src.file_name().unwrap_or_default());
                    let _ = fs::copy(&src, &dest);
                }
            }
        }
        println!("Copied product images to order/images");
    }

    let order_path = Path::new("templates").join("order").join("order.html");
    if order_path.exists() {
        match fs::read_to_string(&order_path) {
            Ok(content) => {
                let products = read_products();
                let products_json =
                    serde_json::to_string(&products).unwrap_or_else(|_| "[]".to_string());
                let content = content.replace("{{PRODUCTS_JSON}}", &products_json);
                let html = generate_page(
                    "Order Merch | Amber Techel — Shop Signed Photos, Apparel & Music",
                    &content,
                    &version,
                    "/order/",
                );
                let file_path = order_dir.join("index.html");
                fs::write(&file_path, &html).expect("Failed to write order/index.html");
                println!("Generated order/index.html");

                // Generate per-product order pages for direct URLs (e.g. /order/ZPLB)
                for product in &products {
                    let product_dir = order_dir.join(&product.code);
                    create_dir_if_not_exists(&product_dir);
                    let product_file = product_dir.join("index.html");
                    fs::write(&product_file, &html).expect(&format!(
                        "Failed to write order/{}/index.html",
                        product.code
                    ));
                    println!(
                        "Generated order/{}/index.html ({} auto-selected)",
                        product.code, product.name
                    );
                }
            }
            Err(e) => {
                println!("Failed to read order template: {}", e);
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
        convert_and_copy_images(&acting_bg_src, &acting_bg_dest);
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
                    "url('./Background/bckgrnd.webp')",
                );
                let html = generate_page("Acting | Amber Techel — Film, TV & Theater Since 2013", &updated_content, &version, "/acting/");
                let file_path = acting_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write acting/index.html");
                println!("Generated acting/index.html");
            }
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
                let html = generate_page("Reviews | Amber Techel — Testimonials & Feedback", &content, &version, "/reviews/");
                let file_path = reviews_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write reviews/index.html");
                println!("Generated reviews/index.html");
            }
            Err(e) => {
                println!("Failed to read reviews template: {}", e);
            }
        }
    }

    // Generate behind-the-scenes page
    let bts_dir = docs_dir.join("behind-the-scenes");
    create_dir_if_not_exists(&bts_dir);

    // Copy BTS images
    let bts_images_src = Path::new("templates")
        .join("Behind the scenes")
        .join("images");
    let bts_images_dest = bts_dir.join("images");
    if bts_images_src.exists() {
        create_dir_if_not_exists(&bts_images_dest);
        convert_and_copy_images(&bts_images_src, &bts_images_dest);
    }

    // Copy BTS background image
    let bts_bg_src = Path::new("templates")
        .join("Behind the scenes")
        .join("background");
    let bts_bg_dest = bts_dir.join("background");
    if bts_bg_src.exists() {
        create_dir_if_not_exists(&bts_bg_dest);
        convert_and_copy_images(&bts_bg_src, &bts_bg_dest);
    }

    let bts_path = Path::new("templates")
        .join("Behind the scenes")
        .join("behind-the-scenes.html");
    if bts_path.exists() {
        match fs::read_to_string(&bts_path) {
            Ok(content) => {
                // Read subtitle
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

                // Get images list
                let mut images = Vec::new();
                if bts_images_src.exists()
                    && let Ok(entries) = fs::read_dir(&bts_images_src)
                {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|e| e.to_str())
                            && is_image_extension(ext)
                            && let Some(stem) = path.file_stem()
                        {
                            let webp_name = format!("{}.webp", stem.to_string_lossy());
                            let url_encoded = url_encode(&webp_name);
                            images.push(format!("./images/{}", url_encoded));
                        }
                    }
                }
                images.sort();

                let images_json: Vec<String> =
                    images.iter().map(|img| format!("\"{}\"", img)).collect();
                let images_json_str = format!("[{}]", images_json.join(", "));

                // Update background image path for GitHub Pages
                let updated_content = content
                    .replace("{{BTS_IMAGES_JSON}}", &images_json_str)
                    .replace("{{BTS_SUBTITLE}}", &subtitle)
                    .replace(
                        "url('/templates/Behind the scenes/background/bkgrnd.png')",
                        "url('./background/bkgrnd.webp')",
                    );

                let html = generate_page(
                    "Behind the Scenes | Amber Techel — Film & Shoots",
                    &updated_content,
                    &version,
                    "/behind-the-scenes/",
                );
                let file_path = bts_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write behind-the-scenes/index.html");
                println!("Generated behind-the-scenes/index.html");
            }
            Err(e) => {
                println!("Failed to read behind-the-scenes template: {}", e);
            }
        }
    }

    // Generate dance page (under construction)
    let dance_dir = docs_dir.join("dance");
    create_dir_if_not_exists(&dance_dir);

    let dance_path = Path::new("templates")
        .join("dance")
        .join("dance-under-construction.html");
    if dance_path.exists() {
        match fs::read_to_string(&dance_path) {
            Ok(content) => {
                let html =
                    generate_page(                    "Dance | Amber Techel", &content, &version, "/dance/");
                let file_path = dance_dir.join("index.html");
                fs::write(&file_path, html).expect("Failed to write dance/index.html");
                println!("Generated dance/index.html (under construction)");
            }
            Err(e) => println!("Failed to read dance template: {}", e),
        }
    }

    let arts_dir = docs_dir.join("arts");
    create_dir_if_not_exists(&arts_dir);

    let arts_categories = discover_arts_categories(docs_dir);
    println!("\nArts categories discovered:");
    for (name, data) in &arts_categories {
        println!("  - {} ({} pieces)", name, data.images.len());
    }

    let arts_content = include_str!("../templates/arts/arts.html");
    let arts_categories_json = generate_categories_json(&arts_categories);
    let arts_body = arts_content.replace("{{CATEGORIES_JSON}}", &arts_categories_json);
    let arts_html = generate_page("Art & Jewelry | Amber Techel — Original Works", &arts_body, &version, "/arts/");
    let arts_file_path = arts_dir.join("index.html");
    fs::write(&arts_file_path, arts_html).expect("Failed to write arts/index.html");
    println!("Generated arts/index.html");

    generate_sitemap(docs_dir);
    generate_robots_txt(docs_dir);
    generate_404(docs_dir, &version);

    println!("\nStatic files generated successfully!");
}
