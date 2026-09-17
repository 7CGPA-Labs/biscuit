import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { parse, HtmlGenerator } from 'latex.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const filePath = path.join(__dirname, '../tests/preview_tests/pure_latex.tex');
let text = fs.readFileSync(filePath, 'utf-8');

let safeLatex = text.replace(/^---[\s\S]*?---\s*/, '')
    .replace(/\\usepackage\{.*?\}/g, '')
    .replace(/\\begin\{equation\}/g, '\\[')
    .replace(/\\end\{equation\}/g, '\\]')
    .replace(/\\begin\{align\}/g, '\\[\\begin{aligned}')
    .replace(/\\end\{align\}/g, '\\end{aligned}\\]');

try {
  const generator = new HtmlGenerator({ hyphenate: false });
  parse(safeLatex, { generator: generator });
  console.log("Success pure_latex");
} catch (e) {
  console.log("Error pure_latex:", e.message);
}

const filePath2 = path.join(__dirname, '../tests/preview_tests/latex_yaml.tex');
let text2 = fs.readFileSync(filePath2, 'utf-8');
let safeLatex2 = text2.replace(/^---[\s\S]*?---\s*/, '')
    .replace(/\\usepackage\{.*?\}/g, '')
    .replace(/\\begin\{equation\}/g, '\\[')
    .replace(/\\end\{equation\}/g, '\\]')
    .replace(/\\begin\{align\}/g, '\\[\\begin{aligned}')
    .replace(/\\end\{align\}/g, '\\end{aligned}\\]');

try {
  const generator = new HtmlGenerator({ hyphenate: false });
  parse(safeLatex2, { generator: generator });
  console.log("Success latex_yaml");
} catch (e) {
  console.log("Error latex_yaml:", e.message);
}
