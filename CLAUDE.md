# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based web server project for Amber Techel's Arts, Talents, & Entertainment LLC that serves a portfolio website featuring modeling, music, acting, bio, and reviews pages. The project supports both development server mode and static site generation for GitHub Pages deployment.

## Technology Stack

- **Language**: Rust (2024 edition)
- **Web Framework**: Axum 0.7
- **Async Runtime**: Tokio 1.0
- **HTTP Utilities**: Tower 0.4, Tower-HTTP 0.5
- **Template System**: Custom HTML template replacement system

## Common Commands

### Development
- `cargo run --bin server` - Start the development server (runs on http://127.0.0.1:3000)
- `cargo run --bin generate-static` - Generate static HTML files for GitHub Pages
- `cargo check` - Check code for compilation errors without building
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version

### Code Quality
- `cargo test` - Run all tests
- `cargo clippy` - Run the Rust linter
- `cargo fmt` - Format code according to Rust standards

## Project Structure

- `src/main.rs` - Development server with dynamic route handlers
- `src/generate_static.rs` - Static site generator for GitHub Pages
- `templates/` - HTML template files with placeholder substitution
  - `base.html` - Base template with `{{TITLE}}`, `{{CONTENT}}`, navigation, and footer
  - `index.html` - Home page content template
  - `styles.css` - Shared CSS styles for all pages
  - `global-images/` - Images used across multiple pages (including `homebackground.png`)
  - `bio/bio.html` - Bio page with timeline layout
  - `bio/background/bkgrnd.png` - Bio page background image
  - `music/music.html` - Music page with album and track listing
  - `music/background/bkgrnd.png` - Music page background image
  - `music/youtubeLinks.txt` - YouTube video URLs to embed (one per line)
  - `acting/acting.html` - Acting page with resumes and videos
  - `acting/Background/bckgrnd.png` - Acting page background image
  - `acting/youtubeLinks.txt` - YouTube video URLs for acting page
  - `reviews/reviews.html` - Reviews/testimonials page
  - `contact/contact.html` - Contact page with form
  - `modeling/modeling.html` - Unified modeling page with category dropdown
  - `Behind the scenes/behind-the-scenes.html` - Behind the scenes gallery page
  - `Behind the scenes/images/` - Behind the scenes photos
  - `Behind the scenes/Background/bkgrnd.png` - Behind the scenes page background image
  - `Behind the scenes/subtitle.txt` - Custom subtitle text for behind the scenes page
  - `modeling/{category}/` - Category-specific folders containing:
    - `images/` - Portfolio images for that category
    - `images/Links.txt` - Optional file mapping image names to URLs (format: `imagename,https://...`)
    - `subtitle.txt` - Optional custom subtitle text
    - `Background/bkgrnd.png` - Optional category-specific background image
- `docs/` - Generated static HTML files and assets for GitHub Pages
- `.github/workflows/deploy.yml` - GitHub Actions workflow for automated deployment

## Architecture

### Template System
The project uses a custom template replacement system:
- `base.html` provides the layout with `{{TITLE}}` and `{{CONTENT}}` placeholders
- `base.html` includes the site footer (appears on all pages automatically)
- `base.html` links to `styles.css` for consistent styling across all pages
- Page-specific templates are embedded into the base template
- Modeling page uses `{{CATEGORIES_JSON}}` placeholder for all category data

### Background Images
Each page can have a custom background image applied to the `.main-content` area:
- **Home page**: Uses `templates/global-images/homebackground.png`
- **Bio page**: Uses `templates/bio/background/bkgrnd.png`
- **Music page**: Uses `templates/music/background/bkgrnd.png`
- **Acting page**: Uses `templates/acting/Background/bckgrnd.png`
- **Modeling page**: Dynamic backgrounds per category from `templates/modeling/{category}/Background/bkgrnd.png`
- **Behind the Scenes page**: Uses `templates/Behind the scenes/Background/bkgrnd.png`

Background styles are added via `<style>` blocks in each template targeting `.main-content` with:
- `background-size: cover`
- `background-position: center`
- `background-repeat: no-repeat`
- `background-attachment: fixed`

### Unified Modeling Page
- Single `/modeling/` route serves all categories
- Intro section with Amber's modeling background
- Embedded PDF resume from Google Drive
- Category dropdown selector to switch between headshots, groupshots, etc.
- All category data (images, subtitles, links, backgrounds) embedded as JSON in the page
- Background image changes dynamically when switching categories
- Images are clickable:
  - If `Links.txt` contains a mapping for the image, opens that URL
  - Otherwise, opens the image in a new tab

### Music Page Features
- Intro section with musical background
- Embedded PDF musical resume from Google Drive
- Instruments grid and genres cloud
- Album feature with streaming links
- Track listing with YouTube links
- Dynamically loaded YouTube video embeds from `youtubeLinks.txt`

### Acting Page Features
- Intro section about acting background (started at age 3, film industry since 2013)
- Side-by-side Acting Resume and Theater Resume PDFs from Google Drive
- Quote/philosophy section
- IMDB Pro profile link
- Current & Recent Projects cards (Finesse, Horse Camp 3, etc.)
- Featured Work section with embedded "Best Interest" video
- Dynamically loaded YouTube video embeds from `acting/youtubeLinks.txt`
- Uses `{{ACTING_YOUTUBE_EMBEDS}}` placeholder

### Reviews Page Features
- Testimonials section with card grid layout
- Each testimonial card includes quote text, author name, and title/company
- Green-themed styling to differentiate from other pages
- Responsive grid that stacks on mobile

### Behind the Scenes Page Features
- Gallery layout displaying behind-the-scenes photos from film & shoots
- Custom subtitle text from `subtitle.txt`
- Responsive grid gallery with hover effects
- Images open in new tab when clicked
- Optional background image support

### Embedded Content
- **YouTube Videos**: Add URLs to `templates/music/youtubeLinks.txt` or `templates/acting/youtubeLinks.txt` (one per line, supports `youtu.be/ID` and `youtube.com/watch?v=ID` formats)
- **Google Drive PDFs**: Use embed format `https://drive.google.com/file/d/{FILE_ID}/preview` in iframe
- Music page uses `{{YOUTUBE_EMBEDS}}` placeholder that gets replaced with generated iframe HTML
- Acting page uses `{{ACTING_YOUTUBE_EMBEDS}}` placeholder for its video section

### Links.txt Format
Place in `templates/modeling/{category}/images/Links.txt`:
```
imagename,https://example.com/product-link
another-image,https://example.com/another-link
```
- Image name should match filename without extension
- One mapping per line, comma-separated

### Development Server (src/main.rs)
- Multi-route Axum application with separate handlers per page
- Template compilation happens at runtime using `include_str!` macros
- Static file serving from `templates/` directory
- **No-cache headers**: Browser caching disabled for instant updates during development
- Routes:
  - `/` - Home page
  - `/bio/` - Bio page with career timeline
  - `/acting/` - Acting page with resumes, projects, and videos
  - `/music/` - Music page with album info and track listing
  - `/modeling/` - Unified modeling portfolio with category dropdown
  - `/reviews/` - Reviews/testimonials page
  - `/behind-the-scenes/` - Behind the scenes gallery page
  - `/contact/` - Contact page (GET shows form, POST handles submission)

### Static Site Generator (src/generate_static.rs)
- Compiles templates into static HTML files in `docs/` directory
- Automatically scans for images in modeling category folders and copies them to `docs/`
- Copies background images from `Background/` folders to `docs/`
- Reads `Links.txt` files and embeds link data in generated pages
- Updates background image paths for GitHub Pages deployment
- **Cache busting**: Appends git commit hash to CSS links (`styles.css?v=abc1234`)
- Updates all navigation links and asset paths for GitHub Pages deployment

### Caching Strategy
- **Development**: No-cache headers ensure changes appear immediately
- **Production**: Git commit hash appended to CSS URL forces browser refresh on new deployments

### GitHub Pages Deployment
- Automated deployment via GitHub Actions on push to main
- Uses Rust toolchain with dependency caching
- Builds static files and deploys to GitHub Pages
- CSS version changes automatically on each commit (cache busting)

## Development Workflow

1. **Adding New Pages**: Add route handler in `main.rs` and corresponding entry in `generate_static.rs`, update navigation in `base.html`
2. **Template Updates**: Modify templates in `templates/` directory - changes affect both dev server and static generation
3. **Adding Background Images**:
   - For regular pages: Add `background/bkgrnd.png` in the page's template folder and add `<style>` block targeting `.main-content`
   - For modeling categories: Add `Background/bkgrnd.png` in the category folder (auto-discovered)
4. **Adding Modeling Images**: Place images in `templates/modeling/{category}/images/` - they'll be auto-discovered and copied to `docs/` by static generator
5. **Adding Image Links**: Create/edit `templates/modeling/{category}/images/Links.txt` with `imagename,url` mappings
6. **Adding YouTube Videos**: Add URLs to `templates/music/youtubeLinks.txt` or `templates/acting/youtubeLinks.txt` (one per line) - auto-embedded on respective pages
7. **Adding PDF Embeds**: Use Google Drive preview URL format in iframe: `https://drive.google.com/file/d/{FILE_ID}/preview`
8. **Testing Changes**: Use development server for rapid iteration (no need to hard-refresh), then generate static files to test final output
9. **Footer/Navigation Changes**: Edit `templates/base.html` - changes apply to all pages

## Deployment Setup

To configure GitHub Pages:
1. Go to your repository Settings → Pages
2. Set source to "GitHub Actions"
3. Push to main branch to trigger automatic deployment
