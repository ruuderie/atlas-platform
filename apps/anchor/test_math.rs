fn main() {
    let md = r#"Here is inline \( L \subseteq \{0,1\}^* \) and $x=y$ and display $$a=b$$."#;
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_MATH);
    let parser = pulldown_cmark::Parser::new_ext(md, options);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    println!("{}", html);
}
