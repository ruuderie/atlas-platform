//! CLI utility to compile Markdown market strategy reports in `docs/market-analysis` to high-fidelity PDFs.
//!
//! Usage examples:
//!   Compile all reports:
//!     cargo run --bin generate_market_reports -- --all
//!   Compile a single report:
//!     cargo run --bin generate_market_reports -- --file ../docs/market-analysis/private_lending_us.md
//!   Compile all to a specific folder:
//!     cargo run --bin generate_market_reports -- --all --output ../docs/market-analysis/pdf

use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;
use pulldown_cmark::{Parser as MarkdownParser, Event, Tag};

#[derive(Parser, Debug)]
#[command(
    name = "generate_market_reports",
    author = "Ruud Erie",
    version = "2.1.0",
    about = "Compiles Markdown market reports in docs/market-analysis to beautiful, premium LaTeX PDFs",
    long_about = None
)]
struct Args {
    /// Compile all markdown files in the market analysis directory
    #[arg(short, long)]
    all: bool,

    /// Compile a specific markdown file
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Custom output directory or path for the generated PDF(s)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 1. Resolve the docs root directory relative to the current working directory
    let current_dir = std::env::current_dir()?;
    let docs_root = if current_dir.ends_with("backend") {
        current_dir.parent().unwrap().join("docs")
    } else {
        current_dir.join("docs")
    };
    let reports_root = docs_root.join("reports");

    // 2. Process according to command line flags
    if args.all {
        if !reports_root.exists() || !reports_root.is_dir() {
            return Err(format!(
                "Reports directory not found at: {}\nTry specifying the direct file path using --file",
                reports_root.display()
            ).into());
        }

        // Handle creating output directory if specified
        if let Some(ref out_dir) = args.output {
            if !out_dir.exists() {
                println!("Creating output directory: {}", out_dir.display());
                fs::create_dir_all(out_dir)?;
            }
        }

        // Dynamically find all subdirectories in reports_root as categories
        let mut categories = Vec::new();
        for entry in fs::read_dir(&reports_root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy().to_string();
                    if name_str != "pdf" && name_str != "tmp_compile" && !name_str.starts_with('.') {
                        categories.push(name_str);
                    }
                }
            }
        }
        categories.sort();

        let mut files_processed = 0;
        let mut files_skipped = 0;

        for cat in &categories {
            let cat_dir = reports_root.join(cat);
            if !cat_dir.exists() || !cat_dir.is_dir() {
                continue;
            }

            println!("Scanning directory for {}: {}", cat, cat_dir.display());
            let entries = fs::read_dir(&cat_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "md") && path.file_name().unwrap().to_str().unwrap().to_lowercase() != "readme.md" {
                    // Smart check: Only compile if no PDF exists yet (from today or in past version control),
                    // or if the newest existing PDF is older than the MD file (meaning it has been updated since).
                    let existing_pdf = find_existing_pdf(&path, &args.output);
                    let should_compile = if let Some(ref pdf_path) = existing_pdf {
                        if let (Ok(md_meta), Ok(pdf_meta)) = (fs::metadata(&path), fs::metadata(pdf_path)) {
                            if let (Ok(md_mod), Ok(pdf_mod)) = (md_meta.modified(), pdf_meta.modified()) {
                                md_mod > pdf_mod
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if should_compile {
                        if let Err(e) = compile_report(&path, &args.output) {
                            println!("Failed to compile {}: {}", path.display(), e);
                        } else {
                            files_processed += 1;
                        }
                    } else {
                        files_skipped += 1;
                        println!("Skipping up-to-date document: {}", path.file_name().unwrap().to_str().unwrap());
                    }
                }
            }
        }

        if files_processed == 0 {
            if files_skipped > 0 {
                println!("All {} documents are already up-to-date.", files_skipped);
            } else {
                println!("No Markdown files (.md) found in supported categories.");
            }
        } else {
            println!("\nSuccessfully generated {} documents (skipped {} up-to-date).", files_processed, files_skipped);
        }
    } else if let Some(file_path) = args.file {
        if !file_path.exists() {
            return Err(format!("Specified file does not exist: {}", file_path.display()).into());
        }
        
        // Handle creating output directory if specified
        if let Some(ref out_dir) = args.output {
            if !out_dir.exists() && out_dir.extension().is_none() {
                println!("Creating output directory: {}", out_dir.display());
                fs::create_dir_all(out_dir)?;
            }
        }

        compile_report(&file_path, &args.output)?;
        println!("\nSuccessfully generated report PDF!");
    } else {
        println!("Error: Please specify either --all or --file <PATH>");
        println!("Run with --help for all available options.");
        std::process::exit(1);
    }

    Ok(())
}

