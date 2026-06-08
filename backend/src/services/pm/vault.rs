//! Folio — Vault Service (PM document taxonomy + atlas_documents G-14 wrapper)
//!
//! PM-specific wrapper over G-14 `atlas_documents` + `attachments`.
//! Document categories are typed — no raw strings.
//!
//! # Write path: `register_document()`
//!
//! 1. Insert `attachments` row with `upload_status = 'pending_upload'`
//!    and the caller-supplied `r2_key` (the file is already in R2 at this point).
//! 2. Insert `atlas_documents` row linking to the attachment with PM taxonomy.
//! 3. Return the `atlas_documents.id`.
//!
//! # Read path (Phase 6 — presigned URL generation)
//!
//! A separate `presign_upload_url()` method will generate a Cloudflare R2 presigned
//! PUT URL before the client uploads. After upload completes, the client calls
//! `register_document()` with the confirmed `r2_key`.
//!
//! # Entity field map (`atlas_documents`)
//!   `app_namespace`       → "folio"
//!   `document_category`   → `PmDocumentType.to_string()`
//!   `related_entity_type` → caller-supplied (e.g. "atlas_contracts", "atlas_assets")
//!   `related_entity_id`   → entity FK

use anyhow::Result;
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt;

/// PM document categories.
///
/// Stored in `atlas_documents.document_category`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PmDocumentType {
    /// Signed lease agreement.
    LeaseAgreement,
    /// Landlord or tenant identification.
    IdDocument,
    /// Vendor contractor license.
    ContractorLicense,
    /// STR operating permit.
    StrPermit,
    /// Inspection report (move-in, move-out, annual).
    InspectionReport,
    /// Property insurance policy.
    InsurancePolicy,
    /// Title deed or certificate of title.
    TitleDeed,
    /// Condomínio monthly statement (BR).
    ConominioStatement,
    /// Maintenance receipt or work order.
    MaintenanceReceipt,
    /// Security deposit receipt.
    SecurityDepositReceipt,
}

impl fmt::Display for PmDocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::LeaseAgreement        => "lease_agreement",
            Self::IdDocument            => "id_document",
            Self::ContractorLicense     => "contractor_license",
            Self::StrPermit             => "str_permit",
            Self::InspectionReport      => "inspection_report",
            Self::InsurancePolicy       => "insurance_policy",
            Self::TitleDeed             => "title_deed",
            Self::ConominioStatement    => "condominio_statement",
            Self::MaintenanceReceipt    => "maintenance_receipt",
            Self::SecurityDepositReceipt => "security_deposit_receipt",
        })
    }
}

impl TryFrom<String> for PmDocumentType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "lease_agreement"          => Ok(Self::LeaseAgreement),
            "id_document"              => Ok(Self::IdDocument),
            "contractor_license"       => Ok(Self::ContractorLicense),
            "str_permit"               => Ok(Self::StrPermit),
            "inspection_report"        => Ok(Self::InspectionReport),
            "insurance_policy"         => Ok(Self::InsurancePolicy),
            "title_deed"               => Ok(Self::TitleDeed),
            "condominio_statement"     => Ok(Self::ConominioStatement),
            "maintenance_receipt"      => Ok(Self::MaintenanceReceipt),
            "security_deposit_receipt" => Ok(Self::SecurityDepositReceipt),
            other => Err(format!("unknown PmDocumentType: '{other}'")),
        }
    }
}

pub struct VaultService;

impl VaultService {
    /// Register a document in `atlas_documents` with the PM taxonomy.
    ///
    /// # Arguments
    /// - `entity_type`  — The related entity (e.g. `"atlas_contracts"`, `"atlas_assets"`)
    /// - `entity_id`    — The FK to the related entity
    /// - `doc_type`     — Typed document category (stored in `document_category`)
    /// - `r2_key`       — The R2 object key (file must already be uploaded to R2)
    /// - `mime_type`    — MIME type of the document (e.g. `"application/pdf"`)
    /// - `size_bytes`   — File size in bytes (optional; helps UI show file info)
    ///
    /// Returns the `atlas_documents.id`.
    pub async fn register_document(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        doc_type: PmDocumentType,
        r2_key: &str,
    ) -> Result<Uuid> {
        Self::register_document_full(
            db, tenant_id, entity_type, entity_id, doc_type, r2_key,
            "application/octet-stream", None,
        ).await
    }

    /// Full document registration with MIME type and size.
    pub async fn register_document_full(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        doc_type: PmDocumentType,
        r2_key: &str,
        mime_type: &str,
        size_bytes: Option<i64>,
    ) -> Result<Uuid> {
        use sea_orm::{Set, ActiveModelTrait};
        use chrono::Utc;
        use anyhow::anyhow;

        let now = Utc::now();

        // ── 1. Insert attachments row ─────────────────────────────────────────
        // The R2 key is the permanent file location. upload_status = 'complete'
        // because register_document is called after the file is already in R2.
        let attachment_id = Uuid::new_v4();
        let r2_url = format!("r2://{}", r2_key); // internal R2 reference URL

        let attachment = crate::entities::attachment::ActiveModel {
            id: Set(attachment_id),
            r2_key: Set(Some(r2_key.to_string())),
            url: Set(r2_url),
            mime_type: Set(mime_type.to_string()),
            upload_status: Set(Some("complete".to_string())),
            size_in_bytes: Set(size_bytes),
            created_at: Set(now),
            updated_at: Set(now),
            // Non-required fields default to None
            feed_item_id: Set(None),
            title: Set(None),
            duration_in_seconds: Set(None),
            access_level: Set(Some("private".to_string())),
            r2_bucket: Set(None), // default bucket configured at infra level
            checksum_sha256: Set(None),
        };

        attachment.insert(db).await.map_err(|e| {
            anyhow!("VaultService: attachment insert failed for tenant {tenant_id}: {e}")
        })?;

        // ── 2. Insert atlas_documents row ─────────────────────────────────────
        let document_id = Uuid::new_v4();
        let doc_category = doc_type.to_string();

        let document = crate::entities::atlas_document::ActiveModel {
            id: Set(document_id),
            tenant_id: Set(tenant_id),
            attachment_id: Set(attachment_id),
            share_token_id: Set(None),
            document_category: Set(doc_category.clone()),
            app_namespace: Set("folio".to_string()),
            related_entity_type: Set(Some(entity_type.to_string())),
            related_entity_id: Set(Some(entity_id)),
            is_counterparty_visible: Set(false), // landlord controls visibility
            requires_signature: Set(false),
            is_signed: Set(false),
            signed_at: Set(None),
            signed_by_user_id: Set(None),
            signature_blob: Set(None),
            version_number: Set(1),
            supersedes_document_id: Set(None),
            created_at: Set(now),
        };

        document.insert(db).await.map_err(|e| {
            anyhow!("VaultService: atlas_document insert failed for tenant {tenant_id}: {e}")
        })?;

        tracing::info!(
            document_id = %document_id,
            attachment_id = %attachment_id,
            %tenant_id,
            entity_type,
            entity_id = %entity_id,
            doc_category,
            r2_key,
            "VaultService: document registered"
        );

        Ok(document_id)
    }
}
