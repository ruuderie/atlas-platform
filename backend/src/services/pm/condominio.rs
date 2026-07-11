//! Folio — Condomínio Service (Brazilian Lei do Inquilinato expense classifier)
//!
//! Classifies building expenses into despesas ordinárias (tenant obligation)
//! vs. extraordinárias (landlord obligation) per Law 8.245/91, Art. 22–23.

use crate::types::pm::ConominioExpenseCategory;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConominioExpense {
    pub description: String,
    /// Amount in BRL centavos.
    pub amount_cents: i64,
    pub category: ConominioExpenseCategory,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConominioSplit {
    pub despesas_ordinarias_cents: i64,
    pub despesas_extraordinarias_cents: i64,
    pub total_cents: i64,
    pub expenses: Vec<ConominioExpense>,
}

impl ConominioSplit {
    pub fn from_expenses(expenses: Vec<ConominioExpense>) -> Self {
        let ordinarias: i64 = expenses
            .iter()
            .filter(|e| e.category == ConominioExpenseCategory::DespesasOrdinarias)
            .map(|e| e.amount_cents)
            .sum();

        let extraordinarias: i64 = expenses
            .iter()
            .filter(|e| e.category == ConominioExpenseCategory::DespesasExtraordinarias)
            .map(|e| e.amount_cents)
            .sum();

        Self {
            despesas_ordinarias_cents: ordinarias,
            despesas_extraordinarias_cents: extraordinarias,
            total_cents: ordinarias + extraordinarias,
            expenses,
        }
    }
}

// ── Keyword lists ─────────────────────────────────────────────────────────────

/// Day-to-day operating expenses — tenant obligation (Art. 23).
const ORDINARIAS_KEYWORDS: &[&str] = &[
    "condomínio",
    "agua",
    "água",
    "gas",
    "gás",
    "luz",
    "energia",
    "limpeza",
    "zeladoria",
    "porteiro",
    "faxineiro",
    "segurança",
    "piscina",
    "academia",
    "elevador operação",
    "manutenção rotina",
    "jardinagem",
    "dedetização",
    "lixo",
    "coleta",
    "reparo pequeno",
];

/// Capital / structural expenses — landlord obligation (Art. 22).
const EXTRAORDINARIAS_KEYWORDS: &[&str] = &[
    "fundo reserva",
    "fundo de reserva",
    "obras estruturais",
    "reforma fachada",
    "pintura fachada",
    "pintura áreas comuns",
    "elevador substituição",
    "elevador reforma",
    "impermeabilização",
    "retrofitting",
    "ampliação",
    "construção",
    "instalação",
    "substituição equipamento",
    "modernização",
];

// ── Service ───────────────────────────────────────────────────────────────────

pub struct ConominioService;

impl ConominioService {
    /// Auto-classify an expense description using canonical keyword lists.
    ///
    /// Returns `None` if the description matches neither list (manual review needed).
    /// Conservative default when ambiguous: caller should use `DespesasOrdinarias`
    /// to avoid improperly charging tenants.
    pub fn auto_classify(description: &str) -> Option<ConominioExpenseCategory> {
        let lower = description.to_lowercase();

        for kw in ORDINARIAS_KEYWORDS {
            if lower.contains(kw) {
                return Some(ConominioExpenseCategory::DespesasOrdinarias);
            }
        }
        for kw in EXTRAORDINARIAS_KEYWORDS {
            if lower.contains(kw) {
                return Some(ConominioExpenseCategory::DespesasExtraordinarias);
            }
        }
        None
    }

    /// Classify and compute a monthly condomínio split.
    ///
    /// Unclassified expenses default to `DespesasOrdinarias` (conservative).
    pub fn classify_and_split(raw_expenses: Vec<(String, i64, Option<String>)>) -> ConominioSplit {
        let expenses = raw_expenses
            .into_iter()
            .map(|(description, amount_cents, reference)| {
                let category = Self::auto_classify(&description)
                    .unwrap_or(ConominioExpenseCategory::DespesasOrdinarias);
                ConominioExpense {
                    description,
                    amount_cents,
                    category,
                    reference,
                }
            })
            .collect();

        ConominioSplit::from_expenses(expenses)
    }

    /// Serialize a split into the `atlas_ledger_splits.metadata` JSONB format.
    pub fn to_ledger_metadata(split: &ConominioSplit) -> serde_json::Value {
        serde_json::json!({
            "condominio_split": {
                "despesas_ordinarias_cents": split.despesas_ordinarias_cents,
                "despesas_extraordinarias_cents": split.despesas_extraordinarias_cents,
                "total_cents": split.total_cents,
                "jurisdiction": "BR",
            }
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_agua() {
        assert_eq!(
            ConominioService::auto_classify("Conta de água e esgoto"),
            Some(ConominioExpenseCategory::DespesasOrdinarias)
        );
    }

    #[test]
    fn test_classify_fundo_reserva() {
        assert_eq!(
            ConominioService::auto_classify("Fundo de Reserva - Dezembro"),
            Some(ConominioExpenseCategory::DespesasExtraordinarias)
        );
    }

    #[test]
    fn test_classify_unrecognized() {
        assert!(ConominioService::auto_classify("Taxa administrativa XYZ").is_none());
    }

    #[test]
    fn test_split_totals() {
        let split = ConominioService::classify_and_split(vec![
            ("Água".to_string(), 15_000, None),
            ("Limpeza".to_string(), 20_000, None),
            ("Fundo reserva".to_string(), 5_000, None),
        ]);
        assert_eq!(split.despesas_ordinarias_cents, 35_000);
        assert_eq!(split.despesas_extraordinarias_cents, 5_000);
        assert_eq!(split.total_cents, 40_000);
    }

    #[test]
    fn test_category_roundtrip() {
        // Enum → Display → TryFrom roundtrip
        for cat in [
            ConominioExpenseCategory::DespesasOrdinarias,
            ConominioExpenseCategory::DespesasExtraordinarias,
        ] {
            let s = cat.to_string();
            let parsed = ConominioExpenseCategory::try_from(s).unwrap();
            assert_eq!(cat, parsed);
        }
    }
}