/// Helper: Escapes special characters for LaTeX safety
fn latex_escape(s: &str) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '&' => escaped.push_str("\\&"),
            '%' => escaped.push_str("\\%"),
            '$' => escaped.push_str("\\$"),
            '#' => escaped.push_str("\\#"),
            '_' => escaped.push_str("\\_"),
            '{' => escaped.push_str("\\{"),
            '}' => escaped.push_str("\\}"),
            '~' => escaped.push_str("\\textasciitilde{}"),
            '^' => escaped.push_str("\\textasciicircum{}"),
            '\\' => escaped.push_str("\\textbackslash{}"),
            '·' => escaped.push_str(" \\textbullet{} "),
            '→' => escaped.push_str(" $\\rightarrow$ "),
            '←' => escaped.push_str(" $\\leftarrow$ "),
            '↔' => escaped.push_str(" $\\leftrightarrow$ "),
            '↓' => escaped.push_str(" $\\downarrow$ "),
            '↑' => escaped.push_str(" $\\uparrow$ "),
            '—' => escaped.push_str("---"),
            '–' => escaped.push_str("--"),
            '−' => escaped.push_str("-"),
            '|' => escaped.push_str("\\textbar{}"),
            '≤' => escaped.push_str("$\\le$"),
            '≥' => escaped.push_str("$\\ge$"),
            '≠' => escaped.push_str("$\\ne$"),
            '≈' => escaped.push_str("$\\approx$"),
            // Emojis mapping for LaTeX compatibility
            '✅' => escaped.push_str("[YES] "),
            '❌' => escaped.push_str("[NO] "),
            '⚠' => escaped.push_str("[WARN] "),
            '🔴' => escaped.push_str("[CRITICAL] "),
            '🟡' => escaped.push_str("[DEFERRED] "),
            '🔵' => escaped.push_str("[INFO] "),
            '🏆' => escaped.push_str("[WINNER] "),
            '🔑' => escaped.push_str("[KEY] "),
            '\u{FE0F}' | '\u{FE00}' | '\u{FE01}' | '\u{FE02}' | '\u{FE03}' | '\u{FE04}' | '\u{FE05}' | '\u{FE06}' | '\u{FE07}' | '\u{FE08}' | '\u{FE09}' | '\u{FE0A}' | '\u{FE0B}' | '\u{FE0C}' | '\u{FE0D}' | '\u{FE0E}' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{200E}' | '\u{200F}' => {}
            _ => {
                let code = c as u32;
                if (code >= 0x2600 && code <= 0x27BF) 
                    || (code >= 0x1F000 && code <= 0x1FFFF)
                {
                    // Skip generic emojis to prevent LaTeX compilation issues
                } else {
                    escaped.push(c);
                }
            }
        }
    }
    escaped
}

struct ParserState {
    tex_content: String,
    current_block: String,
    current_cell: String,
    current_row: Vec<String>,
    table_rows: Vec<Vec<String>>,
    in_table: bool,
    in_table_cell: bool,
    table_alignments: Vec<pulldown_cmark::Alignment>,
    in_list_level: usize,
    in_ordered_list: Vec<bool>,
    first_h1: bool,
    in_code_block: bool,
    code_block_content: String,
    active_heading_level: Option<pulldown_cmark::HeadingLevel>,
}

