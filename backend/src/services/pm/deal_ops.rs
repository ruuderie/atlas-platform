//! Folio Deal Ops — unified Wholesaling + Creative Finance over G-15.
//!
//! Tracks share `atlas_opportunities` with typed `opportunity_type` + `status`
//! stage machines. Economics live in `financial_inputs` / `computed_outputs`.

use anyhow::{Result, anyhow};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::str::FromStr;
use uuid::Uuid;

use crate::services::pm::scorecard_provisioner::get_pm_template;
use crate::services::pm::wholesale::{MaoResult, WholesaleService};
use crate::services::scorecard_service::ScorecardService;
use crate::types::pm::{
    AcquisitionStructure, CreativeFinanceAcquireStage, CreativeFinanceDisposeStage, DealTrack,
    ExitMode, PmContractType, PmOpportunityType, SellerMotivation, WholesaleBuyerStage,
    WholesaleStage,
};

// ── Summary DTO (flattened for Folio UI) ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealSummary {
    pub id: Uuid,
    pub track: String,
    pub opportunity_type: String,
    pub name: String,
    pub status: String,
    pub property_address: String,
    pub arv_cents: Option<i64>,
    pub repair_cents: Option<i64>,
    pub offer_cents: Option<i64>,
    pub deal_amount_cents: Option<i64>,
    pub acquisition_structure: Option<String>,
    pub exit_mode: Option<String>,
    pub cya_required: Option<bool>,
    pub cya_signed: Option<bool>,
    pub title_clear: Option<bool>,
    pub asset_id: Option<Uuid>,
    pub currency: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureOfferInput {
    pub acquisition_structure: String,
    pub exit_mode: Option<String>,
    pub offer_cents: Option<i64>,
    pub cash_to_seller_cents: Option<i64>,
    pub seller_second_cents: Option<i64>,
    pub loan_balance_cents: Option<i64>,
    pub piti_cents: Option<i64>,
    pub ti_escrow_cents: Option<i64>,
    pub planned_sale_price_cents: Option<i64>,
    pub planned_rent_cents: Option<i64>,
    pub option_deposit_target_cents: Option<i64>,
    pub assignment_fee_cents: Option<i64>,
    pub repair_cents: Option<i64>,
    pub arv_cents: Option<i64>,
    pub asking_cents: Option<i64>,
    pub cya_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDealInput {
    pub track: DealTrack,
    pub address: String,
    pub arv_cents: Option<i64>,
    pub repair_cents: Option<i64>,
    pub asking_cents: Option<i64>,
    pub loan_balance_cents: Option<i64>,
    pub piti_cents: Option<i64>,
    pub sqft: Option<i64>,
    pub vacant: Option<bool>,
    pub listed: Option<bool>,
    pub seller_motivation: Option<String>,
    pub owner_user_id: Option<Uuid>,
    /// When creating a buyer-side opportunity.
    pub as_buyer: Option<bool>,
    pub buyer_fit: Option<String>,
    pub max_cash_cents: Option<i64>,
}

pub struct DealOpsService;

impl DealOpsService {
    pub fn calculate_mao(
        arv_cents: i64,
        repair_cents: i64,
        wholesale_fee_cents: i64,
        multiplier: Option<Decimal>,
        currency: Option<String>,
    ) -> MaoResult {
        let mut result = WholesaleService::calculate_mao(
            arv_cents,
            repair_cents,
            wholesale_fee_cents,
            multiplier,
            currency,
        );
        // Equity cushion: (ARV - MAO - repairs) / ARV when ARV > 0
        let _ = &mut result; // MaoResult may not have equity field yet — enriched in handler
        result
    }

    /// Equity cushion percent for UI (0–100).
    pub fn equity_cushion_pct(arv_cents: i64, mao_cents: i64, repair_cents: i64) -> f64 {
        if arv_cents <= 0 {
            return 0.0;
        }
        let cushion = arv_cents - mao_cents.max(0) - repair_cents.max(0);
        (cushion as f64 / arv_cents as f64) * 100.0
    }

    pub fn summarize(opp: &crate::entities::atlas_opportunity::Model) -> DealSummary {
        let inputs = opp.financial_inputs.as_ref();
        let address = inputs
            .and_then(|v| v.get("address"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let property_address = if address.is_empty() {
            opp.name.clone()
        } else {
            address
        };

        let track = match PmOpportunityType::try_from(opp.opportunity_type.clone()) {
            Ok(PmOpportunityType::WholesaleLead | PmOpportunityType::WholesaleBuyer) => {
                DealTrack::Wholesale.to_string()
            }
            Ok(
                PmOpportunityType::CreativeFinanceAcquisition
                | PmOpportunityType::CreativeFinanceDisposition,
            ) => DealTrack::CreativeFinance.to_string(),
            _ => "unknown".to_string(),
        };

        DealSummary {
            id: opp.id,
            track,
            opportunity_type: opp.opportunity_type.clone(),
            name: opp.name.clone(),
            status: opp.status.clone(),
            property_address,
            arv_cents: json_i64(inputs, "arv_cents"),
            repair_cents: json_i64(inputs, "repair_cents"),
            offer_cents: json_i64(inputs, "offer_cents")
                .or(opp.deal_amount_cents),
            deal_amount_cents: opp.deal_amount_cents,
            acquisition_structure: json_str(inputs, "acquisition_structure"),
            exit_mode: json_str(inputs, "exit_mode"),
            cya_required: json_bool(inputs, "cya_required"),
            cya_signed: json_bool(inputs, "cya_signed"),
            title_clear: json_bool(inputs, "title_clear"),
            asset_id: opp.asset_id,
            currency: opp.currency.clone(),
            created_at: opp.created_at,
        }
    }

    pub async fn list_deals(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        track: Option<DealTrack>,
    ) -> Result<Vec<DealSummary>> {
        let mut q = crate::entities::atlas_opportunity::Entity::find()
            .filter(crate::entities::atlas_opportunity::Column::TenantId.eq(tenant_id));

        if let Some(t) = track {
            let types: Vec<String> = match t {
                DealTrack::Wholesale => vec![
                    PmOpportunityType::WholesaleLead.to_string(),
                    PmOpportunityType::WholesaleBuyer.to_string(),
                ],
                DealTrack::CreativeFinance => vec![
                    PmOpportunityType::CreativeFinanceAcquisition.to_string(),
                    PmOpportunityType::CreativeFinanceDisposition.to_string(),
                ],
            };
            q = q.filter(
                crate::entities::atlas_opportunity::Column::OpportunityType.is_in(types),
            );
        } else {
            q = q.filter(
                crate::entities::atlas_opportunity::Column::OpportunityType.is_in(vec![
                    PmOpportunityType::WholesaleLead.to_string(),
                    PmOpportunityType::WholesaleBuyer.to_string(),
                    PmOpportunityType::CreativeFinanceAcquisition.to_string(),
                    PmOpportunityType::CreativeFinanceDisposition.to_string(),
                ]),
            );
        }

        let rows = q.all(db).await?;
        Ok(rows.iter().map(Self::summarize).collect())
    }

    pub async fn create_deal(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateDealInput,
    ) -> Result<Uuid> {
        let as_buyer = input.as_buyer.unwrap_or(false);
        let (opp_type, status, name_prefix) = match (input.track, as_buyer) {
            (DealTrack::Wholesale, false) => (
                PmOpportunityType::WholesaleLead,
                WholesaleStage::New.to_string(),
                "Wholesale",
            ),
            (DealTrack::Wholesale, true) => (
                PmOpportunityType::WholesaleBuyer,
                WholesaleBuyerStage::BuyerLead.to_string(),
                "Cash Buyer",
            ),
            (DealTrack::CreativeFinance, false) => (
                PmOpportunityType::CreativeFinanceAcquisition,
                CreativeFinanceAcquireStage::New.to_string(),
                "CF Acquire",
            ),
            (DealTrack::CreativeFinance, true) => (
                PmOpportunityType::CreativeFinanceDisposition,
                CreativeFinanceDisposeStage::BuyerLead.to_string(),
                "Tenant Buyer",
            ),
        };

        let motivation = input
            .seller_motivation
            .as_ref()
            .and_then(|s| SellerMotivation::try_from(s.clone()).ok())
            .unwrap_or(SellerMotivation::Other);

        let arv = input.arv_cents.unwrap_or(0);
        let repair = input.repair_cents.unwrap_or(0);
        let fee = 5_000_00i64;
        let mao = if input.track == DealTrack::Wholesale && !as_buyer {
            Some(WholesaleService::calculate_mao(arv, repair, fee, None, None))
        } else {
            None
        };

        let mut financial_inputs = json!({
            "address": input.address,
            "arv_cents": arv,
            "repair_cents": repair,
            "asking_cents": input.asking_cents,
            "loan_balance_cents": input.loan_balance_cents,
            "piti_cents": input.piti_cents,
            "sqft": input.sqft,
            "vacant": input.vacant,
            "listed": input.listed,
            "motivation": motivation.to_string(),
            "track": input.track.to_string(),
        });

        if let Some(ref m) = mao {
            financial_inputs["multiplier"] = json!("0.70");
            financial_inputs["wholesale_fee_cents"] = json!(fee);
            financial_inputs["offer_cents"] = json!(m.mao_cents.max(0));
            financial_inputs["acquisition_structure"] =
                json!(AcquisitionStructure::AllCashMao.to_string());
            financial_inputs["exit_mode"] = json!(ExitMode::WholesaleAssignment.to_string());
            financial_inputs["title_search_ordered"] = json!(false);
            financial_inputs["title_clear"] = json!(false);
        }

        if input.track == DealTrack::CreativeFinance && !as_buyer {
            financial_inputs["cya_required"] = json!(true);
            financial_inputs["cya_signed"] = json!(false);
            financial_inputs["exit_mode"] = json!(ExitMode::LeaseOption.to_string());
        }

        if as_buyer {
            if let Some(fit) = &input.buyer_fit {
                financial_inputs["buyer_fit"] = json!(fit);
            }
            if let Some(cash) = input.max_cash_cents {
                financial_inputs["max_cash_cents"] = json!(cash);
            }
        }

        let computed_outputs = if let Some(ref m) = mao {
            json!({
                "mao_cents": m.mao_cents,
                "is_viable": m.is_viable,
                "equity_cushion_pct": Self::equity_cushion_pct(arv, m.mao_cents, repair),
            })
        } else {
            json!({})
        };

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let model = crate::entities::atlas_opportunity::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            opportunity_type: Set(opp_type.to_string()),
            name: Set(format!("{name_prefix} — {}", input.address)),
            status: Set(status),
            deal_amount_cents: Set(mao.map(|m| m.mao_cents.max(0))),
            currency: Set("USD".to_string()),
            owner_user_id: Set(input.owner_user_id),
            financial_inputs: Set(Some(financial_inputs)),
            computed_outputs: Set(Some(computed_outputs)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;

        if matches!(opp_type, PmOpportunityType::WholesaleLead) {
            let _ = provision_lqa(db, tenant_id, id).await;
        }

        tracing::info!(
            opportunity_id = %id,
            %tenant_id,
            track = %input.track,
            "DealOpsService: deal created"
        );
        Ok(id)
    }

    pub async fn advance_stage(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        new_stage: &str,
    ) -> Result<()> {
        let opp = load_opp(db, tenant_id, opportunity_id).await?;
        let opp_type = PmOpportunityType::try_from(opp.opportunity_type.clone())
            .map_err(|e| anyhow!(e))?;

        match opp_type {
            PmOpportunityType::WholesaleLead => {
                let current = WholesaleStage::try_from(opp.status.clone()).map_err(|e| anyhow!(e))?;
                if current.is_terminal() {
                    return Err(anyhow!("Cannot advance terminal stage {}", opp.status));
                }
                let _ = WholesaleStage::try_from(new_stage.to_string()).map_err(|e| anyhow!(e))?;
            }
            PmOpportunityType::WholesaleBuyer => {
                let current =
                    WholesaleBuyerStage::try_from(opp.status.clone()).map_err(|e| anyhow!(e))?;
                if current.is_terminal() {
                    return Err(anyhow!("Cannot advance terminal stage {}", opp.status));
                }
                let _ =
                    WholesaleBuyerStage::try_from(new_stage.to_string()).map_err(|e| anyhow!(e))?;
            }
            PmOpportunityType::CreativeFinanceAcquisition => {
                let current = CreativeFinanceAcquireStage::try_from(opp.status.clone())
                    .map_err(|e| anyhow!(e))?;
                if current.is_terminal() {
                    return Err(anyhow!("Cannot advance terminal stage {}", opp.status));
                }
                let _ = CreativeFinanceAcquireStage::try_from(new_stage.to_string())
                    .map_err(|e| anyhow!(e))?;
            }
            PmOpportunityType::CreativeFinanceDisposition => {
                let current = CreativeFinanceDisposeStage::try_from(opp.status.clone())
                    .map_err(|e| anyhow!(e))?;
                if current.is_terminal() {
                    return Err(anyhow!("Cannot advance terminal stage {}", opp.status));
                }
                let _ = CreativeFinanceDisposeStage::try_from(new_stage.to_string())
                    .map_err(|e| anyhow!(e))?;
            }
            _ => return Err(anyhow!("Opportunity type not a Deal Ops deal")),
        }

        let mut active = opp.into_active_model();
        active.status = Set(new_stage.to_string());
        active.update(db).await?;
        Ok(())
    }

    pub async fn structure_offer(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        input: StructureOfferInput,
    ) -> Result<DealSummary> {
        let opp = load_opp(db, tenant_id, opportunity_id).await?;
        let structure = AcquisitionStructure::try_from(input.acquisition_structure.clone())
            .map_err(|e| anyhow!(e))?;
        let exit = input
            .exit_mode
            .as_ref()
            .and_then(|s| ExitMode::try_from(s.clone()).ok());

        let mut inputs = opp
            .financial_inputs
            .clone()
            .unwrap_or_else(|| json!({}));
        let obj = inputs.as_object_mut().ok_or_else(|| anyhow!("invalid financial_inputs"))?;

        obj.insert(
            "acquisition_structure".into(),
            json!(structure.to_string()),
        );
        if let Some(e) = &exit {
            obj.insert("exit_mode".into(), json!(e.to_string()));
        }
        merge_opt_i64(obj, "offer_cents", input.offer_cents);
        merge_opt_i64(obj, "cash_to_seller_cents", input.cash_to_seller_cents);
        merge_opt_i64(obj, "seller_second_cents", input.seller_second_cents);
        merge_opt_i64(obj, "loan_balance_cents", input.loan_balance_cents);
        merge_opt_i64(obj, "piti_cents", input.piti_cents);
        merge_opt_i64(obj, "ti_escrow_cents", input.ti_escrow_cents);
        merge_opt_i64(obj, "planned_sale_price_cents", input.planned_sale_price_cents);
        merge_opt_i64(obj, "planned_rent_cents", input.planned_rent_cents);
        merge_opt_i64(
            obj,
            "option_deposit_target_cents",
            input.option_deposit_target_cents,
        );
        merge_opt_i64(obj, "assignment_fee_cents", input.assignment_fee_cents);
        merge_opt_i64(obj, "repair_cents", input.repair_cents);
        merge_opt_i64(obj, "arv_cents", input.arv_cents);
        merge_opt_i64(obj, "asking_cents", input.asking_cents);

        let cya_required = input.cya_required.unwrap_or(matches!(
            structure,
            AcquisitionStructure::SubjectToFreeEquity
                | AcquisitionStructure::SubjectToCashEquity
                | AcquisitionStructure::SubjectToDeferredEquity
                | AcquisitionStructure::SubjectToSellerSecond
        ));
        obj.insert("cya_required".into(), json!(cya_required));
        if cya_required && !obj.contains_key("cya_signed") {
            obj.insert("cya_signed".into(), json!(false));
        }

        let computed = compute_structure_outputs(&inputs, &structure);

        let opp_type = opp.opportunity_type.clone();
        let opp_status = opp.status.clone();
        let mut active = opp.into_active_model();
        active.financial_inputs = Set(Some(inputs.clone()));
        active.computed_outputs = Set(Some(computed));
        if let Some(offer) = input.offer_cents.or(json_i64(Some(&inputs), "offer_cents")) {
            active.deal_amount_cents = Set(Some(offer.max(0)));
        }

        // Advance CF to offer_structured when structuring
        if opp_type == PmOpportunityType::CreativeFinanceAcquisition.to_string()
            && (opp_status == CreativeFinanceAcquireStage::New.to_string()
                || opp_status == CreativeFinanceAcquireStage::Prescreened.to_string())
        {
            active.status = Set(CreativeFinanceAcquireStage::OfferStructured.to_string());
        }
        if opp_type == PmOpportunityType::WholesaleLead.to_string()
            && (opp_status == WholesaleStage::New.to_string()
                || opp_status == WholesaleStage::Prescreened.to_string()
                || opp_status == WholesaleStage::Qualified.to_string())
        {
            active.status = Set(WholesaleStage::OfferOut.to_string());
        }

        let updated = active.update(db).await?;
        Ok(Self::summarize(&updated))
    }

    pub async fn set_cya_signed(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        signed: bool,
    ) -> Result<DealSummary> {
        let opp = load_opp(db, tenant_id, opportunity_id).await?;
        let mut inputs = opp.financial_inputs.clone().unwrap_or_else(|| json!({}));
        let obj = inputs
            .as_object_mut()
            .ok_or_else(|| anyhow!("invalid financial_inputs"))?;
        obj.insert("cya_signed".into(), json!(signed));
        let mut active = opp.into_active_model();
        active.financial_inputs = Set(Some(inputs));
        if signed {
            active.status = Set(CreativeFinanceAcquireStage::CyaClosing.to_string());
        }
        let updated = active.update(db).await?;
        Ok(Self::summarize(&updated))
    }

    pub async fn set_title_flags(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        title_search_ordered: Option<bool>,
        title_clear: Option<bool>,
    ) -> Result<DealSummary> {
        let opp = load_opp(db, tenant_id, opportunity_id).await?;
        let mut inputs = opp.financial_inputs.clone().unwrap_or_else(|| json!({}));
        let obj = inputs
            .as_object_mut()
            .ok_or_else(|| anyhow!("invalid financial_inputs"))?;
        if let Some(v) = title_search_ordered {
            obj.insert("title_search_ordered".into(), json!(v));
        }
        if let Some(v) = title_clear {
            obj.insert("title_clear".into(), json!(v));
            if v {
                let mut active = opp.clone().into_active_model();
                active.financial_inputs = Set(Some(inputs.clone()));
                active.status = Set(WholesaleStage::TitleClear.to_string());
                let updated = active.update(db).await?;
                return Ok(Self::summarize(&updated));
            }
        }
        let mut active = opp.into_active_model();
        active.financial_inputs = Set(Some(inputs));
        let updated = active.update(db).await?;
        Ok(Self::summarize(&updated))
    }

    /// Convert CF acquisition to G-11 contract when CYA satisfied.
    pub async fn convert_acquisition(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        counterparty_user_id: Option<Uuid>,
    ) -> Result<(Uuid, Uuid)> {
        let opp = load_opp(db, tenant_id, opportunity_id).await?;
        if opp.opportunity_type != PmOpportunityType::CreativeFinanceAcquisition.to_string() {
            return Err(anyhow!("convert_acquisition only for CF acquisitions"));
        }
        let inputs = opp.financial_inputs.clone().unwrap_or_else(|| json!({}));
        let cya_required = json_bool(Some(&inputs), "cya_required").unwrap_or(false);
        let cya_signed = json_bool(Some(&inputs), "cya_signed").unwrap_or(false);
        if cya_required && !cya_signed {
            return Err(anyhow!("CYA Letter of Agreement required before convert"));
        }

        let structure = json_str(Some(&inputs), "acquisition_structure")
            .and_then(|s| AcquisitionStructure::try_from(s).ok())
            .unwrap_or(AcquisitionStructure::SubjectToFreeEquity);

        let contract_type = match structure {
            AcquisitionStructure::PurchaseOption => PmContractType::PurchaseOption,
            AcquisitionStructure::SellerFinanceWrap => PmContractType::Wrap,
            AcquisitionStructure::AllCashMao => PmContractType::WholesalePurchase,
            _ => PmContractType::SubjectToPurchase,
        };

        let address = json_str(Some(&inputs), "address").unwrap_or_else(|| opp.name.clone());
        let asset_id = if let Some(aid) = opp.asset_id {
            aid
        } else {
            create_placeholder_asset(db, tenant_id, &address).await?
        };

        let contract_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let terms = json!({
            "acquisition_structure": structure.to_string(),
            "exit_mode": json_str(Some(&inputs), "exit_mode"),
            "loan_balance_cents": json_i64(Some(&inputs), "loan_balance_cents"),
            "piti_cents": json_i64(Some(&inputs), "piti_cents"),
            "ti_escrow_cents": json_i64(Some(&inputs), "ti_escrow_cents"),
            "cya_signed": cya_signed,
            "source_opportunity_id": opportunity_id,
        });

        let contract = crate::entities::atlas_contract::ActiveModel {
            id: Set(contract_id),
            tenant_id: Set(tenant_id),
            contract_type: Set(contract_type.to_string()),
            asset_id: Set(Some(asset_id)),
            counterparty_user_id: Set(counterparty_user_id),
            status: Set("active".to_string()),
            start_date: Set(now.date_naive()),
            end_date: Set(None),
            auto_renew: Set(false),
            currency: Set("USD".to_string()),
            billing_interval: Set("once".to_string()),
            terms_metadata: Set(Some(terms)),
            created_at: Set(now),
            ..Default::default()
        };
        contract.insert(db).await?;

        let mut active = opp.into_active_model();
        active.asset_id = Set(Some(asset_id));
        active.status = Set(CreativeFinanceAcquireStage::OwnedOrOptioned.to_string());
        active.won_at = Set(Some(now));
        active.update(db).await?;

        Ok((asset_id, contract_id))
    }

    /// Install lease-option (or wrap) for a CF disposition linked to an asset.
    pub async fn install_lease_option(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        disposition_id: Uuid,
        asset_id: Uuid,
        counterparty_user_id: Uuid,
        option_price_cents: i64,
        option_deposit_cents: i64,
        monthly_rent_cents: i64,
        dpap_extra_cents: Option<i64>,
        dpap_price_credit_cents: Option<i64>,
        term_end: Option<chrono::NaiveDate>,
    ) -> Result<Uuid> {
        let opp = load_opp(db, tenant_id, disposition_id).await?;
        if opp.opportunity_type != PmOpportunityType::CreativeFinanceDisposition.to_string() {
            return Err(anyhow!("install_lease_option requires CF disposition"));
        }

        let contract_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let terms = json!({
            "option_price_cents": option_price_cents,
            "option_deposit_cents": option_deposit_cents,
            "dpap_extra_cents": dpap_extra_cents.unwrap_or(0),
            "dpap_price_credit_cents": dpap_price_credit_cents.unwrap_or(0),
            "dpap_skips_allowed": 2,
            "dpap_applied_cents": 0,
            "price_credits_cents": 0,
            "non_refundable": true,
            "repair_shift_day": 30,
            "source_opportunity_id": disposition_id,
            "term_end": term_end.map(|d| d.to_string()),
        });

        let contract = crate::entities::atlas_contract::ActiveModel {
            id: Set(contract_id),
            tenant_id: Set(tenant_id),
            contract_type: Set(PmContractType::LeaseOption.to_string()),
            asset_id: Set(Some(asset_id)),
            counterparty_user_id: Set(Some(counterparty_user_id)),
            status: Set("active".to_string()),
            start_date: Set(now.date_naive()),
            end_date: Set(term_end),
            auto_renew: Set(false),
            recurring_amount_cents: Set(Some(monthly_rent_cents)),
            currency: Set("USD".to_string()),
            billing_interval: Set("monthly".to_string()),
            terms_metadata: Set(Some(terms)),
            created_at: Set(now),
            ..Default::default()
        };
        contract.insert(db).await?;

        let mut active = opp.into_active_model();
        active.asset_id = Set(Some(asset_id));
        active.counterparty_user_id = Set(Some(counterparty_user_id));
        active.status = Set(CreativeFinanceDisposeStage::Installed.to_string());
        active.deal_amount_cents = Set(Some(option_price_cents));
        active.won_at = Set(Some(now));
        active.update(db).await?;

        Ok(contract_id)
    }

    /// Create assignment contract for wholesale under-contract deal.
    pub async fn create_assignment(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        assignee_user_id: Option<Uuid>,
        assignment_fee_cents: i64,
        deposit_cents: i64,
        expires_days: i32,
    ) -> Result<Uuid> {
        let opp = load_opp(db, tenant_id, opportunity_id).await?;
        if opp.opportunity_type != PmOpportunityType::WholesaleLead.to_string() {
            return Err(anyhow!("assignment requires wholesale_lead"));
        }
        let title_clear = json_bool(opp.financial_inputs.as_ref(), "title_clear").unwrap_or(false);
        if !title_clear {
            return Err(anyhow!("Title must be clear before assignment"));
        }

        let contract_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let expires = now.date_naive() + chrono::Duration::days(expires_days as i64);
        let terms = json!({
            "assignment_fee_cents": assignment_fee_cents,
            "deposit_cents": deposit_cents,
            "deposit_non_refundable_except_title": true,
            "expires_on": expires.to_string(),
            "source_opportunity_id": opportunity_id,
        });

        let contract = crate::entities::atlas_contract::ActiveModel {
            id: Set(contract_id),
            tenant_id: Set(tenant_id),
            contract_type: Set(PmContractType::Assignment.to_string()),
            asset_id: Set(opp.asset_id),
            counterparty_user_id: Set(assignee_user_id),
            status: Set("active".to_string()),
            start_date: Set(now.date_naive()),
            end_date: Set(Some(expires)),
            auto_renew: Set(false),
            currency: Set("USD".to_string()),
            billing_interval: Set("once".to_string()),
            terms_metadata: Set(Some(terms)),
            created_at: Set(now),
            ..Default::default()
        };
        contract.insert(db).await?;

        let mut inputs = opp.financial_inputs.clone().unwrap_or_else(|| json!({}));
        if let Some(obj) = inputs.as_object_mut() {
            obj.insert("assignment_fee_cents".into(), json!(assignment_fee_cents));
            obj.insert("assignment_contract_id".into(), json!(contract_id));
        }
        let mut active = opp.into_active_model();
        active.financial_inputs = Set(Some(inputs));
        active.status = Set(WholesaleStage::AssignedOrClosed.to_string());
        active.deal_amount_cents = Set(Some(assignment_fee_cents));
        active.won_at = Set(Some(now));
        active.update(db).await?;

        Ok(contract_id)
    }

    pub async fn convert_wholesale_to_cf(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
    ) -> Result<DealSummary> {
        let opp = load_opp(db, tenant_id, opportunity_id).await?;
        if opp.opportunity_type != PmOpportunityType::WholesaleLead.to_string() {
            return Err(anyhow!("only wholesale_lead can convert to CF"));
        }
        let mut inputs = opp.financial_inputs.clone().unwrap_or_else(|| json!({}));
        if let Some(obj) = inputs.as_object_mut() {
            obj.insert("track".into(), json!(DealTrack::CreativeFinance.to_string()));
            obj.insert("cya_required".into(), json!(true));
            obj.insert("cya_signed".into(), json!(false));
            obj.insert(
                "exit_mode".into(),
                json!(ExitMode::LeaseOption.to_string()),
            );
        }
        let mut active = opp.into_active_model();
        active.opportunity_type =
            Set(PmOpportunityType::CreativeFinanceAcquisition.to_string());
        active.status = Set(CreativeFinanceAcquireStage::Prescreened.to_string());
        active.financial_inputs = Set(Some(inputs));
        // Mark prior wholesale status for audit via lost_reason
        active.lost_reason = Set(Some("converted_to_cf".to_string()));
        let updated = active.update(db).await?;
        // Also set wholesale terminal on a note — opportunity is now CF
        let _ = WholesaleStage::ConvertedToCf;
        Ok(Self::summarize(&updated))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn json_i64(v: Option<&Value>, key: &str) -> Option<i64> {
    v.and_then(|j| j.get(key)).and_then(|x| x.as_i64())
}

fn json_str(v: Option<&Value>, key: &str) -> Option<String> {
    v.and_then(|j| j.get(key))
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
}

fn json_bool(v: Option<&Value>, key: &str) -> Option<bool> {
    v.and_then(|j| j.get(key)).and_then(|x| x.as_bool())
}

fn merge_opt_i64(
    obj: &mut serde_json::Map<String, Value>,
    key: &str,
    val: Option<i64>,
) {
    if let Some(v) = val {
        obj.insert(key.into(), json!(v));
    }
}

fn compute_structure_outputs(inputs: &Value, structure: &AcquisitionStructure) -> Value {
    let arv = json_i64(Some(inputs), "arv_cents").unwrap_or(0);
    let repair = json_i64(Some(inputs), "repair_cents").unwrap_or(0);
    let offer = json_i64(Some(inputs), "offer_cents").unwrap_or(0);
    let loan = json_i64(Some(inputs), "loan_balance_cents").unwrap_or(0);
    let piti = json_i64(Some(inputs), "piti_cents").unwrap_or(0);
    let rent = json_i64(Some(inputs), "planned_rent_cents").unwrap_or(0);
    let deposit = json_i64(Some(inputs), "option_deposit_target_cents").unwrap_or(0);
    let cash_to_seller = json_i64(Some(inputs), "cash_to_seller_cents").unwrap_or(0);
    let sale = json_i64(Some(inputs), "planned_sale_price_cents").unwrap_or(0);
    let assign_fee = json_i64(Some(inputs), "assignment_fee_cents").unwrap_or(0);

    let monthly_spread = if rent > 0 && piti > 0 {
        rent - piti
    } else {
        0
    };
    let deposit_profit = deposit.saturating_sub(cash_to_seller);
    let backend_equity = if sale > 0 {
        sale - loan.max(offer)
    } else {
        0
    };

    let fee = json_i64(Some(inputs), "wholesale_fee_cents").unwrap_or(5_000_00);
    let mult = inputs
        .get("multiplier")
        .and_then(|v| v.as_str())
        .and_then(|s| Decimal::from_str(s).ok())
        .unwrap_or_else(|| Decimal::from_str("0.70").unwrap());
    let mao = if matches!(structure, AcquisitionStructure::AllCashMao) && arv > 0 {
        let m = WholesaleService::calculate_mao(arv, repair, fee, Some(mult), None);
        Some(m)
    } else {
        None
    };

    json!({
        "monthly_spread_cents": monthly_spread,
        "deposit_profit_cents": deposit_profit,
        "backend_equity_cents": backend_equity,
        "assignment_fee_cents": assign_fee,
        "mao_cents": mao.as_ref().map(|m| m.mao_cents),
        "is_viable": mao.as_ref().map(|m| m.is_viable).unwrap_or(true),
        "equity_cushion_pct": mao.as_ref().map(|m| DealOpsService::equity_cushion_pct(arv, m.mao_cents, repair)),
        "acquisition_structure": structure.to_string(),
    })
}

async fn load_opp(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    opportunity_id: Uuid,
) -> Result<crate::entities::atlas_opportunity::Model> {
    let opp = crate::entities::atlas_opportunity::Entity::find_by_id(opportunity_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Opportunity {opportunity_id} not found"))?;
    if opp.tenant_id != tenant_id {
        return Err(anyhow!(
            "Opportunity {opportunity_id} not found for tenant {tenant_id}"
        ));
    }
    Ok(opp)
}

async fn provision_lqa(db: &DatabaseConnection, tenant_id: Uuid, id: Uuid) -> Result<()> {
    match get_pm_template(db, tenant_id, "Lead Quality Assessment").await {
        Ok(template) => {
            let _ = ScorecardService::get_or_create(
                db,
                tenant_id,
                template.id,
                "atlas_opportunity",
                id,
            )
            .await;
        }
        Err(_) => {}
    }
    Ok(())
}

async fn create_placeholder_asset(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    address: &str,
) -> Result<Uuid> {
    use crate::entities::atlas_asset;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let model = atlas_asset::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        asset_type: Set("real_estate_property".to_string()),
        name: Set(address.to_string()),
        status: Set("active".to_string()),
        address_line_1: Set(Some(address.to_string())),
        created_at: Set(now),
        ..Default::default()
    };
    model.insert(db).await?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equity_cushion() {
        let pct = DealOpsService::equity_cushion_pct(150_000_00, 80_000_00, 20_000_00);
        assert!((pct - 33.333).abs() < 0.1);
    }

    #[test]
    fn test_structure_outputs_spread() {
        let inputs = json!({
            "planned_rent_cents": 175000,
            "piti_cents": 133315,
            "option_deposit_target_cents": 1000000,
            "cash_to_seller_cents": 100000,
            "planned_sale_price_cents": 23490000,
            "loan_balance_cents": 19800000,
        });
        let out = compute_structure_outputs(&inputs, &AcquisitionStructure::SubjectToFreeEquity);
        assert_eq!(out["monthly_spread_cents"], 41685);
        assert_eq!(out["deposit_profit_cents"], 900000);
    }

    #[test]
    fn test_wholesale_stage_aliases() {
        assert_eq!(
            WholesaleStage::try_from("lead".to_string()).unwrap(),
            WholesaleStage::New
        );
        assert_eq!(
            WholesaleStage::Qualified.canonical(),
            WholesaleStage::Prescreened
        );
    }
}
