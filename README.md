# Website-test

A Rust-based website for Eternal Cataclysm Studios, deployable to GitHub Pages.

## Development Commands

### Local Development
- `cargo run --bin server` - Start the development server (runs on http://127.0.0.1:3000)
- `cargo run --bin generate-static` - Generate static HTML files for GitHub Pages

### Build Commands
- `cargo check` - Check code for compilation errors without building
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version

### Code Quality
- `cargo test` - Run all tests
- `cargo clippy` - Run the Rust linter
- `cargo fmt` - Format code according to Rust standards

## GitHub Pages Deployment

The site automatically deploys to GitHub Pages via GitHub Actions when you push to the main branch.

**Setup:**
1. Go to repository Settings → Pages
2. Set source to "GitHub Actions"
3. Push to main branch to trigger deployment

Your site will be available at: `https://yourusername.github.io/Website-test`
