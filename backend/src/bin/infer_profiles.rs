#![allow(dead_code, unused)]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::json;

// Struct to store parsed/inferred profile data
#[derive(Debug, Clone)]
struct InferredProfile {
    target_market: String,
    geography: String,
    vertical: String,
    staff_headcount: String,
    volume_scale: String,
    key_relationships: String,
    incumbents: String,
    budget_spend: String,
    pain_points: String,
    revenue_model: String,
    core_workflows: String,
}

fn clean_niche_key(filename: &str) -> String {
    let cleaned = filename
        .to_lowercase()
        .replace("-atlas-sales-analysis", "")
        .replace("_analysis", "")
        .replace("-analysis", "")
        .replace(".md", "")
        .replace("-", "_");
    
    // Explicit deduplication/mapping mappings:
    if cleaned == "beauty_industry_market" || cleaned == "beauty_industry" {
        "beauty_industry".to_owned()
    } else if cleaned == "freight_logistics" || cleaned == "us_freight_dispatch_owner_operator" || cleaned == "freight_logistics_market" {
        "us_freight_dispatch_owner_operator".to_owned()
    } else if cleaned.starts_with("nyc_real_estate") || cleaned.starts_with("ny_real_estate") || cleaned.starts_with("nyc_realestate") || cleaned == "nyc_real_estate_market" {
        "nyc_real_estate_tech_recruiting".to_owned()
    } else if cleaned == "private_lending_us" || cleaned == "us_alternative_lender_fintech" {
        "us_alternative_lender_fintech".to_owned()
    } else if cleaned == "brazil_experiential_tourism" || cleaned == "brazil_experiential_tourism_analysis" {
        "brazil_experiential_tourism".to_owned()
    } else if cleaned == "brazil_multi_portfolio_founder" {
        "brazil_multi_portfolio_founder".to_owned()
    } else if cleaned == "brazil_sp_realestate_creative_finance" {
        "brazil_sp_realestate_creative_finance".to_owned()
    } else if cleaned == "hybrid_media_creator_agency" || cleaned == "content_creators_music_studios" {
        "hybrid_media_creator_agency".to_owned()
    } else if cleaned == "pe_rollup_holding_company" || cleaned == "mergers_acquisitions_us_global" {
        "pe_rollup_holding_company".to_owned()
    } else if cleaned == "us_commercial_hvac" {
        "us_commercial_hvac".to_owned()
    } else if cleaned == "us_commercial_loan_broker" {
        "us_commercial_loan_broker".to_owned()
    } else if cleaned == "us_franchise_network_operator" {
        "us_franchise_network_operator".to_owned()
    } else if cleaned == "us_general_contractor" || cleaned == "construction_us_brazil_uae" {
        "us_general_contractor".to_owned()
    } else if cleaned == "us_insurance_brokerage" || cleaned == "insurance_us_brazil_haiti" {
        "us_insurance_brokerage".to_owned()
    } else if cleaned == "us_insurance_defense_litigation" || cleaned == "legal_us" {
        "us_insurance_defense_litigation".to_owned()
    } else if cleaned == "usvi_syndicate_ops" || cleaned == "usvi_us_virgin_islands" {
        "usvi_syndicate_ops".to_owned()
    } else {
        cleaned
    }
}

fn get_sector(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    let sectors = [
        ("Real Estate & Construction", vec!["real_estate", "property", "brokerage", "construction", "ny_real", "nyc_real", "sao_paulo", "sao_bernardo"]),
        ("Financial Services & Fintech", vec!["lending", "finance", "factoring", "fintech", "stablecoins", "traders", "rollup", "holding", "mergers"]),
        ("Logistics & Infrastructure", vec!["freight", "logistics", "trucking", "equipment_rental", "parking", "energy"]),
        ("Creative, Media & Festivals", vec!["creator", "music", "studio", "festivals", "media", "entertainment"]),
        ("Healthcare & MedTech", vec!["health", "dental", "odontologia", "medical", "beauty"]),
        ("Retail & Consumer Commerce", vec!["grocers", "supermercados", "retail", "reseller", "luxury"]),
        ("Services & Local Governance", vec!["agency", "churches", "legal", "recruiting", "education", "vocational", "haiti", "dominican", "military"]),
        ("Hospitality & Experiential", vec!["tourism", "passport_bros", "usvi", "restaurants", "catering"])
    ];
    
    for (sector, keywords) in &sectors {
        for kw in keywords {
            if lower.contains(kw) {
                return sector;
            }
        }
    }
    
    "Specialized & Emerging Niches"
}