/// Helper: Finds the newest existing PDF in the output folder for a given report, regardless of date suffix.
fn find_existing_pdf(md_path: &Path, custom_output: &Option<PathBuf>) -> Option<PathBuf> {
    let file_stem = md_path.file_stem().unwrap().to_str().unwrap();
    let pdf_dir = if let Some(out) = custom_output {
        if out.is_dir() || out.extension().is_none() {
            out.clone()
        } else {
            out.parent().unwrap_or(Path::new(".")).to_path_buf()
        }
    } else {
        md_path.parent().unwrap().join("pdf")
    };

    if !pdf_dir.exists() || !pdf_dir.is_dir() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(&pdf_dir) {
        let mut newest_pdf: Option<(PathBuf, std::time::SystemTime)> = None;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "pdf") {
                    let name = path.file_stem().unwrap().to_str().unwrap();
                    // Match either exact name or name with date suffix: {file_stem} or {file_stem}_{YYYY-MM-DD}
                    if name == file_stem || (name.starts_with(file_stem) && name.len() > file_stem.len() && name.chars().nth(file_stem.len()) == Some('_')) {
                        if let Ok(meta) = fs::metadata(&path) {
                            if let Ok(modified) = meta.modified() {
                                if newest_pdf.is_none() || modified > newest_pdf.as_ref().unwrap().1 {
                                    newest_pdf = Some((path, modified));
                                }
                            }
                        }
                    }
                }
            }
        }
        return newest_pdf.map(|(path, _)| path);
    }
    None
}

/// Helper: Resolves output PDF path with date suffix YYYY-MM-DD for version control
fn get_pdf_path(md_path: &Path, custom_output: &Option<PathBuf>) -> PathBuf {
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    if let Some(out) = custom_output {
        if out.is_dir() || out.extension().is_none() {
            let file_name = md_path.file_stem().unwrap().to_str().unwrap();
            let final_name = if file_name.ends_with(&date_str) {
                format!("{}.pdf", file_name)
            } else {
                format!("{}_{}.pdf", file_name, date_str)
            };
            out.join(final_name)
        } else {
            out.clone()
        }
    } else {
        let pdf_dir = md_path.parent().unwrap().join("pdf");
        let file_name = md_path.file_stem().unwrap().to_str().unwrap();
        let final_name = if file_name.ends_with(&date_str) {
            format!("{}.pdf", file_name)
        } else {
            format!("{}_{}.pdf", file_name, date_str)
        };
        pdf_dir.join(final_name)
    }
}

/// Helper: Formats a category directory name (like "market-analysis") into a Title Case string (like "Market Analysis")
fn format_category_title(name: &str) -> String {
    name.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Helper: Traverses the path of the markdown report to determine the category folder under the reports root directory
fn get_category_name(path: &Path) -> String {
    let mut current = path.parent();
    while let Some(p) = current {
        if p.file_name().map_or(false, |n| n == "reports") {
            for ancestor in path.ancestors() {
                if ancestor.parent() == Some(p) {
                    return ancestor.file_name().unwrap_or_default().to_string_lossy().to_string();
                }
            }
        }
        current = p.parent();
    }
    // Fallback: use the direct parent directory name
    path.parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "generic".to_string())
}

#[derive(Clone, Copy, Debug)]
enum DocLanguage {
    English,
    Portuguese,
    Spanish,
}

struct LanguageAssets {
    header_text: String,
    footer_copyright: String,
    confidential_text: String,
}

impl LanguageAssets {
    fn get(category_name: &str, lang: DocLanguage) -> Self {
        let category_title = format_category_title(category_name);
        match lang {
            DocLanguage::Portuguese => {
                let header = match category_name {
                    "market-analysis" => "Análise de Mercado Vertical da Atlas Platform · CONFIDENCIAL".to_string(),
                    "sales" | "sales-analysis" => "Análise de Vendas Técnicas da Atlas Platform · CONFIDENCIAL".to_string(),
                    _ => format!("Atlas Platform {} · CONFIDENCIAL", category_title),
                };
                Self {
                    header_text: header,
                    footer_copyright: "© 2026 Sistema Atlas Platform".to_string(),
                    confidential_text: "CONFIDENCIAL".to_string(),
                }
            }
            DocLanguage::Spanish => {
                let header = match category_name {
                    "market-analysis" => "Análisis de Mercado de Atlas Platform · CONFIDENCIAL".to_string(),
                    "sales" | "sales-analysis" => "Análisis de Ventas Técnicas de Atlas Platform · CONFIDENCIAL".to_string(),
                    _ => format!("Atlas Platform {} · CONFIDENCIAL", category_title),
                };
                Self {
                    header_text: header,
                    footer_copyright: "© 2026 Sistema Atlas Platform".to_string(),
                    confidential_text: "CONFIDENCIAL".to_string(),
                }
            }
            DocLanguage::English => {
                let header = match category_name {
                    "market-analysis" => "Atlas Platform Vertical Market Analysis · CONFIDENTIAL".to_string(),
                    "sales" | "sales-analysis" => "Atlas Platform Technical Sales Analysis · CONFIDENTIAL".to_string(),
                    _ => format!("Atlas Platform {} · CONFIDENTIAL", category_title),
                };
                Self {
                    header_text: header,
                    footer_copyright: "© 2026 Atlas Platform System".to_string(),
                    confidential_text: "CONFIDENTIAL".to_string(),
                }
            }
        }
    }
}

