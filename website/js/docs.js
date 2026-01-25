/**
 * oops Documentation JavaScript
 */

document.addEventListener('DOMContentLoaded', () => {
    initDocsNavigation();
    initCodeCopy();
    initTableOfContents();
    highlightCurrentPage();
});

/**
 * Docs Navigation
 */
function initDocsNavigation() {
    const sidebar = document.querySelector('.docs-sidebar');
    const mobileMenuBtn = document.getElementById('mobileMenuBtn');
    
    if (mobileMenuBtn && sidebar) {
        mobileMenuBtn.addEventListener('click', () => {
            sidebar.classList.toggle('active');
        });
        
        // Close sidebar when clicking on a link (mobile)
        sidebar.querySelectorAll('a').forEach(link => {
            link.addEventListener('click', () => {
                if (window.innerWidth <= 768) {
                    sidebar.classList.remove('active');
                }
            });
        });
    }
}

/**
 * Code Block Copy Buttons
 */
function initCodeCopy() {
    const codeBlocks = document.querySelectorAll('.docs-article pre');
    
    codeBlocks.forEach(block => {
        const wrapper = document.createElement('div');
        wrapper.className = 'code-block-wrapper';
        wrapper.style.position = 'relative';
        
        block.parentNode.insertBefore(wrapper, block);
        wrapper.appendChild(block);
        
        const copyBtn = document.createElement('button');
        copyBtn.className = 'code-copy-btn';
        copyBtn.innerHTML = `
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2"/>
                <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
            </svg>
        `;
        copyBtn.style.cssText = `
            position: absolute;
            top: 8px;
            right: 8px;
            padding: 6px;
            background: var(--color-surface);
            border: 1px solid var(--color-border);
            border-radius: 4px;
            color: var(--color-text-muted);
            cursor: pointer;
            opacity: 0;
            transition: opacity 0.2s ease;
        `;
        
        wrapper.appendChild(copyBtn);
        
        wrapper.addEventListener('mouseenter', () => {
            copyBtn.style.opacity = '1';
        });
        
        wrapper.addEventListener('mouseleave', () => {
            copyBtn.style.opacity = '0';
        });
        
        copyBtn.addEventListener('click', async () => {
            const code = block.querySelector('code')?.textContent || block.textContent;
            
            try {
                await navigator.clipboard.writeText(code);
                copyBtn.innerHTML = `
                    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="20 6 9 17 4 12"/>
                    </svg>
                `;
                copyBtn.style.color = 'var(--color-success)';
                
                setTimeout(() => {
                    copyBtn.innerHTML = `
                        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                            <rect x="9" y="9" width="13" height="13" rx="2"/>
                            <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                        </svg>
                    `;
                    copyBtn.style.color = 'var(--color-text-muted)';
                }, 2000);
            } catch (err) {
                console.error('Failed to copy:', err);
            }
        });
    });
}

/**
 * Generate Table of Contents
 */
function initTableOfContents() {
    const article = document.querySelector('.docs-article');
    const tocContainer = document.querySelector('.table-of-contents');
    
    if (!article || !tocContainer) return;
    
    const headings = article.querySelectorAll('h2, h3');
    
    if (headings.length === 0) {
        tocContainer.style.display = 'none';
        return;
    }
    
    const toc = document.createElement('ul');
    
    headings.forEach((heading, index) => {
        // Add ID if missing
        if (!heading.id) {
            heading.id = heading.textContent
                .toLowerCase()
                .replace(/[^a-z0-9]+/g, '-')
                .replace(/(^-|-$)/g, '');
        }
        
        const li = document.createElement('li');
        li.className = heading.tagName === 'H3' ? 'toc-h3' : 'toc-h2';
        
        const link = document.createElement('a');
        link.href = `#${heading.id}`;
        link.textContent = heading.textContent;
        
        li.appendChild(link);
        toc.appendChild(li);
    });
    
    tocContainer.appendChild(toc);
    
    // Highlight current section on scroll
    const observer = new IntersectionObserver(
        (entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    const id = entry.target.id;
                    toc.querySelectorAll('a').forEach(link => {
                        link.classList.remove('active');
                        if (link.getAttribute('href') === `#${id}`) {
                            link.classList.add('active');
                        }
                    });
                }
            });
        },
        { rootMargin: '-100px 0px -66%' }
    );
    
    headings.forEach(heading => observer.observe(heading));
}

/**
 * Highlight Current Page in Navigation
 */
function highlightCurrentPage() {
    const currentPath = window.location.pathname;
    const navLinks = document.querySelectorAll('.docs-nav a');
    
    navLinks.forEach(link => {
        const linkPath = link.getAttribute('href');
        if (currentPath.endsWith(linkPath) || 
            (linkPath === 'index.html' && (currentPath.endsWith('/docs/') || currentPath.endsWith('/docs')))) {
            link.classList.add('active');
        }
    });
}

/**
 * Search (for future use)
 */
function initSearch() {
    const searchInput = document.getElementById('docsSearch');
    if (!searchInput) return;
    
    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase();
        // Implement search logic
    });
}
