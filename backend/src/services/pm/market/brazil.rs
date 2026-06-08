//! Brazil market configuration — Lei do Inquilinato (Law 8.245/91).
//!
//! Covers: São Paulo, Rio de Janeiro, Fortaleza, and any other BR property.

use rust_decimal::Decimal;
use std::str::FromStr;
use crate::types::pm::{
    Jurisdiction, Currency, CreditIdField, TaxRate, StatuteRef,
    GuaranteeType, ConominioExpenseCategory,
};
use crate::services::pm::market::market_config::{
    MarketConfig, TenancyLaw, AntiDiscriminationLaw, StrRegulation, TaxEngine, CreditBureau,
};

// ── Expense classification rules ──────────────────────────────────────────────

/// A single expense classification rule for the condomínio split.
///
/// # Why a struct, not an enum or &[&str]
///
/// Keywords are **free-text substring patterns** matched against arbitrary
/// invoice descriptions. They are not domain discriminators (those are enums)
/// and not DB values (those have TryFrom/Display). An enum of keyword variants
/// would just wrap strings with no semantic gain — you'd call `.as_str()` on
/// every match arm.
///
/// A `KeywordRule` struct co-locates:
///   - the keyword itself
///   - the legal basis (Art. 22 vs Art. 23)
///   - the resulting classification
///
/// This makes each rule self-documenting, independently testable, and impossible
/// to accidentally add to the wrong obligation bucket.
#[derive(Debug)]
pub struct KeywordRule {
    /// Lowercase substring to search for in the expense description.
    pub keyword: &'static str,
    /// Legal citation explaining why this expense has this classification.
    pub legal_basis: &'static str,
    /// The expense obligation this keyword indicates.
    pub category: ConominioExpenseCategory,
}

/// All Lei do Inquilinato expense classification rules.
///
/// Art. 23 — Despesas Ordinárias (tenant obligation): day-to-day operating costs.
/// Art. 22 — Despesas Extraordinárias (landlord obligation): capital/structural costs.
///
/// Evaluated in order — first match wins.
static EXPENSE_RULES: &[KeywordRule] = &[
    // ── Art. 23: Despesas Ordinárias (tenant) ──────────────────────────────
    KeywordRule { keyword: "condomínio",         legal_basis: "Art. 23, §único",   category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "agua",               legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "água",               legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "gas",                legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "gás",                legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "luz",                legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "energia",            legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "limpeza",            legal_basis: "Art. 23, §único-a", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "zeladoria",          legal_basis: "Art. 23, §único-a", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "porteiro",           legal_basis: "Art. 23, §único-a", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "faxineiro",          legal_basis: "Art. 23, §único-a", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "segurança",          legal_basis: "Art. 23, §único-b", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "piscina",            legal_basis: "Art. 23, §único-c", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "academia",           legal_basis: "Art. 23, §único-c", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "elevador operação",  legal_basis: "Art. 23, §único-d", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "manutenção rotina",  legal_basis: "Art. 23, §único-e", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "jardinagem",         legal_basis: "Art. 23, §único-f", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "dedetização",        legal_basis: "Art. 23, §único-g", category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "lixo",               legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "coleta",             legal_basis: "Art. 23, III",      category: ConominioExpenseCategory::DespesasOrdinarias },
    KeywordRule { keyword: "reparo pequeno",     legal_basis: "Art. 23, §único-e", category: ConominioExpenseCategory::DespesasOrdinarias },

    // ── Art. 22: Despesas Extraordinárias (landlord) ───────────────────────
    KeywordRule { keyword: "fundo reserva",           legal_basis: "Art. 22, X",    category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "fundo de reserva",        legal_basis: "Art. 22, X",    category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "obras estruturais",       legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "reforma fachada",         legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "pintura fachada",         legal_basis: "Art. 22, IX",   category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "pintura áreas comuns",    legal_basis: "Art. 22, IX",   category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "elevador substituição",   legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "elevador reforma",        legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "impermeabilização",       legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "retrofitting",            legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "ampliação",               legal_basis: "Art. 22, VII",  category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "construção",              legal_basis: "Art. 22, VII",  category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "instalação",              legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "substituição equipamento",legal_basis: "Art. 22, VIII", category: ConominioExpenseCategory::DespesasExtraordinarias },
    KeywordRule { keyword: "modernização",            legal_basis: "Art. 22, VII",  category: ConominioExpenseCategory::DespesasExtraordinarias },
];

// ── Law: Lei do Inquilinato ───────────────────────────────────────────────────

pub struct LeiDoInquilinato;

static BR_GUARANTEES: &[GuaranteeType] = &[
    GuaranteeType::Fiador,
    GuaranteeType::SeguroFianca,
    GuaranteeType::Caucao,
    GuaranteeType::TituloCapitalizacao,
    GuaranteeType::None,
];

impl TenancyLaw for LeiDoInquilinato {
    fn statute(&self) -> StatuteRef {
        StatuteRef {
            code: "Lei 8.245/91",
            name: "Lei do Inquilinato",
            country: Jurisdiction::Br,
            url: Some("http://www.planalto.gov.br/ccivil_03/leis/l8245.htm"),
        }
    }

