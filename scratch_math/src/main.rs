fn main() {
    let md = r#"A language $L \subseteq \{0,1\}^*$ belongs to **P**

$$
\phi \text{ is satisfiable} \quad \Leftrightarrow \quad A(\phi) \text{ outputs ``unsatisfiable''}.
$$"#;
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_MATH);
    let parser = pulldown_cmark::Parser::new_ext(md, options);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    println!("HTML:\n{}", html);
}
