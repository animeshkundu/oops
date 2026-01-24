# Website & Documentation Structure

This guide explains how the GitHub Pages site is organized and how it maps to
repo documentation.

## Landing Page

The landing page lives in `site/index.html` and focuses on:

- One-line installs for macOS, Linux, and Windows
- Benchmarks and why oops is faster than thefuck
- Quick onboarding steps
- Clear navigation to documentation

## Documentation Pages

The GitHub Pages documentation hub is in `site/docs/`:

- `index.html` - Documentation landing page
- `installation.html` - One-line installs, package managers, shell setup
- `quickstart.html` - 60-second onboarding flow
- `rules.html` - Rule categories and coverage info
- `migration.html` - thefuck migration guide
- `porting-rules.html` - How to port and create rules
- `configuration.html` - Config and env variables

Each page links back to the full Markdown docs in `/docs/guides/` for exhaustive
reference.

## Assets

Static assets are stored in `site/assets/`:

- `styles.css` - Shared styling for all pages
- `copy.js` - Copy-to-clipboard helper
- `og-image.png` - Social preview image
- `favicon.png` - Site favicon

## Deployment

The GitHub Pages workflow (`.github/workflows/pages.yml`) publishes the `site/`
directory on pushes to `main` or `master`. The site uses a static HTML
structure with no build step.
