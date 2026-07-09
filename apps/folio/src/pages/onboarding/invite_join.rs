// apps/folio/src/pages/onboarding/invite_join.rs
//
// InviteJoin — /join/:code
//
// Public landing page for invite code deep links (QR codes, SMS links, etc.).
// Resolves the code, displays the context card, and provides CTAs to:
//   a) Create an account (→ role-appropriate /onboard/:role?code=XXX)
//   b) Log in with existing account (→ /auth/login?next=...)
//
// No authentication required — this is the entry point for new users.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::wizard_shell::{ResolvedInviteCode, resolve_invite_code};

#[component]
pub fn InviteJoin() -> impl IntoView {
    let params  = use_params_map();
    let code    = move || params.with(|p| p.get("code").map(|s| s.to_string()).unwrap_or_default());

    let resolved: Resource<Result<Option<ResolvedInviteCode>, _>> =
        Resource::new(code, |c| resolve_invite_code(c));

    view! {
        <style>
            {r#"
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap');
            @import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap');
            .ms { font-family:'Material Symbols Outlined'; font-variation-settings:'FILL' 0,'wght' 400; line-height:1; display:inline-block; }
            .msf { font-variation-settings:'FILL' 1,'wght' 400; }
            body { margin:0; font-family:'Inter',sans-serif; background:#0f1922; color:#fff; min-height:100vh; display:flex; align-items:center; justify-content:center; padding:20px; }
            .ij-wrap { max-width:460px; width:100%; }
            .ij-logo { display:flex; align-items:center; justify-content:center; gap:10px; margin-bottom:32px; }
            .ij-logo-mark { width:36px; height:36px; background:rgba(255,255,255,.08); border:1px solid rgba(255,255,255,.12); border-radius:10px; display:flex; align-items:center; justify-content:center; }
            .ij-card { background:rgba(255,255,255,.04); border:1px solid rgba(255,255,255,.1); border-radius:20px; overflow:hidden; }
            .ij-hero { padding:28px 24px 20px; text-align:center; border-bottom:1px solid rgba(255,255,255,.06); }
            .ij-role-badge { display:inline-flex; align-items:center; gap:6px; background:rgba(99,102,241,.12); border:1px solid rgba(99,102,241,.25); color:#a5b4fc; font-size:11px; font-weight:700; text-transform:uppercase; letter-spacing:.08em; padding:4px 12px; border-radius:20px; margin-bottom:16px; }
            .ij-h { font-size:20px; font-weight:800; letter-spacing:-.02em; margin-bottom:6px; }
            .ij-sub { font-size:13px; color:rgba(255,255,255,.55); line-height:1.6; }
            .ij-details { padding:18px 24px; border-bottom:1px solid rgba(255,255,255,.06); }
            .ij-row { display:flex; align-items:flex-start; gap:10px; margin-bottom:12px; }
            .ij-row:last-child { margin-bottom:0; }
            .ij-row-ico { width:34px; height:34px; background:rgba(255,255,255,.05); border:1px solid rgba(255,255,255,.08); border-radius:9px; display:flex; align-items:center; justify-content:center; flex-shrink:0; }
            .ij-row-ico .ms { font-size:16px; color:rgba(255,255,255,.4); }
            .ij-row-label { font-size:11px; font-weight:700; text-transform:uppercase; letter-spacing:.06em; color:rgba(255,255,255,.3); margin-bottom:2px; }
            .ij-row-val { font-size:14px; font-weight:600; }
            .ij-msg { padding:16px 24px; border-bottom:1px solid rgba(255,255,255,.06); font-size:13px; color:rgba(255,255,255,.55); line-height:1.6; font-style:italic; }
            .ij-code { padding:14px 24px; border-bottom:1px solid rgba(255,255,255,.06); display:flex; align-items:center; justify-content:space-between; }
            .ij-code-label { font-size:11px; color:rgba(255,255,255,.3); margin-bottom:3px; }
            .ij-code-val { font-size:16px; font-weight:800; font-family:monospace; letter-spacing:.05em; color:#a5b4fc; }
            .ij-cta { padding:20px 24px; display:flex; flex-direction:column; gap:10px; }
            .ij-btn { display:flex; align-items:center; justify-content:center; gap:8px; width:100%; padding:13px; border-radius:12px; border:none; cursor:pointer; font-family:'Inter',sans-serif; font-size:15px; font-weight:700; text-decoration:none; transition:.15s; }
            .ij-btn-primary { background:#6366f1; color:#fff; box-shadow:0 4px 16px rgba(99,102,241,.3); }
            .ij-btn-primary:hover { background:#4f46e5; }
            .ij-btn-ghost { background:rgba(255,255,255,.06); color:#fff; border:1px solid rgba(255,255,255,.1); }
            .ij-btn-ghost:hover { background:rgba(255,255,255,.1); }
            .ij-note { font-size:12px; color:rgba(255,255,255,.3); text-align:center; margin-top:4px; }
            .ij-err { text-align:center; padding:40px 24px; }
            .ij-err-ico { font-size:48px; color:rgba(255,255,255,.2); margin-bottom:16px; }
            .ij-err-h { font-size:18px; font-weight:700; margin-bottom:8px; }
            .ij-err-p { font-size:14px; color:rgba(255,255,255,.5); line-height:1.6; }
            .ij-loading { text-align:center; padding:60px 24px; color:rgba(255,255,255,.4); font-size:14px; }
            "#}
        </style>

        <div class="ij-wrap">
            <div class="ij-logo">
                <div class="ij-logo-mark">
                    <span class="ms msf" style="font-size:18px; color:#fff;">"apartment"</span>
                </div>
                <span style="font-size:17px; font-weight:700;">"Folio"</span>
            </div>

            <Suspense fallback=|| view! {
                <div class="ij-card"><div class="ij-loading"><span class="ms">"sync"</span>" Resolving invite..."</div></div>
            }>
                {move || {
                    match resolved.get() {
                        None => view! { <div class="ij-card"><div class="ij-loading">"Loading..."</div></div> }.into_any(),
                        Some(Err(_)) | Some(Ok(None)) => view! {
                            <div class="ij-card">
                                <div class="ij-err">
                                    <div class="ij-err-ico"><span class="ms msf">"link_off"</span></div>
                                    <div class="ij-err-h">"Invite Code Not Found"</div>
                                    <div class="ij-err-p">
                                        "This invite code is invalid, expired, or has already been used. "
                                        "Please ask the person who sent it to generate a new one."
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        Some(Ok(Some(code_data))) => {
                            let role = code_data.role.clone();
                            let onboard_url = format!("/onboard/{}?code={}", role_to_path(&role), code_data.code);
                            let login_url   = format!("/auth/login?next={}", urlencoding::encode(&onboard_url));

                            view! {
                                <div class="ij-card">
                                    <div class="ij-hero">
                                        <div class="ij-role-badge">
                                            <span class="ms msf" style="font-size:14px;">{role_icon(&code_data.role)}</span>
                                            {role_label(&code_data.role)}
                                        </div>
                                        <div class="ij-h">
                                            {code_data.label.clone().unwrap_or_else(|| format!("You've been invited as a {}", role_label(&code_data.role)))}
                                        </div>
                                        <div class="ij-sub">
                                            {role_description(&code_data.role)}
                                        </div>
                                    </div>

                                    {if let Some(asset) = &code_data.context.asset {
                                        let addr = asset.address.clone().unwrap_or_default();
                                        view! {
                                            <div class="ij-details">
                                                <div class="ij-row">
                                                    <div class="ij-row-ico"><span class="ms msf">"location_on"</span></div>
                                                    <div>
                                                        <div class="ij-row-label">"Property"</div>
                                                        <div class="ij-row-val">{asset.name.clone()}</div>
                                                        <div style="font-size:12px; color:rgba(255,255,255,.4);">{addr}</div>
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else { view! { <span></span> }.into_any() }}

                                    {if let Some(msg) = &code_data.invite_message {
                                        view! { <div class="ij-msg"><span class="ms" style="font-size:15px; color:rgba(255,255,255,.2); margin-right:6px;">"format_quote"</span>{msg.clone()}</div> }.into_any()
                                    } else { view! { <span></span> }.into_any() }}

                                    <div class="ij-code">
                                        <div>
                                            <div class="ij-code-label">"Invite code"</div>
                                            <div class="ij-code-val">{code_data.code.clone()}</div>
                                        </div>
                                        {if let Some(rem) = code_data.uses_remaining {
                                            view! { <div style="font-size:12px; color:rgba(255,255,255,.35);">{format!("{} uses left", rem)}</div> }.into_any()
                                        } else { view! { <span></span> }.into_any() }}
                                    </div>

                                    <div class="ij-cta">
                                        <a href=onboard_url.clone() class="ij-btn ij-btn-primary">
                                            <span class="ms msf">{role_icon(&code_data.role)}</span>
                                            {format!("Create Account — {}", role_label(&code_data.role))}
                                        </a>
                                        <a href=login_url class="ij-btn ij-btn-ghost">
                                            <span class="ms">"login"</span>
                                            "I already have an account"
                                        </a>
                                        <div class="ij-note">"By continuing you agree to Folio\u{2019}s Terms of Service and Privacy Policy"</div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn role_to_path(role: &str) -> &'static str {
    match role {
        "landlord"          => "landlord",
        "tenant"            => "tenant",
        "str_guest"         => "str-guest",
        "vendor"            => "vendor",      // → /onboard/vendor (VendorWizard)
        "cohost"            => "cohost",
        "owner"             => "owner",
        "agent"             => "agent",
        "broker"            => "broker",
        "property_manager"  => "pmc",         // → /onboard/pmc (PmcWizard)
        _                   => "landlord",
    }
}

fn role_label(role: &str) -> &'static str {
    match role {
        "landlord"          => "Landlord",
        "tenant"            => "Tenant Applicant",
        "str_guest"         => "STR Guest",
        "vendor"            => "Vendor",
        "cohost"            => "Co-host",
        "owner"             => "Property Owner",
        "agent"             => "Agent",
        "broker"            => "Broker",
        "property_manager"  => "Property Manager",
        _                   => "User",
    }
}

fn role_icon(role: &str) -> &'static str {
    match role {
        "landlord"          => "apartment",
        "tenant"            => "door_front",
        "str_guest"         => "beach_access",
        "vendor"            => "handyman",
        "cohost"            => "supervisor_account",
        "owner"             => "account_balance",
        "agent"             => "real_estate_agent",
        "broker"            => "gavel",
        "property_manager"  => "corporate_fare",
        _                   => "person",
    }
}

fn role_description(role: &str) -> &'static str {
    match role {
        "tenant"            => "Complete your rental application in about 8 minutes.",
        "str_guest"         => "Book your stay directly and skip platform fees.",
        "vendor"            => "Accept your work order and connect your vendor profile.",
        "cohost"            => "Accept your co-host invitation and set up your STR workspace.",
        "owner"             => "Activate your owner portal and view your portfolio.",
        "agent"             => "Join your broker\u{2019}s workspace to manage listings and clients.",
        "broker"            => "Set up your brokerage and invite your agent team.",
        "property_manager"  => "Connect your PMC workspace to manage client portfolios.",
        _                   => "Complete your profile and get started.",
    }
}

// Minimal URL encoding for the `next` redirect param
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars().map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        }).collect()
    }
}
