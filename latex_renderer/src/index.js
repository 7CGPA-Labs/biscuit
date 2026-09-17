window.renderLatex = function(content) {
    try {
        document.body.innerHTML = '';
        
        // LaTeX.js does not support \usepackage synchronously in the browser, nor does it support
        // equation/align environments perfectly natively. Since it uses KaTeX under the hood,
        // we can replace block math environments with standard KaTeX block math delimiters ($$).
        content = content.replace(/\\usepackage\{.*?\}/g, '');
        content = content.replace(/\\begin\{equation\}/g, "$$");
        content = content.replace(/\\end\{equation\}/g, "$$");
        content = content.replace(/\\begin\{align\}/g, "$$\\begin{aligned}");
        content = content.replace(/\\end\{align\}/g, "\\end{aligned}$$");
        
        var generator = new latexjs.HtmlGenerator({ hyphenate: false, baseURL: './' });
        var doc = latexjs.parse(content, { generator: generator });
        document.body.appendChild(doc.domFragment());
    } catch (e) {
        document.body.innerHTML = '<div style="color:red; background:white; position:fixed; top:0; left:0; z-index:9999; width:100%; height:100%; overflow:auto; padding: 1rem;">' +
            '<h3>LaTeX Parsing Error</h3>' +
            '<pre>' + e.message + '</pre>' +
            '</div>';
    }
}