    /// Classify a building expense description against the `EXPENSE_RULES` table.
    ///
    /// Evaluation is a lowercase substring scan — first matching rule wins.
    /// Returns `None` if no rule matches (caller decides the fallback via
    /// `unclassified_expense_default()`).
    ///
    /// To inspect *why* a description was classified, use `classify_with_rule()`.
    fn classify_expense(&self, description: &str) -> Option<ConominioExpenseCategory> {
        self.classify_with_rule(description).map(|r| r.category.clone())
    }

    /// Conservative default: unclassified expenses fall on the tenant.
    /// Better than silently absorbing landlord obligations as platform defaults.
    fn unclassified_expense_default(&self) -> Option<ConominioExpenseCategory> {
        Some(ConominioExpenseCategory::DespesasOrdinarias)
    }

    fn max_deposit_months(&self) -> Option<u8> { Some(3) }    // Art. 38
    fn landlord_termination_notice_days(&self) -> u32 { 30 }  // Art. 46
    fn tenant_vacate_notice_days(&self) -> u32 { 30 }         // Art. 6

    fn allowed_guarantee_types(&self) -> &'static [GuaranteeType] {
        BR_GUARANTEES
    }
}

impl LeiDoInquilinato {
    /// Returns the matching `KeywordRule` (with its legal basis) for a description.
    ///
    /// Useful for:
    ///   - Audit logging (`rule.legal_basis` explains the classification)
    ///   - UI tooltips ("Classified as ordinária per Art. 23, III")
    ///   - Per-rule unit testing
    pub fn classify_with_rule<'a>(
        &self,
        description: &str,
    ) -> Option<&'a KeywordRule> {
        let lower = description.to_lowercase();
        EXPENSE_RULES.iter().find(|rule| lower.contains(rule.keyword))
    }
}

// ── Tax: Brazil IRRF ──────────────────────────────────────────────────────────

pub struct BrazilIrrf;

impl TaxEngine for BrazilIrrf {
    fn str_tax_rate(&self) -> Option<TaxRate> {
        None
    }

    fn rental_withholding_rate(&self) -> Option<TaxRate> {
        Some(TaxRate {
            rate: Decimal::from_str("0.15").unwrap(),
            remittance_currency: Currency::Brl,
            ota_collects: false,
            label: "IRRF — Imposto de Renda Retido na Fonte (15%)",
        })
    }

    fn remittance_currency(&self) -> Currency { Currency::Brl }
}

// ── Credit: Serasa Experian ───────────────────────────────────────────────────

pub struct SerasaBureau;

impl CreditBureau for SerasaBureau {
    fn name(&self) -> &'static str { "Serasa Experian" }
    fn applicant_id_field(&self) -> CreditIdField { CreditIdField::Cpf }
    fn minimum_score_auto_approve(&self) -> Option<i32> { Some(700) }
}

// ── Root: BrazilMarket ────────────────────────────────────────────────────────

pub struct BrazilMarket;

impl MarketConfig for BrazilMarket {
    fn jurisdiction(&self) -> Jurisdiction { Jurisdiction::Br }
    fn default_currency(&self) -> Currency { Currency::Brl }
    fn display_name(&self) -> &'static str { "Brazil" }

    fn tenancy_law(&self) -> Option<&dyn TenancyLaw> { Some(&LeiDoInquilinato) }
    fn anti_discrimination_law(&self) -> Option<&dyn AntiDiscriminationLaw> { None }
    fn str_regulation(&self) -> Option<&dyn StrRegulation> { None }

    fn tax_engine(&self) -> &dyn TaxEngine { &BrazilIrrf }
    fn credit_bureau(&self) -> &dyn CreditBureau { &SerasaBureau }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agua_is_ordinaria_with_legal_basis() {
        let law = LeiDoInquilinato;
        let rule = law.classify_with_rule("Conta de água e esgoto").unwrap();
        assert_eq!(rule.category, ConominioExpenseCategory::DespesasOrdinarias);
        assert_eq!(rule.legal_basis, "Art. 23, III");
        assert_eq!(rule.keyword, "água");
    }

    #[test]
    fn test_fundo_reserva_is_extraordinaria() {
        let law = LeiDoInquilinato;
        let rule = law.classify_with_rule("Fundo de Reserva — Dezembro").unwrap();
        assert_eq!(rule.category, ConominioExpenseCategory::DespesasExtraordinarias);
        assert_eq!(rule.legal_basis, "Art. 22, X");
    }

    #[test]
    fn test_unrecognized_returns_none() {
        let law = LeiDoInquilinato;
        assert!(law.classify_with_rule("Taxa administrativa XYZ").is_none());
    }

    #[test]
    fn test_unclassified_defaults_to_ordinaria() {
        let law = LeiDoInquilinato;
        assert_eq!(
            law.unclassified_expense_default(),
            Some(ConominioExpenseCategory::DespesasOrdinarias)
        );
    }

    #[test]
    fn test_all_rules_have_legal_basis() {
        // Every rule must document WHY it's classified the way it is.
        for rule in EXPENSE_RULES {
            assert!(
                !rule.legal_basis.is_empty(),
                "Rule '{}' is missing a legal_basis citation",
                rule.keyword
            );
        }
    }

    #[test]
    fn test_no_duplicate_keywords() {
        let mut seen = std::collections::HashSet::new();
        for rule in EXPENSE_RULES {
            assert!(
                seen.insert(rule.keyword),
                "Duplicate keyword '{}' in EXPENSE_RULES",
                rule.keyword
            );
        }
    }
}