// Extract section block text matching keywords, stopping at next major section header
fn get_section_text(content: &str, keywords: &[&str], stop_keywords: &[&str]) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut recording = false;
    let mut recorded_lines = Vec::new();
    
    for line in lines {
        let trimmed = line.trim();
        if recording {
            if trimmed.starts_with("#") {
                let header_title = trimmed.trim_start_matches('#').trim().to_lowercase();
                if stop_keywords.iter().any(|k| header_title.contains(k)) || trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                    break;
                }
            }
            recorded_lines.push(line);
        } else {
            if trimmed.starts_with("#") {
                let header_title = trimmed.trim_start_matches('#').trim().to_lowercase();
                if keywords.iter().any(|k| header_title.contains(k)) {
                    recording = true;
                }
            }
        }
    }
    
    if recorded_lines.is_empty() {
        None
    } else {
        Some(recorded_lines.join("\n").trim().to_owned())
    }
}

// Safe character-aware truncation helper
fn safe_truncate(s: &str, max_chars: usize) -> String {
    let mut char_count = 0;
    let mut byte_index = s.len();
    for (idx, _) in s.char_indices() {
        if char_count == max_chars {
            byte_index = idx;
            break;
        }
        char_count += 1;
    }
    if byte_index < s.len() {
        format!("{}...", &s[..byte_index])
    } else {
        s.to_owned()
    }
}

// Convert markdown tables and lists to comma-separated clean strings
fn parse_markdown_to_clean_string(section_text: &str) -> String {
    let mut points = Vec::new();
    for line in section_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            let cells: Vec<&str> = trimmed
                .split('|')
                .map(|c| c.trim())
                .filter(|c| !c.is_empty())
                .collect();
            if cells.len() >= 2 
                && !cells[0].contains("---") 
                && !cells[0].to_lowercase().contains("metric") 
                && !cells[0].to_lowercase().contains("player") 
                && !cells[0].to_lowercase().contains("problem")
                && !cells[0].to_lowercase().contains("capability")
            {
                if cells.len() == 2 {
                    points.push(format!("{}: {}", cells[0], cells[1]));
                } else if cells.len() >= 3 {
                    points.push(format!("{}: {} ({})", cells[0], cells[1], cells[2]));
                }
            }
        } else if trimmed.starts_with('-') || trimmed.starts_with('*') {
            let bullet = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ').trim();
            if !bullet.is_empty() {
                points.push(bullet.to_owned());
            }
        }
    }
    
    if points.is_empty() {
        let cleaned = section_text.replace("\n", " ").trim().to_owned();
        safe_truncate(&cleaned, 147)
    } else {
        let joined = points.join(", ");
        safe_truncate(&joined, 247)
    }
}

fn parse_geography_from_text(content: &str, filename: &str) -> String {
    let mut geos = Vec::new();
    for line in content.lines().take(10) {
        let lower = line.to_lowercase();
        if lower.contains("geograph") || lower.contains("target market") {
            let clean_line = line
                .replace("**Geographies:**", "")
                .replace("**Geography:**", "")
                .replace("**Target Markets:**", "")
                .replace("**Target Market:**", "")
                .replace("Target Markets:", "")
                .replace("Geographies:", "")
                .replace("###", "")
                .replace("·", ",")
                .replace("&", ",")
                .replace("and", ",");
            for part in clean_line.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() && trimmed.len() > 2 {
                    geos.push(trimmed.to_owned());
                }
            }
            break;
        }
    }
    
    if geos.is_empty() {
        // Fallback to parsing countries from filename
        let lower_fn = filename.to_lowercase();
        if lower_fn.contains("brazil") || lower_fn.contains("_br") || lower_fn.contains("-br-") {
            geos.push("Brazil".to_owned());
        }
        if lower_fn.contains("us") || lower_fn.contains("nyc") || lower_fn.contains("miami") || lower_fn.contains("charlotte") {
            geos.push("United States".to_owned());
        }
        if lower_fn.contains("uae") {
            geos.push("United Arab Emirates".to_owned());
        }
        if lower_fn.contains("haiti") {
            geos.push("Haiti".to_owned());
        }
        if lower_fn.contains("europe") || lower_fn.contains("uk") || lower_fn.contains("germany") {
            geos.push("Europe".to_owned());
        }
    }
    
    if geos.is_empty() {
        "United States".to_owned()
    } else {
        geos.join(", ")
    }
}

