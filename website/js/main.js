/**
 * oops Website - Main JavaScript
 */

// DOM Ready
document.addEventListener('DOMContentLoaded', () => {
    initNavigation();
    initInstallTabs();
    initShellTabs();
    initCopyButtons();
    initTerminalDemo();
    initScrollEffects();
});

/**
 * Navigation
 */
function initNavigation() {
    const navbar = document.getElementById('navbar');
    const mobileMenuBtn = document.getElementById('mobileMenuBtn');
    const navLinks = document.querySelector('.nav-links');
    
    // Mobile menu toggle
    if (mobileMenuBtn && navLinks) {
        mobileMenuBtn.addEventListener('click', () => {
            navLinks.classList.toggle('active');
            mobileMenuBtn.classList.toggle('active');
        });
        
        // Close menu on link click
        navLinks.querySelectorAll('a').forEach(link => {
            link.addEventListener('click', () => {
                navLinks.classList.remove('active');
                mobileMenuBtn.classList.remove('active');
            });
        });
    }
    
    // Navbar scroll effect
    let lastScroll = 0;
    window.addEventListener('scroll', () => {
        const currentScroll = window.pageYOffset;
        
        if (currentScroll > 100) {
            navbar.style.background = 'rgba(10, 10, 15, 0.95)';
        } else {
            navbar.style.background = 'rgba(10, 10, 15, 0.85)';
        }
        
        lastScroll = currentScroll;
    });
}

/**
 * Install Tabs
 */
function initInstallTabs() {
    const tabBtns = document.querySelectorAll('.install-tabs .tab-btn');
    const tabPanels = document.querySelectorAll('.install-content .tab-panel');
    
    tabBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            const tabId = btn.dataset.tab;
            
            // Update active states
            tabBtns.forEach(b => b.classList.remove('active'));
            tabPanels.forEach(p => p.classList.remove('active'));
            
            btn.classList.add('active');
            document.getElementById(tabId)?.classList.add('active');
        });
    });
}

/**
 * Shell Integration Tabs
 */
function initShellTabs() {
    const shellTabs = document.querySelectorAll('.shell-tab');
    const shellPanels = document.querySelectorAll('.shell-panel');
    
    shellTabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const shellId = tab.dataset.shell;
            
            // Update active states
            shellTabs.forEach(t => t.classList.remove('active'));
            shellPanels.forEach(p => p.classList.remove('active'));
            
            tab.classList.add('active');
            document.getElementById(`${shellId}-panel`)?.classList.add('active');
        });
    });
}

/**
 * Copy to Clipboard Buttons
 */
function initCopyButtons() {
    const copyBtns = document.querySelectorAll('.copy-btn');
    
    copyBtns.forEach(btn => {
        btn.addEventListener('click', async () => {
            const textToCopy = btn.dataset.copy;
            
            try {
                await navigator.clipboard.writeText(textToCopy);
                
                // Visual feedback
                btn.classList.add('copied');
                btn.innerHTML = `
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="20 6 9 17 4 12"/>
                    </svg>
                `;
                
                // Reset after 2 seconds
                setTimeout(() => {
                    btn.classList.remove('copied');
                    btn.innerHTML = `
                        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                            <rect x="9" y="9" width="13" height="13" rx="2"/>
                            <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
                        </svg>
                    `;
                }, 2000);
            } catch (err) {
                console.error('Failed to copy:', err);
                
                // Fallback for older browsers
                const textArea = document.createElement('textarea');
                textArea.value = textToCopy;
                textArea.style.position = 'fixed';
                textArea.style.left = '-9999px';
                document.body.appendChild(textArea);
                textArea.select();
                
                try {
                    document.execCommand('copy');
                    btn.classList.add('copied');
                } catch (e) {
                    console.error('Fallback copy failed:', e);
                }
                
                document.body.removeChild(textArea);
            }
        });
    });
}

/**
 * Terminal Demo Animation
 */
function initTerminalDemo() {
    const demoCommands = [
        {
            command: 'git psuh',
            error: "git: 'psuh' is not a git command. Did you mean 'push'?",
            fix: 'git push'
        },
        {
            command: 'apt install vim',
            error: 'E: Could not open lock file - Permission denied',
            fix: 'sudo apt install vim'
        },
        {
            command: 'cd /ptojects',
            error: "bash: cd: /ptojects: No such file or directory",
            fix: 'cd /projects'
        },
        {
            command: 'brew isntall node',
            error: "Error: Unknown command: isntall",
            fix: 'brew install node'
        },
        {
            command: 'npm run biuld',
            error: 'npm ERR! Missing script: "biuld"',
            fix: 'npm run build'
        },
        {
            command: 'docker ps -la',
            error: "unknown shorthand flag: 'l' in -la",
            fix: 'docker ps -a'
        }
    ];
    
    let currentIndex = 0;
    const commandEl = document.getElementById('demoCommand');
    const errorEl = document.getElementById('demoError');
    const suggestionEl = document.getElementById('demoSuggestion');
    
    if (!commandEl || !errorEl || !suggestionEl) return;
    
    function updateDemo() {
        const demo = demoCommands[currentIndex];
        
        // Fade out
        commandEl.style.opacity = 0;
        errorEl.style.opacity = 0;
        suggestionEl.style.opacity = 0;
        
        setTimeout(() => {
            commandEl.textContent = demo.command;
            errorEl.querySelector('span').textContent = demo.error;
            suggestionEl.querySelector('.suggestion-cmd').textContent = demo.fix;
            
            // Fade in
            commandEl.style.opacity = 1;
            setTimeout(() => {
                errorEl.style.opacity = 1;
                setTimeout(() => {
                    suggestionEl.style.opacity = 1;
                }, 300);
            }, 300);
        }, 300);
        
        currentIndex = (currentIndex + 1) % demoCommands.length;
    }
    
    // Add transitions
    [commandEl, errorEl, suggestionEl].forEach(el => {
        el.style.transition = 'opacity 0.3s ease';
    });
    
    // Cycle through demos every 4 seconds
    setInterval(updateDemo, 4000);
}

/**
 * Scroll Effects
 */
function initScrollEffects() {
    // Smooth scroll for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', (e) => {
            const href = anchor.getAttribute('href');
            if (href === '#') return;
            
            e.preventDefault();
            const target = document.querySelector(href);
            
            if (target) {
                const navbarHeight = document.getElementById('navbar')?.offsetHeight || 0;
                const targetPosition = target.getBoundingClientRect().top + window.pageYOffset - navbarHeight - 20;
                
                window.scrollTo({
                    top: targetPosition,
                    behavior: 'smooth'
                });
            }
        });
    });
    
    // Animate elements on scroll
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('animate-in');
                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);
    
    // Observe feature cards, rule categories, etc.
    document.querySelectorAll('.feature-card, .rule-category, .install-card, .stat').forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.5s ease, transform 0.5s ease';
        observer.observe(el);
    });
}

// Add animate-in styles
const style = document.createElement('style');
style.textContent = `
    .animate-in {
        opacity: 1 !important;
        transform: translateY(0) !important;
    }
`;
document.head.appendChild(style);

/**
 * Theme Toggle (for future use)
 */
function toggleTheme() {
    document.body.classList.toggle('light-theme');
    localStorage.setItem('theme', document.body.classList.contains('light-theme') ? 'light' : 'dark');
}

// Check for saved theme preference
if (localStorage.getItem('theme') === 'light') {
    document.body.classList.add('light-theme');
}
