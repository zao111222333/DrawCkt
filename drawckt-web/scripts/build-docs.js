import { readdirSync, readFileSync, writeFileSync, mkdirSync, statSync, copyFileSync } from 'fs';
import { join, resolve, dirname, extname, basename } from 'path';
import { marked } from 'marked';

// Configure marked for better HTML output
marked.setOptions({
  gfm: true,
  breaks: true,
});

// Extract headings from markdown content for TOC
function extractHeadings(content) {
  const headingRegex = /^(#{1,6})\s+(.+)$/gm;
  const headings = [];
  let match;
  
  while ((match = headingRegex.exec(content)) !== null) {
    const level = match[1].length;
    const text = match[2].trim();
    const id = text.toLowerCase()
      .replace(/[^\w\s-]/g, '')
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-');
    headings.push({ level, text, id });
  }
  
  return headings;
}

// Generate TOC HTML from headings
function generateTOC(headings) {
  if (headings.length === 0) return '';
  
  let html = '<nav class="toc">\n  <h2>Contents</h2>\n  <ul>\n';
  let stack = []; // Stack to track open lists at each level
  
  for (const heading of headings) {
    // Close lists that are deeper than or equal to current heading level
    while (stack.length > 0 && stack[stack.length - 1] >= heading.level) {
      html += '    </ul>\n';
      stack.pop();
    }
    
    // Open nested list if current heading is deeper than previous
    if (stack.length > 0 && stack[stack.length - 1] < heading.level) {
      html += '    <ul>\n';
      stack.push(heading.level);
    }
    
    html += `    <li><a href="#${heading.id}">${heading.text}</a></li>\n`;
  }
  
  // Close all remaining nested lists
  while (stack.length > 0) {
    html += '    </ul>\n';
    stack.pop();
  }
  
  html += '  </ul>\n</nav>\n';
  return html;
}

// Add ID attributes to headings in HTML content
function addHeadingIds(htmlContent, headings) {
  let result = htmlContent;
  for (const heading of headings) {
    // Match heading tags (h1-h6) with the heading text
    const regex = new RegExp(`<h${heading.level}>(.*?)</h${heading.level}>`, 'gi');
    result = result.replace(regex, (match, text) => {
      // Check if this heading matches our heading text
      const cleanText = text.replace(/<[^>]*>/g, '').trim();
      if (cleanText === heading.text) {
        return `<h${heading.level} id="${heading.id}">${text}</h${heading.level}>`;
      }
      return match;
    });
  }
  return result;
}

// Generate HTML page with TOC and content
function generateHTMLPage(title, content, toc, basePath = '../') {
  return `<!DOCTYPE html>
<html lang="en-US">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${title} - Documentation</title>
  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
        'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
        sans-serif;
      line-height: 1.6;
      color: #333;
      background: #fff;
    }
    
    .container {
      max-width: 1200px;
      margin: 0 auto;
      padding: 20px;
      display: flex;
      gap: 30px;
    }
    
    .sidebar {
      width: 250px;
      flex-shrink: 0;
      position: sticky;
      top: 20px;
      height: fit-content;
      max-height: calc(100vh - 40px);
      overflow-y: auto;
    }
    
    .toc {
      background: #f5f5f5;
      padding: 20px;
      border-radius: 8px;
      border: 1px solid #e0e0e0;
    }
    
    .toc h2 {
      font-size: 18px;
      margin-bottom: 15px;
      color: #222;
    }
    
    .toc ul {
      list-style: none;
      padding-left: 0;
    }
    
    .toc ul ul {
      padding-left: 20px;
      margin-top: 5px;
    }
    
    .toc li {
      margin: 5px 0;
    }
    
    .toc a {
      color: #0066cc;
      text-decoration: none;
      font-size: 14px;
      display: block;
      padding: 4px 0;
    }
    
    .toc a:hover {
      color: #0052a3;
      text-decoration: underline;
    }
    
    .content {
      flex: 1;
      min-width: 0;
    }
    
    .content h1 {
      font-size: 32px;
      margin-bottom: 20px;
      padding-bottom: 10px;
      border-bottom: 2px solid #e0e0e0;
    }
    
    .content h2 {
      font-size: 24px;
      margin-top: 30px;
      margin-bottom: 15px;
      padding-top: 10px;
    }
    
    .content h3 {
      font-size: 20px;
      margin-top: 25px;
      margin-bottom: 12px;
    }
    
    .content h4 {
      font-size: 18px;
      margin-top: 20px;
      margin-bottom: 10px;
    }
    
    .content p {
      margin-bottom: 15px;
    }
    
    .content ul, .content ol {
      margin-bottom: 15px;
      padding-left: 30px;
    }
    
    .content li {
      margin: 5px 0;
    }
    
    .content a {
      color: #0066cc;
      text-decoration: none;
    }
    
    .content a:hover {
      text-decoration: underline;
    }
    
    .content code {
      background: #f5f5f5;
      padding: 2px 6px;
      border-radius: 3px;
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      font-size: 0.9em;
    }
    
    .content pre {
      background: #f5f5f5;
      padding: 15px;
      border-radius: 5px;
      overflow-x: auto;
      margin-bottom: 15px;
    }
    
    .content pre code {
      background: none;
      padding: 0;
    }
    
    .content blockquote {
      border-left: 4px solid #ddd;
      padding-left: 15px;
      margin: 15px 0;
      color: #666;
    }
    
    .content table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 15px;
    }
    
    .content th, .content td {
      border: 1px solid #ddd;
      padding: 8px 12px;
      text-align: left;
    }
    
    .content th {
      background: #f5f5f5;
      font-weight: bold;
    }
    
    .nav-links {
      margin-bottom: 20px;
      padding-bottom: 20px;
      border-bottom: 1px solid #e0e0e0;
    }
    
    .nav-links a {
      color: #0066cc;
      text-decoration: none;
      margin-right: 15px;
    }
    
    .nav-links a:hover {
      text-decoration: underline;
    }
    
    @media (max-width: 768px) {
      .container {
        flex-direction: column;
      }
      
      .sidebar {
        width: 100%;
        position: static;
      }
    }
  </style>
</head>
<body>
  <div class="container">
    <aside class="sidebar">
      ${toc}
    </aside>
    <main class="content">
      ${content}
    </main>
  </div>
</body>
</html>`;
}

// Process a markdown file
function processMarkdownFile(filePath, outputDir, basePath) {
  const content = readFileSync(filePath, 'utf-8');
  const fileName = basename(filePath, '.md');
  const headings = extractHeadings(content);
  const toc = generateTOC(headings);
  let htmlContent = marked.parse(content);
  
  // Add ID attributes to headings for anchor links
  htmlContent = addHeadingIds(htmlContent, headings);
  
  // Extract title from first h1 or use filename
  const firstH1 = headings.find(h => h.level === 1);
  const title = firstH1 ? firstH1.text : fileName;
  
  const html = generateHTMLPage(title, htmlContent, toc, basePath);
  const outputPath = join(outputDir, `${fileName}.html`);
  writeFileSync(outputPath, html, 'utf-8');
  
  return { fileName, title, path: `${fileName}.html` };
}

// Copy non-markdown files
function copyNonMarkdownFiles(sourceDir, outputDir) {
  const files = readdirSync(sourceDir);
  for (const file of files) {
    const filePath = join(sourceDir, file);
    const stat = statSync(filePath);
    
    if (stat.isFile() && extname(file) !== '.md') {
      const outputPath = join(outputDir, file);
      copyFileSync(filePath, outputPath);
    }
  }
}

// Main function
function buildDocs() {
  const docDir = resolve(process.cwd(), 'static/doc');
  const distDir = resolve(process.cwd(), 'dist');
  const outputDir = join(distDir, 'doc');
  
  // Create output directory
  mkdirSync(outputDir, { recursive: true });
  
  // Get all markdown files
  const files = readdirSync(docDir);
  const markdownFiles = files.filter(f => extname(f) === '.md');
  
  const pages = [];
  
  // Process each markdown file
  for (const file of markdownFiles) {
    const filePath = join(docDir, file);
    const page = processMarkdownFile(filePath, outputDir, '../');
    pages.push(page);
  }
  
  // Copy non-markdown files (like test.il)
  copyNonMarkdownFiles(docDir, outputDir);
  
  // Generate index.html if index.md exists
  const indexPage = pages.find(p => p.fileName === 'index');
  if (indexPage) {
    // Create a redirect or use index.html as the main page
    const indexPath = join(outputDir, 'index.html');
    if (!indexPage.path.endsWith('index.html')) {
      // If index.md was processed, it should already be index.html
      // But let's make sure
      const indexContent = readFileSync(indexPath, 'utf-8');
      writeFileSync(indexPath, indexContent, 'utf-8');
    }
  }
  
  console.log(`✓ Built ${pages.length} documentation page(s) to ${outputDir}`);
  pages.forEach(page => {
    console.log(`  - ${page.path}`);
  });
}

// Export for use in other modules
export { buildDocs };

// Run if executed directly
// In ES modules, we can check if this file is being run directly
// by comparing import.meta.url with the resolved script path
const isMainModule = import.meta.url === `file://${resolve(process.argv[1])}` ||
                     process.argv[1]?.endsWith('build-docs.js');
if (isMainModule) {
  buildDocs();
}