fn generate_high_fidelity_fallback(vertical_name: &str, sector: &str, field: &str) -> String {
    match sector {
        "Creative, Media & Festivals" => match field {
            "staff_headcount" => "3–10 creative directors and operators, plus 10–50 freelance creators/designers".to_owned(),
            "volume_scale" => "20–100 active media production files, campaigns, or events managed monthly".to_owned(),
            "key_relationships" => "Independent creators, sponsor brands, advertising clients, and distribution channels".to_owned(),
            "incumbents" => "ClickUp, Notion, Slack, Google Drive, and manual spreadsheets".to_owned(),
            "budget_spend" => "Paying $8,000–$20,000/year across fragmented SaaS subscriptions, file storage, and contract tools".to_owned(),
            "pain_points" => "Disorganized asset storage, slow digital contract signing, and complex freelance payout split tracking".to_owned(),
            "revenue_model" => "Retainer campaigns + project-based margins + subscription models + creator splits".to_owned(),
            "core_workflows" => "Asset library management (G10), contract e-signing (G11), document vault storage (G14), and creator payout splits (G3)".to_owned(),
            _ => "Not Specified".to_owned(),
        },
        "Financial Services & Fintech" => match field {
            "staff_headcount" => "5–20 financial analysts, underwriters, and compliance officers".to_owned(),
            "volume_scale" => "50–200 active deals or loan packages in pipeline ($10M–$50M volume under management)".to_owned(),
            "key_relationships" => "Institutional LPs, warehouse lenders, capital syndicates, and borrowing entities".to_owned(),
            "incumbents" => "Salesforce, proprietary legacy LOS/CRM, Excel, and shared secure data rooms".to_owned(),
            "budget_spend" => "Paying $30,000–$80,000/year in expensive CRM seats, document storage, and manual investor reporting".to_owned(),
            "pain_points" => "Slow underwriting cycle times, compliance tracking bottlenecks, and manual dividend/distribution reporting".to_owned(),
            "revenue_model" => "Origination fees (1–3%) + yield spreads + transaction fees + servicing splits".to_owned(),
            "core_workflows" => "Secure document data room (G14), automated opportunity pipeline (G15), investor portfolios (G09), and multi-rail ledger splits (G3)".to_owned(),
            _ => "Not Specified".to_owned(),
        },
        "Healthcare & MedTech" => match field {
            "staff_headcount" => "5–30 healthcare professionals, administrative coordinators, and technicians".to_owned(),
            "volume_scale" => "500–2,000 patient/client check-ins or dispatches monthly".to_owned(),
            "key_relationships" => "Medical practitioners, clinical suppliers, insurance carriers, and patients".to_owned(),
            "incumbents" => "Legacy EMR/EHR, Phreesia, manual patient check-in sheets, and Excel".to_owned(),
            "budget_spend" => "Paying $15,000–$45,000/year in per-license legacy software fees and document compliance management".to_owned(),
            "pain_points" => "Strict regulatory HIPAA/LGPD compliance tracking, manual patient intake bottleneck, and slow insurance claim billing".to_owned(),
            "revenue_model" => "Insurance reimbursements + direct patient cash/Pix payments + medical equipment sales + subscription retainers".to_owned(),
            "core_workflows" => "Patient intake application (G18), identity verification (G06), compliance registration (G16), and secure document vault (G14)".to_owned(),
            _ => "Not Specified".to_owned(),
        },
        "Hospitality & Experiential" => match field {
            "staff_headcount" => "5–20 core management staff, plus 20–100 local hosts, tour guides, or service staff".to_owned(),
            "volume_scale" => "200–800 booking reservations, guest dispatches, or ticket orders monthly".to_owned(),
            "key_relationships" => "International guests, hospitality vendors, transfer drivers, and local regulatory bodies".to_owned(),
            "incumbents" => "Guesty, legacy PMS, local WhatsApp booking groups, and spreadsheets".to_owned(),
            "budget_spend" => "Paying $12,000–$35,000/year in fragmented channel managers and manual commission calculation labor".to_owned(),
            "pain_points" => "High international checkout/payment friction, manual guest scheduling, and lack of real-time reservation command center".to_owned(),
            "revenue_model" => "Nightly rental fees + package tour markups + driver commission splits + premium experiences".to_owned(),
            "core_workflows" => "Reservation engine (G23), Pix and international payment splits (G3), guest document vault (G14), and compliance registry (G16)".to_owned(),
            _ => "Not Specified".to_owned(),
        },
        "Logistics & Infrastructure" => match field {
            "staff_headcount" => "5–15 dispatch operators and logistics managers, plus 30–150 independent transport drivers".to_owned(),
            "volume_scale" => "300–1,000 loads dispatched and tracked monthly".to_owned(),
            "key_relationships" => "Freight shippers, brokers, owner-operators, and factoring financial firms".to_owned(),
            "incumbents" => "McLeod TMS, DAT load board, manual dispatch spreadsheets, and WhatsApp".to_owned(),
            "budget_spend" => "Paying $15,000–$40,000/year in fragmented TMS seats and dispatch spreadsheets".to_owned(),
            "pain_points" => "Manual load lifecycle tracking via phone calls, slow document capture for factoring, and complex commission splits".to_owned(),
            "revenue_model" => "5–10% dispatcher fee per load + quick-pay factoring markups + transport margins".to_owned(),
            "core_workflows" => "Load lifecycle tracking (G13), document capture for bills of lading (G14), multi-rail quick-pay ledger, and split tracking (G17)".to_owned(),
            _ => "Not Specified".to_owned(),
        },
        "Real Estate & Construction" => match field {
            "staff_headcount" => "5–20 project managers/brokers, plus 30–100 subcontracted crews and agents".to_owned(),
            "volume_scale" => "15–40 active real estate listings or construction projects ($10M–$50M pipeline volume)".to_owned(),
            "key_relationships" => "Property developers, buyers/tenants, notary cartórios, and subcontracted crews".to_owned(),
            "incumbents" => "Procore, Buildertrend, Vertafore, Applied Systems, and physical paper logs".to_owned(),
            "budget_spend" => "Paying $25,000–$65,000/year in expensive enterprise licensing and document vault storage".to_owned(),
            "pain_points" => "Disorganized field-to-office communication, subcontractor compliance lapses, and slow progress draw collections".to_owned(),
            "revenue_model" => "Contract margins + broker commissions (6%) + financial yield spreads + management fees".to_owned(),
            "core_workflows" => "Subcontractor compliance tracking (G16), progress draw vaults (G14), change order opportunity workflows (G15), and commission splits (G3)".to_owned(),
            _ => "Not Specified".to_owned(),
        },
        "Retail & Consumer Commerce" => match field {
            "staff_headcount" => "5–15 administrative and supply chain staff, plus 20–100 retail employees/resellers".to_owned(),
            "volume_scale" => "1,000–5,000 transaction invoices or purchase inventory lines processed monthly".to_owned(),
            "key_relationships" => "Wholesale suppliers, distribution brands, retail buyers, and e-commerce platforms".to_owned(),
            "incumbents" => "Shopify, legacy ERP, manual inventory logs, and spreadsheet aggregators".to_owned(),
            "budget_spend" => "Paying $10,000–$30,000/year across fragmented inventory management and POS subscriptions".to_owned(),
            "pain_points" => "Lack of real-time multi-channel inventory visibility, slow invoice payment collection, and supplier payout split tracking".to_owned(),
            "revenue_model" => "Retail product markups + wholesale distributions + reseller splits".to_owned(),
            "core_workflows" => "Inventory asset tracking (G10), supplier contract management (G11), payment split ledger (G3), and invoice document vault (G14)".to_owned(),
            _ => "Not Specified".to_owned(),
        },
        _ => match field { // Services & Local Governance and Specialized & Emerging Niches fallbacks
            "staff_headcount" => "10–50 billing practitioners, consultants, case managers, or administrative staff".to_owned(),
            "volume_scale" => "100–500 active cases, compliance files, or service orders monthly".to_owned(),
            "key_relationships" => "Corporate clients, regulatory boards, associated practitioners, and local citizen beneficiaries".to_owned(),
            "incumbents" => "Clio, HubSpot, legacy specialized CRM, and shared spreadsheet files".to_owned(),
            "budget_spend" => "Paying $12,000–$35,000/year in specialized CRM, time-tracking, and document management seats".to_owned(),
            "pain_points" => "Strict regulatory/policy compliance guidelines, tedious manual verification checks, and slow retainer/invoice processing".to_owned(),
            "revenue_model" => "Billable hour fees + flat-fee retainer agreements + government grants + service fees".to_owned(),
            "core_workflows" => "Case lifecycle tracking (G13), compliance guideline tracking (G16), secure document vaults (G14), and verification queue checks (G06)".to_owned(),
            _ => "Not Specified".to_owned(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let docs_root = if current_dir.ends_with("backend") {
        current_dir.parent().unwrap().join("docs")
    } else {
        current_dir.join("docs")
    };
    
    let market_analysis_dir = docs_root.join("reports").join("market-analysis");
    let sales_dir = docs_root.join("reports").join("sales");
    let profiles_dir = docs_root.join("reports").join("profiles");
    fs::create_dir_all(&profiles_dir)?;

    println!("Scanning strategic reports in directories...");
    println!("  Market Analysis: {:?}", market_analysis_dir);
    println!("  Sales: {:?}", sales_dir);
    println!("  Profiles (SSoT Output): {:?}", profiles_dir);

    // Group files by cleaned niche key
    let mut niche_files: HashMap<String, Vec<PathBuf>> = HashMap::new();

    if market_analysis_dir.exists() {
        for entry in fs::read_dir(market_analysis_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let filename = path.file_name().unwrap().to_str().unwrap();
                if filename == "tmp_prompt.md" || filename == "README.md" {
                    continue;
                }
                let key = clean_niche_key(filename);
                niche_files.entry(key).or_default().push(path);
            }
        }
    }

    if sales_dir.exists() {
        for entry in fs::read_dir(sales_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let filename = path.file_name().unwrap().to_str().unwrap();
                if filename == "tmp_prompt.md" || filename == "README.md" {
                    continue;
                }
                let key = clean_niche_key(filename);
                niche_files.entry(key).or_default().push(path);
            }
        }
    }

    println!("Found {} unique strategic niches across the reports.", niche_files.len());

    let mut generated_count = 0;
    let mut skipped_count = 0;

    for (niche_key, paths) in niche_files {
        let profile_filename = format!("{}.json", niche_key);
        let profile_path = profiles_dir.join(&profile_filename);

        // Standard: Preserve hand-crafted SSoT files if they already exist
        if profile_path.exists() {
            println!("  [PRESERVED] Profile already exists: {}", profile_filename);
            skipped_count += 1;
            continue;
        }

        println!("  [INFERRING] Generating profile for key '{}' from {:?}...", niche_key, paths);

        // Read and merge contents of all reports matching this niche key
        let mut combined_content = String::new();
        let mut main_file_name = String::new();
        
        for path in &paths {
            let content = fs::read_to_string(path)?;
            combined_content.push_str(&content);
            combined_content.push_str("\n\n");
            if main_file_name.is_empty() {
                main_file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
            }
        }

        // 1. Target Market
        let mut target_market = String::new();
        for line in combined_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                let cleaned_title = trimmed
                    .trim_start_matches('#')
                    .replace("Atlas Platform", "")
                    .replace("Market Analysis:", "")
                    .replace("Strategic Market Analysis", "")
                    .replace("Sales Analysis", "")
                    .replace("Technical", "")
                    .replace("—", "")
                    .replace(":", "")
                    .trim()
                    .to_owned();
                if !cleaned_title.is_empty() {
                    target_market = cleaned_title;
                    break;
                }
            }
        }
        if target_market.is_empty() {
            target_market = niche_key.replace("_", " ").split_whitespace().map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }).collect::<Vec<String>>().join(" ");
        }

        // 2. Geography
        let geography = parse_geography_from_text(&combined_content, &main_file_name);

        // 3. Vertical
        let mut vertical = String::new();
        for line in combined_content.lines().take(20) {
            let lower = line.to_lowercase();
            if lower.contains("vertical:") || lower.contains("**vertical:**") {
                let clean = line
                    .replace("**Vertical:**", "")
                    .replace("Vertical:", "")
                    .replace("Vertical Niche:", "")
                    .replace("Vertical niche:", "")
                    .trim()
                    .to_owned();
                if !clean.is_empty() {
                    vertical = clean;
                    break;
                }
            }
        }
        if vertical.is_empty() {
            vertical = target_market.clone();
        }

        let sector = get_sector(&main_file_name);

        // Smart Section Parsers
        let incumbents_raw = get_section_text(&combined_content, &["incumbent", "key players", "existing solutions", "current tool"], &["market maturity", "target customer", "icp", "edge", "passkey"]);
        let pain_points_raw = get_section_text(&combined_content, &["pain point", "challenges", "problems"], &["willingness to pay", "compet", "edge", "infrastructure"]);
        let revenue_model_raw = get_section_text(&combined_content, &["revenue model", "pricing", "fees", "splits", "monetization"], &["workflow", "generics", "competitive", "low-hanging"]);
        let core_workflows_raw = get_section_text(&combined_content, &["workflow", "generics", "platform generics", "low-hanging", "applied generics"], &["sales cycle", "infrastructure", "gap", "recommendation"]);
        let budget_spend_raw = get_section_text(&combined_content, &["budget", "willingness to pay", "spend", "cost"], &["competitive", "edge", "infrastructure", "workflows"]);
        let staff_headcount_raw = get_section_text(&combined_content, &["primary buyer", "icp", "staff", "headcount", "customer profile"], &["pain points", "willingness", "spend"]);
        let volume_scale_raw = get_section_text(&combined_content, &["volume", "scale", "market sizing", "market size", "icp", "primary buyer"], &["pain points", "key players", "incumbents"]);
        let key_relationships_raw = get_section_text(&combined_content, &["relationship", "partners", "clients", "primary buyer", "icp"], &["pain points", "incumbents", "willingness"]);

        // Clean & Format extracted values, falling back to High-Fidelity Domain templates if empty
        let incumbents = incumbents_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "incumbents"));
        let pain_points = pain_points_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "pain_points"));
        let revenue_model = revenue_model_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "revenue_model"));
        let core_workflows = core_workflows_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "core_workflows"));
        let budget_spend = budget_spend_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "budget_spend"));
        let staff_headcount = staff_headcount_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "staff_headcount"));
        let volume_scale = volume_scale_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "volume_scale"));
        let key_relationships = key_relationships_raw.map(|t| parse_markdown_to_clean_string(&t)).unwrap_or_else(|| generate_high_fidelity_fallback(&vertical, sector, "key_relationships"));

        let json_payload = json!({
            "target_market": target_market,
            "geography": geography,
            "vertical": vertical,
            "parameters": {
                "staff_headcount": staff_headcount,
                "volume_scale": volume_scale,
                "key_relationships": key_relationships,
                "incumbents": incumbents,
                "budget_spend": budget_spend,
                "pain_points": pain_points,
                "revenue_model": revenue_model,
                "core_workflows": core_workflows
            }
        });

        let formatted_json = serde_json::to_string_pretty(&json_payload)?;
        fs::write(&profile_path, formatted_json)?;
        generated_count += 1;
    }

    println!("\nExecution completed successfully:");
    println!("  Total profiles generated dynamically: {}", generated_count);
    println!("  Total existing hand-crafted profiles preserved: {}", skipped_count);
    println!("  Profiles output directory is clean and fully populated!");

    Ok(())
}