fn detect_language(md_path: &Path, content: &str) -> DocLanguage {
    let file_name = md_path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    if file_name.ends_with("_pt_br.md") || file_name.ends_with("_br.md") || file_name.contains("_pt_br") || file_name.contains("_br_") {
        return DocLanguage::Portuguese;
    }
    if file_name.ends_with("_es.md") || file_name.contains("_es_") {
        return DocLanguage::Spanish;
    }

    // Fallback: simple heuristic scanning for common unique stopwords
    let content_lower = content.to_lowercase();
    
    // We check for highly specific keywords
    let pt_indicators = ["relação", "geração", "serviço", "portfólios", "recomendações", "empreendedor", "infraestrutura"];
    let es_indicators = ["relación", "generación", "servicio", "portafolios", "recomendaciones", "emprendedor", "infraestructura"];
    
    let mut pt_count = 0;
    let mut es_count = 0;
    
    for &word in &pt_indicators {
        if content_lower.contains(word) {
            pt_count += 1;
        }
    }
    for &word in &es_indicators {
        if content_lower.contains(word) {
            es_count += 1;
        }
    }
    
    if pt_count > es_count && pt_count >= 2 {
        DocLanguage::Portuguese
    } else if es_count > pt_count && es_count >= 2 {
        DocLanguage::Spanish
    } else {
        DocLanguage::English
    }
}

