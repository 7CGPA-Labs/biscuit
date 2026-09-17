use pulldown_cmark::{html, Options, Parser};

fn main() {
    let md = "$E=mc^2$\n\n$$\n\\int_0^\\infty dx\n$$";
    let mut options = Options::empty();
    options.insert(Options::ENABLE_MATH);
    let parser = Parser::new_ext(md, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    println!("WITH MATH:\n{}", html_output);
}
