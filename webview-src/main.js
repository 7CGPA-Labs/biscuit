import markdownIt from 'markdown-it';
import mathjax3 from 'markdown-it-mathjax3';
import hljs from 'highlight.js';
import { parse, HtmlGenerator } from 'latex.js';

const md = markdownIt({
  html: true,
  linkify: true,
  typographer: true,
  highlight: function (str, lang) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value;
      } catch (__) {}
    }
    return ''; // use external default escaping
  }
}).use(mathjax3);

window.renderContent = function(text, isLatex) {
  const app = document.getElementById('app');
  app.innerHTML = '';
  app.className = '';
  
  // Strip YAML frontmatter for both MD and TEX if present
  let processedText = text.replace(/^\s*---[\s\S]*?---\s*/, '');

  if (isLatex) {
    try {
      // latex.js in browser mode cannot dynamically load packages like amsmath.
      // So we strip \usepackage{amsmath} to prevent the require() error.
      // And we replace \begin{align} with \[ \begin{aligned} which is natively supported.
      let safeLatex = processedText
        .replace(/\\documentclass(?:(?:\[[\s\S]*?\])?\s*\{[\s\S]*?\})?/g, '')
        .replace(/\\begin\s*\{document\}/g, '')
        .replace(/\\end\s*\{document\}/g, '')
        .replace(/\\usepackage(?:(?:\[[\s\S]*?\])?\s*\{[\s\S]*?\})?/g, '')
        .replace(/\\begin\s*\{equation\}/g, '\\[')
        .replace(/\\end\s*\{equation\}/g, '\\]')
        .replace(/\\begin\s*\{align\}/g, '\\[\\begin{aligned}')
        .replace(/\\end\s*\{align\}/g, '\\end{aligned}\\]');
        
      app.className = 'latex-container';
      const generator = new HtmlGenerator({ hyphenate: false });
      const doc = parse(safeLatex, { generator: generator }).htmlDocument();
      app.appendChild(doc.documentElement);
    } catch (e) {
      app.innerHTML = `<div style="color: #f38ba8;"><strong>LaTeX Error:</strong><br><pre>${e}</pre></div>`;
    }
  } else {
    try {
      app.innerHTML = md.render(processedText);
    } catch (e) {
      app.innerHTML = `<div style="color: #f38ba8;"><strong>Markdown Error:</strong><br><pre>${e}</pre></div>`;
    }
  }
};

// Initial placeholder
window.renderContent("# Loading preview...", false);