/// Compiles a single Markdown report to PDF using pdflatex
fn compile_report(
    md_path: &Path,
    custom_output: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let md_content = fs::read_to_string(md_path)?;
    
    // Resolve output PDF path
    let pdf_path = get_pdf_path(md_path, custom_output);

    // Ensure output directory exists
    if let Some(parent) = pdf_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    println!(
        "Compiling: {} -> {}",
        md_path.file_name().unwrap().to_str().unwrap(),
        pdf_path.file_name().unwrap().to_str().unwrap()
    );

    // Detect category, language, and fetch category-specific language assets
    let category_name = get_category_name(md_path);
    let lang = detect_language(md_path, &md_content);
    let lang_assets = LanguageAssets::get(&category_name, lang);

    // Setup LaTeX source
    let mut preamble = format!(
        r#"
\documentclass[10pt,a4paper]{{article}}
\usepackage[utf8]{{inputenc}}
\usepackage[table]{{xcolor}}
\usepackage[left=20mm,top=22mm,right=20mm,bottom=22mm]{{geometry}}
\usepackage{{tabularx}}
\usepackage{{booktabs}}
\usepackage{{array}}
\usepackage{{fancyhdr}}
\usepackage{{listings}}
\usepackage{{hyperref}}

% ── Kami color tokens ────────────────────────────────────────────────────────
\definecolor{{brand}}{{HTML}}{{1B365D}}
\definecolor{{nearblack}}{{HTML}}{{141413}}
\definecolor{{olive}}{{HTML}}{{504E49}}
\definecolor{{stone}}{{HTML}}{{6B6A64}}
\definecolor{{bordersoft}}{{HTML}}{{E5E3D8}}
\definecolor{{parchment}}{{HTML}}{{F5F4ED}}
\definecolor{{tagbg}}{{HTML}}{{EEF2F7}}

\pagecolor{{parchment}}
\color{{nearblack}}

% ── Section styling macros (replacing titlesec) ─────────────────────────────
\newcommand{{\mysection}}[1]{{%
  \vspace{{14pt}}%
  \noindent{{\leavevmode\llap{{\textcolor{{brand}}{{\rule[-0.3ex]{{2.5pt}}{{1.1em}}\hspace{{6pt}}}}}}}}\textbf{{\large\textcolor{{nearblack}}{{#1}}}}%
  \vspace{{4pt}}\\%
  {{\color{{bordersoft}}\hrule height 0.4pt}}%
  \vspace{{6pt}}%
}}

\newcommand{{\mysubsection}}[1]{{%
  \vspace{{10pt}}%
  \noindent\textbf{{\large\textcolor{{brand}}{{#1}}}}%
  \vspace{{4pt}}\\%
}}

\newcommand{{\mysubsubsection}}[1]{{%
  \vspace{{8pt}}%
  \noindent\textbf{{\textcolor{{nearblack}}{{#1}}}}%
  \vspace{{4pt}}\\%
}}

% ── Native list spacing adjustments (replacing enumitem) ────────────────────
\renewcommand{{\labelitemi}}{{\textcolor{{brand}}{{\textendash}}}}
\makeatletter
\renewcommand{{\@listI}}{{%
  \leftmargin=15pt%
  \rightmargin=0pt%
  \labelsep=5pt%
  \labelwidth=10pt%
  \itemsep=2pt%
  \parsep=0pt%
  \topsep=2pt%
}}
\makeatother

% ── Listings layout for code blocks and ASCII art ───────────────────────────
\lstset{{
  basicstyle=\ttfamily\small,
  backgroundcolor=\color{{tagbg}},
  frame=single,
  rulecolor=\color{{bordersoft}},
  framesep=6pt,
  xleftmargin=10pt,
  xrightmargin=10pt,
  breaklines=true,
  showstringspaces=false
}}

% ── Tabularx column types ───────────────────────────────────────────────────
\newcolumntype{{L}}{{>{{\raggedright\arraybackslash}}X}}
\newcolumntype{{R}}{{>{{\raggedleft\arraybackslash}}X}}
\newcolumntype{{C}}{{>{{\centering\arraybackslash}}X}}

% ── Hyperref ────────────────────────────────────────────────────────────────
\hypersetup{{
  colorlinks=true,
  linkcolor=brand,
  filecolor=brand,
  urlcolor=brand
}}

% ── Running Headers and Footers ─────────────────────────────────────────────
\pagestyle{{fancy}}
\fancyhf{{}}
\fancyhead[L]{{\small\textcolor{{stone}}{{{header_text}}}}}
\fancyhead[R]{{\small\textcolor{{brand}}{{\textbf{{\thepage}}}}}}
\fancyfoot[L]{{\small\textcolor{{stone}}{{{footer_copyright}}}}}
\fancyfoot[R]{{\small\textcolor{{stone}}{{{confidential_text}}}}}
\renewcommand{{\headrulewidth}}{{0.4pt}}
\renewcommand{{\footrulewidth}}{{0.4pt}}
\renewcommand{{\headrule}} Horiz{{\hbox to\headwidth{{\color{{bordersoft}}\leaders\hrule height \headrulewidth\hfill}}}}
\renewcommand{{\footrule}} Horiz{{\hbox to\headwidth{{\color{{bordersoft}}\leaders\hrule height \footrulewidth\hfill}}}}

\begin{{document}}
"#,
        header_text = lang_assets.header_text,
        footer_copyright = lang_assets.footer_copyright,
        confidential_text = lang_assets.confidential_text
    );

    // Set custom page header and footer thin rules
    preamble = preamble.replace(" Horiz", "");

    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = MarkdownParser::new_ext(&md_content, options);
    let mut state = ParserState {
        tex_content: preamble,
        current_block: String::new(),
        current_cell: String::new(),
        current_row: Vec::new(),
        table_rows: Vec::new(),
        in_table: false,
        in_table_cell: false,
        table_alignments: Vec::new(),
        in_list_level: 0,
        in_ordered_list: Vec::new(),
        first_h1: true,
        in_code_block: false,
        code_block_content: String::new(),
        active_heading_level: None,
    };

    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        state.active_heading_level = Some(level);
                        state.current_block.clear();
                    }
                    Tag::Paragraph => {
                        state.current_block.clear();
                    }
                    Tag::Strong => {
                        if state.in_table_cell {
                            state.current_cell.push_str("\\textbf{");
                        } else {
                            state.current_block.push_str("\\textbf{");
                        }
                    }
                    Tag::Emphasis => {
                        if state.in_table_cell {
                            state.current_cell.push_str("\\textit{");
                        } else {
                            state.current_block.push_str("\\textit{");
                        }
                    }
                    Tag::Link { dest_url, .. } => {
                        let escaped_dest = latex_escape(&dest_url);
                        if state.in_table_cell {
                            state.current_cell.push_str(&format!("\\href{{{}}}{{", escaped_dest));
                        } else {
                            state.current_block.push_str(&format!("\\href{{{}}}{{", escaped_dest));
                        }
                    }
                    Tag::List(start_num) => {
                        state.in_list_level += 1;
                        if start_num.is_some() {
                            state.in_ordered_list.push(true);
                            state.tex_content.push_str("\\begin{enumerate}\n");
                        } else {
                            state.in_ordered_list.push(false);
                            state.tex_content.push_str("\\begin{itemize}\n");
                        }
                    }
                    Tag::Item => {
                        state.tex_content.push_str("\\item ");
                        state.current_block.clear();
                    }
                    Tag::CodeBlock(_) => {
                        state.in_code_block = true;
                        state.code_block_content.clear();
                    }
                    Tag::Table(alignments) => {
                        state.in_table = true;
                        state.table_alignments = alignments.clone();
                        state.table_rows.clear();
                    }
                    Tag::TableHead => {
                        state.current_row.clear();
                    }
                    Tag::TableRow => {
                        state.current_row.clear();
                    }
                    Tag::TableCell => {
                        state.in_table_cell = true;
                        state.current_cell.clear();
                    }
                    _ => {}
                }
            }
            Event::End(tag_end) => {
                use pulldown_cmark::TagEnd;
                match tag_end {
                    TagEnd::Heading(_) => {
                        let text = state.current_block.trim();
                        let level = state.active_heading_level.take().unwrap_or(pulldown_cmark::HeadingLevel::H2);
                        if state.first_h1 && level == pulldown_cmark::HeadingLevel::H1 {
                            state.tex_content.push_str(&format!(
                                r#"
\begin{{center}}
  {{\fontsize{{22pt}}{{26pt}}\selectfont\textbf{{\textcolor{{brand}}{{{}}}}}}} \\[8pt]
  {{\large\textcolor{{stone}}{{{}}}}} \\[12pt]
  {{\textcolor{{bordersoft}}{{\hrule height 1.2pt}}}}
\end{{center}}
\vspace{{10pt}}
"#,
                                text,
                                lang_assets.header_text
                            ));
                            state.first_h1 = false;
                        } else {
                            let cmd = match level {
                                pulldown_cmark::HeadingLevel::H1 => "mysection",
                                pulldown_cmark::HeadingLevel::H2 => "mysection",
                                pulldown_cmark::HeadingLevel::H3 => "mysubsection",
                                pulldown_cmark::HeadingLevel::H4 => "mysubsubsection",
                                _ => "mysubsection",
                            };
                            state.tex_content.push_str(&format!("\\{}{{{}}}\n", cmd, text));
                        }
                    }
                    TagEnd::Paragraph => {
                        if !state.current_block.trim().is_empty() {
                            state.tex_content.push_str(&format!("{}\n\n", state.current_block.trim()));
                        }
                        state.current_block.clear();
                    }
                    TagEnd::Strong => {
                        if state.in_table_cell {
                            state.current_cell.push_str("}");
                        } else {
                            state.current_block.push_str("}");
                        }
                    }
                    TagEnd::Emphasis => {
                        if state.in_table_cell {
                            state.current_cell.push_str("}");
                        } else {
                            state.current_block.push_str("}");
                        }
                    }
                    TagEnd::Link => {
                        if state.in_table_cell {
                            state.current_cell.push_str("}");
                        } else {
                            state.current_block.push_str("}");
                        }
                    }
                    TagEnd::List(_) => {
                        state.in_list_level -= 1;
                        let ordered = state.in_ordered_list.pop().unwrap_or(false);
                        if ordered {
                            state.tex_content.push_str("\\end{enumerate}\n\n");
                        } else {
                            state.tex_content.push_str("\\end{itemize}\n\n");
                        }
                    }
                    TagEnd::Item => {
                        if !state.current_block.trim().is_empty() {
                            state.tex_content.push_str(&format!("{}\n", state.current_block.trim()));
                        }
                        state.current_block.clear();
                    }
                    TagEnd::CodeBlock => {
                        state.in_code_block = false;
                        state.tex_content.push_str("\\begin{lstlisting}\n");
                        state.tex_content.push_str(&state.code_block_content);
                        state.tex_content.push_str("\\end{lstlisting}\n\n");
                    }
                    TagEnd::Table => {
                        state.in_table = false;
                        let table_latex = generate_latex_table(&state.table_rows, &state.table_alignments);
                        state.tex_content.push_str(&table_latex);
                        state.tex_content.push_str("\n");
                    }
                    TagEnd::TableHead => {
                        state.table_rows.push(state.current_row.clone());
                    }
                    TagEnd::TableRow => {
                        state.table_rows.push(state.current_row.clone());
                    }
                    TagEnd::TableCell => {
                        state.in_table_cell = false;
                        state.current_row.push(state.current_cell.clone());
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if state.in_code_block {
                    let mut cleaned = text.to_string();
                    for &(from, to) in &[
                        ("→", "->"),
                        ("←", "<-"),
                        ("↔", "<->"),
                        ("↓", "v"),
                        ("↑", "^"),
                        ("≤", "<="),
                        ("≥", ">="),
                        ("≠", "!="),
                        ("≈", "~"),
                        ("⚠️", "[WARN]"),
                        ("⚠", "[WARN]"),
                        ("✅", "[YES]"),
                        ("❌", "[NO]"),
                        ("│", "|"),
                        ("─", "-"),
                        ("┌", "+"),
                        ("┐", "+"),
                        ("└", "+"),
                        ("┘", "+"),
                        ("├", "+"),
                        ("┤", "+"),
                        ("┬", "+"),
                        ("┴", "+"),
                        ("┼", "+"),
                        ("—", "-"),
                        ("–", "-"),
                        ("−", "-"),
                        ("“", "\""),
                        ("”", "\""),
                        ("‘", "'"),
                        ("’", "'"),
                        ("ã", "a"),
                        ("õ", "o"),
                        ("á", "a"),
                        ("é", "e"),
                        ("í", "i"),
                        ("ó", "o"),
                        ("ú", "u"),
                        ("â", "a"),
                        ("ê", "e"),
                        ("ô", "o"),
                        ("ç", "c"),
                        ("Ã", "A"),
                        ("Õ", "O"),
                        ("Á", "A"),
                        ("É", "E"),
                        ("Í", "I"),
                        ("Ó", "O"),
                        ("Ú", "U"),
                        ("Â", "A"),
                        ("Ê", "E"),
                        ("Ô", "O"),
                        ("Ç", "C"),
                    ] {
                        cleaned = cleaned.replace(from, to);
                    }
                    state.code_block_content.push_str(&cleaned);
                } else {
                    let escaped = latex_escape(&text);
                    if text.contains("⚠️") || escaped.contains("[WARN]") {
                        println!("DEBUG: Raw text: {:?}, Escaped: {:?}", text, escaped);
                    }
                    if state.in_table_cell {
                        state.current_cell.push_str(&escaped);
                    } else {
                        state.current_block.push_str(&escaped);
                    }
                }
            }
            Event::Code(code) => {
                let escaped = latex_escape(&code);
                let code_latex = format!("\\texttt{{{}}}", escaped);
                if state.in_table_cell {
                    state.current_cell.push_str(&code_latex);
                } else {
                    state.current_block.push_str(&code_latex);
                }
            }
            Event::SoftBreak => {
                if state.in_table_cell {
                    state.current_cell.push(' ');
                } else {
                    state.current_block.push(' ');
                }
            }
            Event::HardBreak => {
                if state.in_table_cell {
                    state.current_cell.push_str(" \\\\ ");
                } else {
                    state.current_block.push_str(" \\\\ ");
                }
            }
            Event::Rule => {
                state.tex_content.push_str("\\vspace{8pt}\\textcolor{bordersoft}{\\hrule height 0.8pt}\\vspace{8pt}\n\n");
            }
            _ => {}
        }
    }

    state.tex_content.push_str("\\end{document}\n");

    // 3. isolated temporary compilation within output directory
    let out_dir = pdf_path.parent().unwrap();
    let temp_dir = out_dir.join("tmp_compile");
    fs::create_dir_all(&temp_dir)?;

    let tex_filename = "report.tex";
    let tex_path = temp_dir.join(tex_filename);
    fs::write(&tex_path, &state.tex_content)?;

    // Run pdflatex
    let mut command = std::process::Command::new("pdflatex");
    command.current_dir(&temp_dir);
    command.arg("-interaction=nonstopmode");
    command.arg(tex_filename);

    let output = match command.output() {
        Ok(out) => Ok(out),
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Fall back to direct absolute path on macOS MacTeX
            let mut fallback_cmd = std::process::Command::new("/Library/TeX/texbin/pdflatex");
            fallback_cmd.current_dir(&temp_dir);
            fallback_cmd.arg("-interaction=nonstopmode");
            fallback_cmd.arg(tex_filename);
            fallback_cmd.output()
        }
        Err(e) => Err(e),
    };

    let compile_success = match output {
        Ok(out) => {
            if out.status.success() {
                true
            } else {
                // pdflatex exits non-zero on warnings/overfull hboxes but still
                // produces a valid PDF. Check if the output file actually exists.
                let pdf_produced = temp_dir.join("report.pdf").exists();
                if pdf_produced {
                    println!("Note: pdflatex exited with warnings (non-zero) but PDF was produced successfully.");
                    true
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    println!("LaTeX Compilation failed!\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to start pdflatex compiler process: {}", e);
            false
        }
    };

    if !compile_success {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err("LaTeX compilation failed".into());
    }

    // Copy PDF to destination
    let generated_pdf = temp_dir.join("report.pdf");
    if generated_pdf.exists() {
        fs::copy(&generated_pdf, &pdf_path)?;
    } else {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err("Compiled PDF not found".into());
    }

    // Clean up completely
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Helper: Formats table rows into beautiful, high-fidelity LaTeX tabularx code
fn generate_latex_table(
    rows: &[Vec<String>],
    alignments: &[pulldown_cmark::Alignment],
) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let col_count = rows[0].len();
    if col_count == 0 {
        return String::new();
    }

    let mut latex = String::new();
    latex.push_str("\\vspace{10pt}\n");
    latex.push_str("\\noindent\n");
    
    // Column specifier: map alignments to L, R, C
    let mut spec = String::new();
    for i in 0..col_count {
        let align = alignments.get(i).unwrap_or(&pulldown_cmark::Alignment::None);
        match align {
            pulldown_cmark::Alignment::Left | pulldown_cmark::Alignment::None => spec.push_str("L"),
            pulldown_cmark::Alignment::Right => spec.push_str("R"),
            pulldown_cmark::Alignment::Center => spec.push_str("C"),
        }
    }
    
    latex.push_str("\\rowcolors{2}{tagbg}{white}\n");
    latex.push_str(&format!("\\begin{{tabularx}}{{\\textwidth}}{{{}}}\n", spec));
    
    // Render Header Row
    latex.push_str("\\rowcolor{brand}\n");
    let header_row = &rows[0];
    let formatted_headers: Vec<String> = header_row
        .iter()
        .map(|cell| format!("\\textcolor{{white}}{{\\textbf{{{}}}}}", cell.trim()))
        .collect();
    latex.push_str(&formatted_headers.join(" & "));
    latex.push_str(" \\\\\n");
    latex.push_str("\\arrayrulecolor{bordersoft}\\hline\n");
    
    // Render Body Rows
    for row in rows.iter().skip(1) {
        let formatted_cells: Vec<String> = row
            .iter()
            .map(|cell| cell.trim().to_string())
            .collect();
        latex.push_str(&formatted_cells.join(" & "));
        latex.push_str(" \\\\\n");
        latex.push_str("\\arrayrulecolor{bordersoft}\\hline\n");
    }
    
    latex.push_str("\\end{tabularx}\n");
    latex.push_str("\\vspace{10pt}\n");
    latex
}
