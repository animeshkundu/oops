# Website launch and GitHub Pages setup

**Date**: 2026-01-24  
**Time**: 22:16 UTC  
**Agent**: Copilot + Claude  
**Context**: Build a dedicated GitHub Pages landing site with SEO, onboarding, and documentation for oops.

## Summary

Created a full static marketing site under `site/` with a landing page, docs hub,
installation one-liners, migration guidance, and rule coverage. Added GitHub
Pages deployment workflow, SEO metadata, sitemap/robots, and new docs guides to
connect the site with repo documentation.

## Key Decisions

- **Static HTML/CSS**: Chosen to keep GitHub Pages deployment simple (no build step).
- **One-line installers**: Leveraged existing install scripts and updated them to use the correct repo owner.
- **Docs separation**: Main landing page stays light; deep details live in `/docs/guides/` and `site/docs/`.

## Technical Details

### Changes Made
- Added `site/` with landing page, docs pages, shared styles, assets, manifest, sitemap, and robots.
- Added `.github/workflows/pages.yml` to publish the `site/` folder.
- Updated install scripts and documentation links to point at `animeshkundu/oops`.
- Added `docs/guides/website.md` and `docs/guides/rules.md` for documentation coverage.

### Challenges Faced
- Needed a social preview image without external assets. Generated a lightweight PNG programmatically.

### Edge Cases Discovered
- GitHub Pages uses `main` or `master` branches; workflow triggers on both.
- Added `.nojekyll` to ensure static assets load correctly without Jekyll processing.

## Testing

- Ran `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings` prior to changes.
- Manual review of HTML structure and links.

## Future Considerations

- Add richer OG image assets if a brand kit is created.
- Consider a script to auto-sync markdown docs into the site if desired.

## References

- Web searches: GitHub Pages SEO, Open Graph meta tags, CLI landing page patterns.
- Related docs: `/docs/guides/installation.md`, `/docs/guides/migration-from-thefuck.md`.

## Handoff Context for Next Session

The site is fully static in `site/` and deployed via `.github/workflows/pages.yml`.
Update the HTML pages directly for content edits; no build pipeline exists.
