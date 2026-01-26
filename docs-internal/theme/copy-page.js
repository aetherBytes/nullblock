// Copy Page for AI - Adds a button to copy page content in agent-friendly format

(function() {
    'use strict';

    // Wait for DOM to be ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    function init() {
        // Create the copy button
        const copyButton = document.createElement('button');
        copyButton.id = 'copy-page-btn';
        copyButton.className = 'copy-page-button';
        copyButton.innerHTML = `
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
            <span>Copy for AI</span>
        `;
        copyButton.title = 'Copy page content in AI-friendly format';
        copyButton.addEventListener('click', copyPageContent);

        // Find the right-buttons container in the menu bar
        const rightButtons = document.querySelector('.right-buttons');
        if (rightButtons) {
            rightButtons.insertBefore(copyButton, rightButtons.firstChild);
        } else {
            // Fallback: add to menu bar
            const menuBar = document.querySelector('.menu-bar');
            if (menuBar) {
                menuBar.appendChild(copyButton);
            }
        }
    }

    function copyPageContent() {
        const content = document.querySelector('.content main');
        if (!content) {
            showFeedback('No content found', false);
            return;
        }

        // Get the page title
        const title = document.querySelector('.content main h1')?.textContent?.trim() ||
                      document.title.replace(' - NullBlock Internal Docs', '').trim();

        // Convert HTML to agent-friendly plain text
        const agentText = htmlToAgentText(content, title);

        // Copy to clipboard
        navigator.clipboard.writeText(agentText).then(() => {
            showFeedback('Copied!', true);
        }).catch(err => {
            console.error('Failed to copy:', err);
            showFeedback('Failed to copy', false);
        });
    }

    function htmlToAgentText(element, title) {
        // Clone to avoid modifying the original
        const clone = element.cloneNode(true);

        // Remove elements that shouldn't be in the copy
        const removeSelectors = [
            '.header-anchor',      // Anchor links
            'script',              // Scripts
            'style',               // Styles
            '.hidden',             // Hidden elements
            '.copy-page-button',   // Our button
            'button.clip-button',  // Code copy buttons
        ];
        removeSelectors.forEach(sel => {
            clone.querySelectorAll(sel).forEach(el => el.remove());
        });

        // Build the agent-friendly output
        let output = [];

        // Add document header
        output.push(`# ${title}`);
        output.push('');
        output.push(`Source: ${window.location.href}`);
        output.push('');
        output.push('---');
        output.push('');

        // Process the content
        output.push(processNode(clone));

        return output.join('\n').trim();
    }

    function processNode(node) {
        let result = [];

        for (const child of node.childNodes) {
            if (child.nodeType === Node.TEXT_NODE) {
                const text = child.textContent.trim();
                if (text) {
                    result.push(text);
                }
            } else if (child.nodeType === Node.ELEMENT_NODE) {
                const tag = child.tagName.toLowerCase();

                switch (tag) {
                    case 'h1':
                        result.push('\n# ' + child.textContent.trim() + '\n');
                        break;
                    case 'h2':
                        result.push('\n## ' + child.textContent.trim() + '\n');
                        break;
                    case 'h3':
                        result.push('\n### ' + child.textContent.trim() + '\n');
                        break;
                    case 'h4':
                        result.push('\n#### ' + child.textContent.trim() + '\n');
                        break;
                    case 'h5':
                    case 'h6':
                        result.push('\n##### ' + child.textContent.trim() + '\n');
                        break;
                    case 'p':
                        const pText = processInlineContent(child);
                        if (pText.trim()) {
                            result.push('\n' + pText + '\n');
                        }
                        break;
                    case 'pre':
                        const code = child.querySelector('code');
                        const lang = code?.className?.match(/language-(\w+)/)?.[1] || '';
                        const codeText = code?.textContent || child.textContent;
                        result.push('\n```' + lang + '\n' + codeText.trim() + '\n```\n');
                        break;
                    case 'code':
                        // Inline code (not in pre)
                        if (child.parentElement?.tagName?.toLowerCase() !== 'pre') {
                            result.push('`' + child.textContent + '`');
                        }
                        break;
                    case 'ul':
                        result.push('\n' + processList(child, false) + '\n');
                        break;
                    case 'ol':
                        result.push('\n' + processList(child, true) + '\n');
                        break;
                    case 'table':
                        result.push('\n' + processTable(child) + '\n');
                        break;
                    case 'blockquote':
                        const quoteText = processNode(child);
                        result.push('\n> ' + quoteText.split('\n').join('\n> ') + '\n');
                        break;
                    case 'hr':
                        result.push('\n---\n');
                        break;
                    case 'a':
                        const href = child.getAttribute('href');
                        const linkText = child.textContent.trim();
                        if (href && !href.startsWith('#')) {
                            result.push(`[${linkText}](${href})`);
                        } else {
                            result.push(linkText);
                        }
                        break;
                    case 'strong':
                    case 'b':
                        result.push('**' + child.textContent + '**');
                        break;
                    case 'em':
                    case 'i':
                        result.push('*' + child.textContent + '*');
                        break;
                    case 'br':
                        result.push('\n');
                        break;
                    case 'div':
                    case 'section':
                    case 'article':
                    case 'main':
                        result.push(processNode(child));
                        break;
                    default:
                        // For other elements, just get their text content
                        const text = processNode(child);
                        if (text.trim()) {
                            result.push(text);
                        }
                }
            }
        }

        return result.join('').replace(/\n{3,}/g, '\n\n');
    }

    function processInlineContent(element) {
        let result = [];
        for (const child of element.childNodes) {
            if (child.nodeType === Node.TEXT_NODE) {
                result.push(child.textContent);
            } else if (child.nodeType === Node.ELEMENT_NODE) {
                const tag = child.tagName.toLowerCase();
                switch (tag) {
                    case 'code':
                        result.push('`' + child.textContent + '`');
                        break;
                    case 'strong':
                    case 'b':
                        result.push('**' + child.textContent + '**');
                        break;
                    case 'em':
                    case 'i':
                        result.push('*' + child.textContent + '*');
                        break;
                    case 'a':
                        const href = child.getAttribute('href');
                        const text = child.textContent.trim();
                        if (href && !href.startsWith('#')) {
                            result.push(`[${text}](${href})`);
                        } else {
                            result.push(text);
                        }
                        break;
                    case 'br':
                        result.push('\n');
                        break;
                    default:
                        result.push(child.textContent);
                }
            }
        }
        return result.join('');
    }

    function processList(listElement, ordered) {
        let items = [];
        let index = 1;
        for (const li of listElement.querySelectorAll(':scope > li')) {
            const prefix = ordered ? `${index}. ` : '- ';
            const content = processInlineContent(li);
            items.push(prefix + content.trim());

            // Handle nested lists
            const nestedUl = li.querySelector(':scope > ul');
            const nestedOl = li.querySelector(':scope > ol');
            if (nestedUl) {
                const nested = processList(nestedUl, false);
                items.push(nested.split('\n').map(line => '  ' + line).join('\n'));
            }
            if (nestedOl) {
                const nested = processList(nestedOl, true);
                items.push(nested.split('\n').map(line => '  ' + line).join('\n'));
            }
            index++;
        }
        return items.join('\n');
    }

    function processTable(table) {
        let rows = [];
        const headerRow = table.querySelector('thead tr') || table.querySelector('tr:first-child');
        const bodyRows = table.querySelectorAll('tbody tr') || table.querySelectorAll('tr:not(:first-child)');

        // Process header
        if (headerRow) {
            const headers = Array.from(headerRow.querySelectorAll('th, td'))
                .map(cell => cell.textContent.trim());
            rows.push('| ' + headers.join(' | ') + ' |');
            rows.push('| ' + headers.map(() => '---').join(' | ') + ' |');
        }

        // Process body rows
        bodyRows.forEach(row => {
            if (row === headerRow) return; // Skip if it was used as header
            const cells = Array.from(row.querySelectorAll('td, th'))
                .map(cell => cell.textContent.trim().replace(/\|/g, '\\|'));
            if (cells.length > 0) {
                rows.push('| ' + cells.join(' | ') + ' |');
            }
        });

        return rows.join('\n');
    }

    function showFeedback(message, success) {
        const button = document.getElementById('copy-page-btn');
        if (!button) return;

        const originalHTML = button.innerHTML;
        button.innerHTML = `
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                ${success
                    ? '<polyline points="20 6 9 17 4 12"></polyline>'
                    : '<line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line>'
                }
            </svg>
            <span>${message}</span>
        `;
        button.classList.add(success ? 'success' : 'error');

        setTimeout(() => {
            button.innerHTML = originalHTML;
            button.classList.remove('success', 'error');
        }, 2000);
    }
})();
