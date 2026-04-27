use leptos::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ModalState {
    None,
    Post(Option<crate::components::content_feed::ContentNode>),
    Profile(Option<crate::resume_engine::ResumeProfile>),
    BaseEntry(
        Option<crate::resume_engine::BaseResumeEntry>,
        Option<crate::resume_engine::ResumeCategory>,
    ),
    LandingPage(Option<crate::pages::dynamic_landing::LandingPageRecord>),
    MailingList(Option<crate::pages::admin::MailingListRecord>),
    NavItem(Option<crate::components::nav::NavItemRecord>),
    FooterItem(Option<crate::components::footer::FooterItemRecord>),
    PageHeader(Option<crate::components::dynamic_header::PageHeaderData>),
    Service(Option<crate::b2b::ServiceRecord>),
    CaseStudy(Option<crate::b2b::CaseStudyRecord>),
    Highlight(Option<crate::b2b::HighlightRecord>),
    LeadOption(Option<crate::pages::landing::LeadCaptureOption>),
    Passkey,
    Settings,
}

#[component]
pub fn AdminEditorModal() -> impl IntoView {
    let modal_state = expect_context::<ReadSignal<ModalState>>();
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let _set_refresh = expect_context::<WriteSignal<i32>>();
    let _refresh = expect_context::<ReadSignal<i32>>();

    let close_modal = move || set_modal_state.set(ModalState::None);

    view! {
        <Show when=move || modal_state.get() != ModalState::None>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-6 overflow-y-auto">
                <div class="relative w-full max-w-4xl bg-surface-container-highest p-1 blueprint-overlay max-h-[90vh] flex flex-col my-auto">
                    <button on:click=move |_| close_modal() class="absolute -top-4 -right-4 p-3 z-50 bg-surface-container-high border border-outline-variant/30 rounded-full text-outline hover:text-error hover:border-error transition-all shadow-xl">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                    <div class="bg-surface-container-lowest p-8 md:p-12 relative flex-1 overflow-y-auto">

                        <div class="mb-12 border-b-2 border-outline-variant/30 pb-6 mt-4">
                            <h2 class="text-3xl font-extrabold text-primary uppercase tracking-widest">
                                {move || match modal_state.get() {
                                    ModalState::Post(None) => "NEW BLOG POST",
                                    ModalState::Post(Some(_)) => "EDIT BLOG POST",
                                    ModalState::Profile(None) => "NEW RESUME PROFILE",
                                    ModalState::Profile(Some(_)) => "EDIT RESUME PROFILE",
                                    ModalState::BaseEntry(None, _) => "NEW RESUME ENTRY",
                                    ModalState::BaseEntry(Some(_), _) => "EDIT RESUME ENTRY",
                                    ModalState::LandingPage(None) => "NEW LANDING PAGE",
                                    ModalState::LandingPage(Some(_)) => "EDIT LANDING PAGE",
                                    ModalState::MailingList(None) => "ADD MAILING LIST MEMBER",
                                    ModalState::MailingList(Some(_)) => "EDIT MAILING LIST MEMBER",
                                    ModalState::NavItem(None) => "NEW NAVIGATION NODE",
                                    ModalState::NavItem(Some(_)) => "EDIT NAVIGATION NODE",
                                    ModalState::FooterItem(None) => "NEW FOOTER NODE",
                                    ModalState::FooterItem(Some(_)) => "EDIT FOOTER NODE",
                                    ModalState::PageHeader(None) => "NEW PAGE HEADER",
                                    ModalState::PageHeader(Some(_)) => "EDIT PAGE HEADER",
                                    ModalState::Service(None) => "NEW SERVICE MODULE",
                                    ModalState::Service(Some(_)) => "EDIT SERVICE MODULE",
                                    ModalState::CaseStudy(None) => "NEW CASE STUDY",
                                    ModalState::CaseStudy(Some(_)) => "EDIT CASE STUDY",
                                    ModalState::Highlight(None) => "NEW HIGHLIGHT SNAP",
                                    ModalState::Highlight(Some(_)) => "EDIT HIGHLIGHT SNAP",
                                    ModalState::LeadOption(None) => "NEW LEAD OPTION",
                                    ModalState::LeadOption(Some(_)) => "EDIT LEAD OPTION",
                                    ModalState::Passkey => "REGISTER NEW PASSKEY",
                                    ModalState::Settings => "EDIT SITE SETTINGS",
                                    ModalState::None => "",
                                }}
                            </h2>
                        </div>

                        // Modal Form Content
                        <div class="space-y-8">
                            {move || match modal_state.get() {
                                ModalState::Post(post) => {
                view! { <PostForm initial_post=post.clone() /> }.into_view()
            },
            ModalState::Profile(prof) => {
                view! { <ResumeProfileForm initial_profile=prof.clone() /> }.into_view()
            },
            ModalState::BaseEntry(entry, default_cat) => {
                let cat_clone = default_cat.clone();
                view! {
                    <BaseResumeEntryForm
                        initial_entry=entry.clone()
                        default_category=cat_clone
                    />
                }.into_view()
            },
                                ModalState::LandingPage(p) => view! { <LandingPageForm initial_page=p /> }.into_view(),
                                ModalState::MailingList(r) => view! { <MailingListForm initial_record=r /> }.into_view(),
                                ModalState::NavItem(n) => view! { <NavItemForm initial_item=n /> }.into_view(),
                                ModalState::FooterItem(f) => view! { <FooterItemForm initial_item=f /> }.into_view(),
                                ModalState::PageHeader(h) => view! { <PageHeaderForm initial_item=h /> }.into_view(),
                                ModalState::Service(s) => view! { <ServiceForm initial_item=s /> }.into_view(),
                                ModalState::CaseStudy(c) => view! { <CaseStudyForm initial_item=c /> }.into_view(),
                                ModalState::Highlight(h) => view! { <HighlightForm initial_item=h /> }.into_view(),
                                ModalState::LeadOption(h) => view! { <LeadOptionForm initial_item=h /> }.into_view(),
                                ModalState::Passkey => view! { <PasskeyForm /> }.into_view(),
                                ModalState::Settings => view! { <SettingsForm /> }.into_view(),
                                ModalState::None => view! { <div/> }.into_view(),
                            }}
                        </div>

                    </div>
                </div>
            </div>
        </Show>
    }
}
// -----------------------------------------

// -----------------------------------------
// Post Form (Markdown)
// -----------------------------------------
#[component]
pub fn PostForm(
    initial_post: Option<crate::components::content_feed::ContentNode>,
) -> impl IntoView {
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_post.is_some();
    let id_val = initial_post
        .as_ref()
        .map(|p| p.id.clone())
        .unwrap_or_default();

    let (title, set_title) = create_signal(
        initial_post
            .as_ref()
            .map(|p| p.title.clone())
            .unwrap_or_default(),
    );
    let (slug, set_slug) = create_signal(
        initial_post
            .as_ref()
            .and_then(|p| p.subtitle.clone())
            .unwrap_or_default(),
    );
    let (tags, set_tags) = create_signal(
        initial_post
            .as_ref()
            .map(|p| p.tags.join(", "))
            .unwrap_or_default(),
    );
    let (content, set_content) = create_signal(
        initial_post
            .as_ref()
            .and_then(|p| p.markdown.clone())
            .unwrap_or_default(),
    );

    // ── PDF settings state ───────────────────────────────────────────
    let (pdf_attachment_url, set_pdf_attachment_url) = create_signal(String::new());
    let (pdf_generate, set_pdf_generate) = create_signal(false);
    let (pdf_require_lead, set_pdf_require_lead) = create_signal(false);
    let (pdf_cta_label, set_pdf_cta_label) = create_signal(String::new());
    let (pdf_notify_email, set_pdf_notify_email) = create_signal(String::new());
    let (pdf_upload_status, set_pdf_upload_status) = create_signal(String::new());

    // Pre-fill PDF settings when editing an existing post
    let pdf_slug = initial_post
        .as_ref()
        .and_then(|p| p.subtitle.clone())
        .unwrap_or_default();
    let pdf_config_res = create_resource(
        move || pdf_slug.clone(),
        |s| async move {
            if s.is_empty() { return None; }
            crate::pages::blog::get_blog_pdf_config(s).await.unwrap_or(None)
        },
    );
    create_effect(move |_| {
        if let Some(Some(cfg)) = pdf_config_res.get() {
            set_pdf_attachment_url.set(cfg.pdf_attachment_url.unwrap_or_default());
            set_pdf_generate.set(cfg.pdf_generate_from_content);
            set_pdf_require_lead.set(cfg.pdf_require_lead_capture);
            set_pdf_cta_label.set(cfg.pdf_lead_capture_label.unwrap_or_default());
            set_pdf_notify_email.set(cfg.pdf_lead_notification_email.unwrap_or_default());
        }
    });

    // R2 presigned upload trigger
    let on_upload_click = move |_| {
        set_pdf_upload_status.set("Requesting upload URL...".to_string());
        spawn_local(async move {
            match crate::pages::blog::get_r2_presigned_upload_url("blog-attachment.pdf".to_string()).await {
                Ok(presigned_url) => {
                    let object_url = presigned_url.split('?').next().unwrap_or("").to_string();
                    set_pdf_attachment_url.set(object_url);
                    set_pdf_upload_status.set(format!("Presigned URL ready — use a PUT request to upload your PDF file."));
                }
                Err(e) => set_pdf_upload_status.set(format!("Error: {:?}", e)),
            }
        });
    };

    let post_id_sv = store_value(id_val.clone());
    let save = move |_| {
        let t = title.get_untracked();
        let s = slug.get_untracked();
        let c = content.get_untracked();
        let tg: Vec<String> = tags
            .get_untracked()
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x: &String| !x.is_empty())
            .collect();
        let current_id = post_id_sv.get_value();

        let p_url = { let u = pdf_attachment_url.get_untracked(); if u.is_empty() { None } else { Some(u) } };
        let p_gen = pdf_generate.get_untracked();
        let p_lead = pdf_require_lead.get_untracked();
        let p_label = { let l = pdf_cta_label.get_untracked(); if l.is_empty() { None } else { Some(l) } };
        let p_email = { let e = pdf_notify_email.get_untracked(); if e.is_empty() { None } else { Some(e) } };

        spawn_local(async move {
            if is_edit {
                let _ = crate::pages::blog::update_post(
                    current_id.clone(), s, t, c, tg, p_url, p_gen, p_lead, p_label, p_email
                ).await;
            } else {
                let _ = crate::pages::blog::add_post(
                    s, t, c, tg, p_url, p_gen, p_lead, p_label, p_email
                ).await;
            }
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            // ── Core post fields ────────────────────────────────────────────
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Title"</label>
                    <input type="text" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Slug (URL)"</label>
                    <input type="text" prop:value=slug on:input=move |ev| set_slug.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
            </div>
            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Tags (CSV)"</label>
                <input type="text" prop:value=tags on:input=move |ev| set_tags.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
            </div>
            <div class="flex flex-col gap-2 mt-4">
                <div class="flex justify-between items-end mb-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Markdown Content"</label>
                    <span class="jetbrains text-[0.55rem] text-secondary tracking-widest">"PULLDOWN-CMARK // ACTIVE"</span>
                </div>
                <textarea prop:value=content on:input=move |ev| set_content.set(event_target_value(&ev)) rows="15" class="bg-surface p-4 border border-outline-variant focus:border-primary focus:ring-0 text-sm font-mono text-on-surface resize-y whitespace-pre block w-full"></textarea>
            </div>

            // ── PDF Settings ────────────────────────────────────────────
            <div class="border-t border-outline-variant/30 pt-8 mt-4">
                <div class="bg-primary/5 border-l-4 border-primary p-4 mb-6">
                    <p class="jetbrains text-[0.65rem] text-on-surface uppercase tracking-widest font-bold">"PDF DELIVERY SETTINGS"</p>
                    <p class="jetbrains text-[0.6rem] mt-1 text-on-surface-variant leading-relaxed">
                        "Attach a pre-uploaded PDF or generate one on the fly from post content. "
                        "Optionally gate downloads behind a lead-capture form (name + email required)."
                    </p>
                </div>

                // Attachment URL + R2 upload trigger
                <div class="flex flex-col gap-2 mb-4">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">
                        "PDF Attachment URL (R2 Vault)"
                    </label>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            prop:value=pdf_attachment_url
                            on:input=move |ev| set_pdf_attachment_url.set(event_target_value(&ev))
                            placeholder="https://... or leave blank to generate on the fly"
                            class="bg-surface flex-1 p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono"
                        />
                        <button
                            on:click=on_upload_click
                            type="button"
                            class="bg-surface-container border border-outline-variant px-4 jetbrains text-[0.65rem] uppercase tracking-widest hover:border-primary transition-colors"
                        >
                            "GET UPLOAD URL"
                        </button>
                    </div>
                    <Show when=move || !pdf_upload_status.get().is_empty()>
                        <span class="jetbrains text-[0.58rem] text-secondary font-mono break-all">
                            {move || pdf_upload_status.get()}
                        </span>
                    </Show>
                </div>

                // Generate on the fly toggle
                <div class="flex items-center gap-4 mb-4 p-4 border border-outline-variant/30">
                    <label class="flex items-center gap-3 cursor-pointer flex-1">
                        <div class="relative">
                            <input type="checkbox" class="sr-only"
                                prop:checked=pdf_generate
                                on:change=move |ev| set_pdf_generate.set(event_target_checked(&ev))
                            />
                            <div class="block bg-surface-container-highest w-10 h-6 rounded-full transition-colors duration-300"
                                class:bg-primary=move || pdf_generate.get()></div>
                            <div class="dot absolute left-1 top-1 bg-surface w-4 h-4 rounded-full transition-transform duration-300"
                                class:translate-x-4=move || pdf_generate.get()></div>
                        </div>
                        <div>
                            <p class="jetbrains text-[0.65rem] uppercase tracking-widest font-bold text-on-surface">
                                "Generate PDF from Content (On the Fly)"
                            </p>
                            <p class="jetbrains text-[0.6rem] text-on-surface-variant mt-0.5">
                                "Creates a Kami-branded PDF from this post's markdown on each download. Ignored when Attachment URL is set."
                            </p>
                        </div>
                    </label>
                </div>

                // Lead capture gate toggle
                <div class="flex items-center gap-4 mb-4 p-4 border border-outline-variant/30">
                    <label class="flex items-center gap-3 cursor-pointer flex-1">
                        <div class="relative">
                            <input type="checkbox" class="sr-only"
                                prop:checked=pdf_require_lead
                                on:change=move |ev| set_pdf_require_lead.set(event_target_checked(&ev))
                            />
                            <div class="block bg-surface-container-highest w-10 h-6 rounded-full transition-colors duration-300"
                                class:bg-secondary=move || pdf_require_lead.get()></div>
                            <div class="dot absolute left-1 top-1 bg-surface w-4 h-4 rounded-full transition-transform duration-300"
                                class:translate-x-4=move || pdf_require_lead.get()></div>
                        </div>
                        <div>
                            <p class="jetbrains text-[0.65rem] uppercase tracking-widest font-bold text-on-surface">
                                "Gate Download Behind Lead Capture"
                            </p>
                            <p class="jetbrains text-[0.6rem] text-on-surface-variant mt-0.5">
                                "Readers must submit name + email before receiving a signed download token."
                            </p>
                        </div>
                    </label>
                </div>

                // CTA label + notification email
                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">
                            "CTA Button Label"
                        </label>
                        <input
                            type="text"
                            prop:value=pdf_cta_label
                            on:input=move |ev| set_pdf_cta_label.set(event_target_value(&ev))
                            placeholder="Download PDF"
                            class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains"
                        />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">
                            "Lead Notification Email"
                        </label>
                        <input
                            type="email"
                            prop:value=pdf_notify_email
                            on:input=move |ev| set_pdf_notify_email.set(event_target_value(&ev))
                            placeholder="admin@domain.com"
                            class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains"
                        />
                        <span class="jetbrains text-[0.58rem] text-on-surface-variant/70">
                            "You'll receive an email notification each time someone downloads this PDF."
                        </span>
                    </div>
                </div>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                "COMMIT TO DATABASE"
            </button>
        </div>
    }
}

// -----------------------------------------
// Passkey Form (New Device Binding)
// -----------------------------------------

#[component]
pub fn PasskeyForm() -> impl IntoView {
    #[allow(unused_variables)]
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    #[allow(unused_variables)]
    let set_refresh = expect_context::<WriteSignal<i32>>();
    #[allow(unused_variables)]
    let refresh = expect_context::<ReadSignal<i32>>();

    let (username, set_username) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (auth_error, set_auth_error) = create_signal(String::new());

    let save = move |_| {
        let uname = username.get_untracked();
        if uname.is_empty() {
            set_auth_error.set("Identity Hash (Username) required.".to_string());
            return;
        }

        set_is_loading.set(true);
        set_auth_error.set(String::new());

        spawn_local(async move {
            match crate::auth::request_magic_link(uname.clone()).await {
                Ok(_) => {
                    set_refresh.set(refresh.get_untracked() + 1);
                    set_modal_state.set(ModalState::None);
                }
                Err(e) => set_auth_error.set(format!("Server error: {:?}", e)),
            }
            set_is_loading.set(false);
        });
// End of spawn_local
    };

    view! {
        <div class="space-y-6">
            <div class="bg-secondary-container/10 p-4 border border-secondary mb-6">
                <p class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider leading-relaxed">
                    "Send a magic link to add a new admin identity."
                </p>
            </div>

            <Show when=move || !auth_error.get().is_empty()>
                <div class="bg-error/10 border-l-4 border-error p-4 text-error jetbrains text-sm font-medium">
                    {move || auth_error.get()}
                </div>
            </Show>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Email Address"</label>
                <input type="text" prop:value=username on:input=move |ev| set_username.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="new-admin@example.com" />
            </div>
            <button
                on:click=save
                disabled=is_loading
                class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex justify-center items-center gap-3"
            >
                <Show when=move || is_loading.get()>
                    <span class="material-symbols-outlined animate-spin text-base">"progress_activity"</span>
                </Show>
                <span class="inline-block translate-y-[1px]">"SEND MAGIC LINK"</span>
            </button>
        </div>
    }
}

// -----------------------------------------
// Settings Form
// -----------------------------------------
#[component]
pub fn SettingsForm() -> impl IntoView {
    use crate::pages::landing::get_site_settings;
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();

    let settings_res = create_resource(|| (), |_| get_site_settings());

    let (current_focus, set_current_focus) = create_signal(String::new());
    let (status, set_status) = create_signal(String::new());
    let (hero_quote, set_hero_quote) = create_signal(String::new());
    let (hero_subtitle, set_hero_subtitle) = create_signal(String::new());
    let (site_title, set_site_title) = create_signal(String::new());
    let (lc_title, set_lc_title) = create_signal(String::new());
    let (lc_desc, set_lc_desc) = create_signal(String::new());
    let (lc_label, set_lc_label) = create_signal(String::new());
    let (lc_placeholder, set_lc_placeholder) = create_signal(String::new());
    let (lc_btn, set_lc_btn) = create_signal(String::new());
    let (lc_footer, set_lc_footer) = create_signal(String::new());
    let (lc_endpoint, set_lc_endpoint) = create_signal(String::new());
    let (status_color, set_status_color) = create_signal(String::new());
    let (webhook_url, set_webhook_url) = create_signal(String::new());
    let (admin_email, set_admin_email) = create_signal(String::new());
    let (google_analytics_id, set_google_analytics_id) = create_signal(String::new());
    let (booking_url, set_booking_url) = create_signal(String::new());
    let (terms_html, set_terms_html) = create_signal(String::new());
    let (privacy_html, set_privacy_html) = create_signal(String::new());
    let (github_url, set_github_url) = create_signal(String::new());
    let (x_url, set_x_url) = create_signal(String::new());
    let (linkedin_url, set_linkedin_url) = create_signal(String::new());
    let (b2b_enabled, set_b2b_enabled) = create_signal(true);
    let (meta_title, set_meta_title) = create_signal(String::new());
    let (meta_description, set_meta_description) = create_signal(String::new());
    let (og_image, set_og_image) = create_signal(String::new());

    let smtp_res = create_resource(|| (), |_| crate::email::get_smtp_config());
    let (smtp_server, set_smtp_server) = create_signal(String::new());
    let (smtp_port, set_smtp_port) = create_signal(String::new());
    let (smtp_username, set_smtp_username) = create_signal(String::new());
    let (smtp_token, set_smtp_token) = create_signal(String::new());
    let (smtp_from, set_smtp_from) = create_signal(String::new());

    create_effect(move |_| {
        if let Some(Ok(s)) = settings_res.get() {
            set_current_focus.set(s.current_focus);
            set_status.set(s.status);
            set_hero_quote.set(s.hero_quote);
            set_hero_subtitle.set(s.hero_subtitle);
            set_site_title.set(s.site_title);
            set_lc_title.set(s.lc_title);
            set_lc_desc.set(s.lc_desc);
            set_lc_label.set(s.lc_label);
            set_lc_placeholder.set(s.lc_placeholder);
            set_lc_btn.set(s.lc_btn);
            set_lc_footer.set(s.lc_footer);
            set_lc_endpoint.set(s.lc_endpoint);
            set_status_color.set(s.status_color);
            set_webhook_url.set(s.webhook_url);
            set_admin_email.set(s.admin_email);
            set_google_analytics_id.set(s.google_analytics_id);
            set_booking_url.set(s.booking_url);
            set_terms_html.set(s.terms_html);
            set_privacy_html.set(s.privacy_html);
            set_github_url.set(s.github_url);
            set_x_url.set(s.x_url);
            set_linkedin_url.set(s.linkedin_url);
            set_b2b_enabled.set(s.b2b_enabled);
            set_meta_title.set(s.meta_title);
            set_meta_description.set(s.meta_description);
            set_og_image.set(s.og_image);
        }
        if let Some(Ok(c)) = smtp_res.get() {
            set_smtp_server.set(c.smtp_server);
            set_smtp_port.set(c.smtp_port);
            set_smtp_username.set(c.smtp_username);
            set_smtp_token.set(c.smtp_token);
            set_smtp_from.set(c.smtp_from);
        }
    });

    let save = move |_| {
        let cf = current_focus.get_untracked();
        let st = status.get_untracked();
        let hq = hero_quote.get_untracked();
        let hs = hero_subtitle.get_untracked();
        let sttl = site_title.get_untracked();
        let lt = lc_title.get_untracked();
        let ld = lc_desc.get_untracked();
        let ll = lc_label.get_untracked();
        let lp = lc_placeholder.get_untracked();
        let lb = lc_btn.get_untracked();
        let lf = lc_footer.get_untracked();
        let le = lc_endpoint.get_untracked();
        let sc = status_color.get_untracked();
        let wu = webhook_url.get_untracked();
        let ae = admin_email.get_untracked();
        let gai = google_analytics_id.get_untracked();
        let bu = booking_url.get_untracked();
        let th = terms_html.get_untracked();
        let ph = privacy_html.get_untracked();
        let gu = github_url.get_untracked();
        let xu = x_url.get_untracked();
        let lu = linkedin_url.get_untracked();
        let b2b = b2b_enabled.get_untracked();
        let mt = meta_title.get_untracked();
        let md = meta_description.get_untracked();
        let og = og_image.get_untracked();

        let shost = smtp_server.get_untracked();
        let sport = smtp_port.get_untracked();
        let suser = smtp_username.get_untracked();
        let stoken = smtp_token.get_untracked();
        let sfrom = smtp_from.get_untracked();

        spawn_local(async move {
            let _ = crate::pages::landing::update_site_settings(
                cf, st, hq, hs, sttl, lt, ld, ll, lp, lb, lf, le, sc, wu, ae, gai, bu, th, ph, gu,
                xu, lu, b2b, mt, md, og,
            )
            .await;
            let _ = crate::email::update_smtp_config(shost, sport, suser, stoken, sfrom).await;
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <Suspense fallback=move || view! { <div class="jetbrains text-sm">"Hydrating..."</div> }>
            <div class="space-y-6">
                <div class="bg-primary/10 border-l-4 border-primary p-4 mb-8">
                    <p class="jetbrains text-xs text-on-surface uppercase tracking-widest leading-relaxed">
                        "These key-value parameters are injected directly into the hero layout on the Root Navigation landing page."
                    </p>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Hero Parameter // Current Focus"</label>
                    <input type="text" prop:value=current_focus on:input=move |ev| set_current_focus.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Hero Parameter // Status"</label>
                    <input type="text" prop:value=status on:input=move |ev| set_status.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Hero Parameter // Subtitle"</label>
                    <textarea prop:value=hero_subtitle on:input=move |ev| set_hero_subtitle.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Hero Parameter // Quote"</label>
                    <textarea prop:value=hero_quote on:input=move |ev| set_hero_quote.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
                </div>

                <div class="bg-primary/5 border-l-4 border-primary p-4 my-6">
                    <p class="jetbrains text-[0.65rem] text-on-surface uppercase tracking-widest font-bold">"SEO & METADATA CONFIGURATION"</p>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Metadata // Global Title"</label>
                    <input type="text" prop:value=meta_title on:input=move |ev| set_meta_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Metadata // Global Description"</label>
                    <textarea prop:value=meta_description on:input=move |ev| set_meta_description.set(event_target_value(&ev)) rows="2" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Metadata // Open Graph Image URL"</label>
                    <input type="text" prop:value=og_image on:input=move |ev| set_og_image.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="https://example.com/banner.jpg" />
                </div>

                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider mt-4">"Lead Capture Parameter // Site Title"</label>
                    <input type="text" prop:value=site_title on:input=move |ev| set_site_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains" />
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider">"Lead Capture Parameter // Title"</label>
                        <input type="text" prop:value=lc_title on:input=move |ev| set_lc_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider">"Lead Capture Parameter // Description"</label>
                        <input type="text" prop:value=lc_desc on:input=move |ev| set_lc_desc.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains" />
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider">"Lead Capture Parameter // Form Label"</label>
                        <input type="text" prop:value=lc_label on:input=move |ev| set_lc_label.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider">"Lead Capture Parameter // Placeholder"</label>
                        <input type="text" prop:value=lc_placeholder on:input=move |ev| set_lc_placeholder.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains" />
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider">"Lead Capture Parameter // Button Stack"</label>
                        <input type="text" prop:value=lc_btn on:input=move |ev| set_lc_btn.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider">"Lead Capture Parameter // Endpoint Target"</label>
                        <input type="text" prop:value=lc_endpoint on:input=move |ev| set_lc_endpoint.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains font-mono text-secondary" />
                    </div>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-secondary tracking-wider">"Lead Capture Parameter // Footer Disclaimer"</label>
                    <input type="text" prop:value=lc_footer on:input=move |ev| set_lc_footer.set(event_target_value(&ev)) class="bg-surface p-3 border border-secondary/50 focus:border-secondary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Hero Parameter // Status LED Color"</label>
                    <div class="flex items-center gap-4">
                        <select on:change=move |ev| {
                                let v = event_target_value(&ev);
                                if v != "custom" { set_status_color.set(v); }
                            }
                            class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono uppercase"
                        >
                            <option value="custom">"Custom Color..."</option>
                            <option value="#ff5449" selected=move || status_color.get() == "#ff5449">"Error (Red) [#FF5449]"</option>
                            <option value="#34d399" selected=move || status_color.get() == "#34d399">"Success (Green) [#34D399]"</option>
                            <option value="#fbfaf8" selected=move || status_color.get() == "#fbfaf8">"Inactive (White) [#FBFAF8]"</option>
                            <option value="#f7931a" selected=move || status_color.get() == "#f7931a">"Bitcoin (Orange) [#F7931A]"</option>
                            <option value="#22d3ee" selected=move || status_color.get() == "#22d3ee">"Primary (Cyan) [#22D3EE]"</option>
                        </select>
                        <input type="color" prop:value=status_color on:input=move |ev| set_status_color.set(event_target_value(&ev)) class="w-12 h-12 bg-surface border border-outline-variant cursor-pointer p-1" />
                        <input type="text" prop:value=status_color on:input=move |ev| set_status_color.set(event_target_value(&ev)) class="bg-surface flex-1 p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono uppercase" placeholder="#FFFFFF" />
                    </div>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Webhook URL (On Lead Capture)"</label>
                    <input type="text" prop:value=webhook_url on:input=move |ev| set_webhook_url.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="https://..." />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Admin Notification Email"</label>
                    <input type="text" prop:value=admin_email on:input=move |ev| set_admin_email.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="admin@domain.com" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Google Analytics Tag ID"</label>
                    <input type="text" prop:value=google_analytics_id on:input=move |ev| set_google_analytics_id.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono uppercase" placeholder="G-XXXXXX" />
                </div>

                <div class="border-t border-outline-variant/30 pt-6 mt-4">
                    <h3 class="font-label text-sm font-bold text-primary tracking-widest uppercase mb-4">"B2B Config"</h3>
                    <div class="flex flex-col gap-2 mb-4">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Discovery Booking URL (Cal.com)"</label>
                        <input type="text" prop:value=booking_url on:input=move |ev| set_booking_url.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="https://cal.com/..." />
                    </div>
                    <div class="flex flex-col gap-2 mb-4">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Terms of Service (Markdown)"</label>
                        <textarea prop:value=terms_html on:input=move |ev| set_terms_html.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono resize-y"></textarea>
                    </div>
                    <div class="flex flex-col gap-2 mb-4">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Privacy Policy (Markdown)"</label>
                        <textarea prop:value=privacy_html on:input=move |ev| set_privacy_html.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono resize-y"></textarea>
                    </div>

                    <h3 class="font-label text-sm font-bold text-primary tracking-widest uppercase mb-4 mt-6">"Social Nav Links"</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div class="flex flex-col gap-2">
                            <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"GitHub Profile"</label>
                            <input type="text" prop:value=github_url on:input=move |ev| set_github_url.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="https://github.com/..." />
                        </div>
                        <div class="flex flex-col gap-2">
                            <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"X (Twitter) Profile"</label>
                            <input type="text" prop:value=x_url on:input=move |ev| set_x_url.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="https://x.com/..." />
                        </div>
                        <div class="flex flex-col gap-2">
                            <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"LinkedIn Profile"</label>
                            <input type="text" prop:value=linkedin_url on:input=move |ev| set_linkedin_url.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="https://linkedin.com/in/..." />
                        </div>
                    </div>
                </div>

                <div class="bg-tertiary/10 border-l-4 border-tertiary p-6 mt-8">
                    <h3 class="font-display font-bold text-tertiary text-lg mb-2">"B2B Consulting Mode (Stealth Flag)"</h3>
                    <p class="jetbrains text-[0.65rem] text-on-surface-variant leading-relaxed mb-6">
                        "If unchecked, all pages explicitly meant for B2B consulting (/services, /book) will redirect to the home page, successfully hiding your offerings from unauthorized visitors."
                    </p>
                    <label class="flex items-center gap-3 cursor-pointer">
                        <div class="relative">
                            <input type="checkbox" class="sr-only"
                                prop:checked=b2b_enabled
                                on:change=move |ev| set_b2b_enabled.set(event_target_checked(&ev))
                            />
                            <div class="block bg-surface-container-highest w-14 h-8 rounded-full shadow-inner transition-colors duration-300"
                                class:bg-tertiary=move || b2b_enabled.get()></div>
                            <div class="dot absolute left-1 top-1 bg-surface w-6 h-6 rounded-full transition-transform duration-300"
                                class:translate-x-6=move || b2b_enabled.get()></div>
                        </div>
                        <span class="jetbrains text-[0.7rem] uppercase tracking-widest font-bold text-on-surface">"Enable Master B2B Platform Flag"</span>
                    </label>
                </div>
                <div class="bg-primary/5 border-l-4 border-primary p-4 my-6">
                    <p class="jetbrains text-[0.65rem] text-on-surface uppercase tracking-widest font-bold">"EMAIL PROTOCOL SETTINGS (SMTP/SECURE)"</p>
                    <p class="jetbrains text-[0.6rem] mt-1 text-on-surface-variant leading-relaxed">"These credentials are sent directly to the server side system_secrets table. They never broadcast to public site hydration requests."</p>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"SMTP Protocol // Host"</label>
                        <input type="text" prop:value=smtp_server on:input=move |ev| set_smtp_server.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="smtp.protonmail.ch" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"SMTP Protocol // Port"</label>
                        <input type="text" prop:value=smtp_port on:input=move |ev| set_smtp_port.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="587" />
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"SMTP Credentials // Username"</label>
                        <input type="text" prop:value=smtp_username on:input=move |ev| set_smtp_username.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="username@proton.me" />
                    </div>
                    <div class="flex flex-col gap-2">
                        <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"SMTP Credentials // Secret Token"</label>
                        <input type="password" prop:value=smtp_token on:input=move |ev| set_smtp_token.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="Leave blank to keep current" />
                    </div>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"System Default // From Address"</label>
                    <input type="text" prop:value=smtp_from on:input=move |ev| set_smtp_from.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="admin@domain.com" />
                    <span class="text-[0.65rem] text-on-surface-variant/70">"The email address your messages are sent 'from'. For most providers (like Proton, Gmail), this must match your Username."</span>
                </div>

                <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                    "OVERWRITE GLOBAL SETTINGS"
                </button>
            </div>
        </Suspense>
    }
}

// -----------------------------------------
// Resume Profile Form
// -----------------------------------------
#[component]
pub fn ResumeProfileForm(
    initial_profile: Option<crate::resume_engine::ResumeProfile>,
) -> impl IntoView {
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_profile.is_some();
    let id_val = initial_profile.as_ref().map(|p| p.id).unwrap_or(0);

    let (name, set_name) = create_signal(
        initial_profile
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default(),
    );
    let (full_name, set_full_name) = create_signal(
        initial_profile
            .as_ref()
            .map(|p| p.full_name.clone())
            .unwrap_or_default(),
    );
    let (objective, set_objective) = create_signal(
        initial_profile
            .as_ref()
            .and_then(|p| p.objective.clone())
            .unwrap_or_default(),
    );
    let (is_public, set_is_public) = create_signal(
        initial_profile
            .as_ref()
            .map(|p| p.is_public)
            .unwrap_or(false),
    );

    let (target_role, set_target_role) = create_signal(
        initial_profile
            .as_ref()
            .and_then(|p| p.target_role.clone())
            .unwrap_or_default(),
    );
    let (contact_email, set_contact_email) = create_signal(
        initial_profile
            .as_ref()
            .and_then(|p| p.contact_email.clone())
            .unwrap_or_default(),
    );
    let (contact_phone, set_contact_phone) = create_signal(
        initial_profile
            .as_ref()
            .and_then(|p| p.contact_phone.clone())
            .unwrap_or_default(),
    );
    let (contact_location, set_contact_location) = create_signal(
        initial_profile
            .as_ref()
            .and_then(|p| p.contact_location.clone())
            .unwrap_or_default(),
    );
    let (contact_link, set_contact_link) = create_signal(
        initial_profile
            .as_ref()
            .and_then(|p| p.contact_link.clone())
            .unwrap_or_default(),
    );

    let get_vis = |key: &str| -> bool {
        initial_profile
            .as_ref()
            .and_then(|p| p.category_visibility.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    };

    let (work_vis, set_work_vis) = create_signal(get_vis("work"));
    let (education_vis, set_education_vis) = create_signal(get_vis("education"));
    let (certification_vis, set_certification_vis) = create_signal(get_vis("certification"));
    let (skill_vis, set_skill_vis) = create_signal(get_vis("skill"));
    let (project_vis, set_project_vis) = create_signal(get_vis("project"));
    let (language_vis, set_language_vis) = create_signal(get_vis("language"));
    let (volunteer_vis, set_volunteer_vis) = create_signal(get_vis("volunteer"));
    let (extracurricular_vis, set_extracurricular_vis) = create_signal(get_vis("extracurricular"));
    let (hobby_vis, set_hobby_vis) = create_signal(get_vis("hobby"));

    let default_order = vec![
        "work".to_string(),
        "education".to_string(),
        "certification".to_string(),
        "project".to_string(),
        "skill".to_string(),
        "volunteer".to_string(),
        "extracurricular".to_string(),
        "language".to_string(),
        "hobby".to_string(),
    ];
    let initial_order = initial_profile
        .as_ref()
        .and_then(|p| p.category_order.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or(default_order);

    let (category_order, set_category_order) = create_signal(initial_order);

    let move_up = move |idx: usize| {
        if idx > 0 {
            set_category_order.update(|order| {
                order.swap(idx, idx - 1);
            });
        }
    };

    let move_down = move |idx: usize| {
        set_category_order.update(|order| {
            if idx < order.len() - 1 {
                order.swap(idx, idx + 1);
            }
        });
    };

    let entries_res = create_resource(move || (), |_| crate::resume_engine::get_all_base_entries());
    let mapped_res = create_resource(
        move || (),
        move |_| async move {
            if id_val > 0 {
                crate::resume_engine::get_profile_entry_mappings(id_val).await
            } else {
                Ok(vec![])
            }
        },
    );

    let (active_entries, set_active_entries) =
        create_signal(std::collections::HashMap::<i32, Option<serde_json::Value>>::new());
    let (expanded_entries, set_expanded_entries) =
        create_signal(std::collections::HashSet::<i32>::new());

    create_effect(move |_| {
        if let Some(Ok(mappings)) = mapped_res.get() {
            let mut map = std::collections::HashMap::new();
            for m in mappings {
                map.insert(m.entry_id, m.overrides);
            }
            set_active_entries.set(map);
        }
    });

    let toggle_entry = move |eid: i32, checked: bool| {
        set_active_entries.update(|state| {
            if checked && !state.contains_key(&eid) {
                state.insert(eid, None);
            } else if !checked {
                state.remove(&eid);
                set_expanded_entries.update(|e| {
                    e.remove(&eid);
                });
            }
        });
    };

    let toggle_expand = move |eid: i32| {
        set_expanded_entries.update(|e| {
            if e.contains(&eid) {
                e.remove(&eid);
            } else {
                e.insert(eid);
            }
        });
    };

    let update_override = move |eid: i32, key: &str, val: String| {
        set_active_entries.update(|state| {
            if let Some(opt_val) = state.get_mut(&eid) {
                let mut obj = opt_val.take().unwrap_or_else(|| serde_json::json!({}));
                if val.is_empty() {
                    if let Some(map) = obj.as_object_mut() {
                        map.remove(key);
                    }
                } else if key == "bullets" {
                    let arr: Vec<String> = val
                        .split('\n')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    obj[key] = serde_json::to_value(arr).unwrap();
                } else {
                    obj[key] = serde_json::Value::String(val);
                }
                *opt_val = Some(obj);
            }
        });
    };

    let get_override = move |eid: i32, key: &str| -> String {
        active_entries.with(|state| {
            state
                .get(&eid)
                .and_then(|opt| opt.as_ref())
                .and_then(|v| v.get(key))
                .and_then(|v| {
                    if key == "bullets" {
                        v.as_array().map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                    } else {
                        v.as_str().map(|s| s.to_string())
                    }
                })
                .unwrap_or_default()
        })
    };

    let get_reactive_vis = move |key: &str| -> bool {
        match key {
            "work" => work_vis.get(),
            "education" => education_vis.get(),
            "certification" => certification_vis.get(),
            "skill" => skill_vis.get(),
            "project" => project_vis.get(),
            "language" => language_vis.get(),
            "volunteer" => volunteer_vis.get(),
            "extracurricular" => extracurricular_vis.get(),
            "hobby" => hobby_vis.get(),
            _ => true,
        }
    };

    let save = move |_| {
        let n = name.get_untracked();
        let fnm = full_name.get_untracked();
        let obj = if objective.get_untracked().is_empty() {
            None
        } else {
            Some(objective.get_untracked())
        };
        let p_pub = is_public.get_untracked();

        let tr = if target_role.get_untracked().is_empty() {
            None
        } else {
            Some(target_role.get_untracked())
        };
        let ce = if contact_email.get_untracked().is_empty() {
            None
        } else {
            Some(contact_email.get_untracked())
        };
        let cp = if contact_phone.get_untracked().is_empty() {
            None
        } else {
            Some(contact_phone.get_untracked())
        };
        let clo = if contact_location.get_untracked().is_empty() {
            None
        } else {
            Some(contact_location.get_untracked())
        };
        let cli = if contact_link.get_untracked().is_empty() {
            None
        } else {
            Some(contact_link.get_untracked())
        };

        let cv = serde_json::json!({
            "work": work_vis.get_untracked(),
            "education": education_vis.get_untracked(),
            "certification": certification_vis.get_untracked(),
            "skill": skill_vis.get_untracked(),
            "project": project_vis.get_untracked(),
            "language": language_vis.get_untracked(),
            "volunteer": volunteer_vis.get_untracked(),
            "extracurricular": extracurricular_vis.get_untracked(),
            "hobby": hobby_vis.get_untracked(),
        });

        let ae_map = active_entries.get_untracked();
        let mut ae = Vec::new();
        for (eid, overrides) in ae_map {
            ae.push(crate::resume_engine::ProfileEntryMapping {
                entry_id: eid,
                overrides,
            });
        }
        let co =
            serde_json::to_value(category_order.get_untracked()).unwrap_or(serde_json::json!([]));

        spawn_local(async move {
            if is_edit {
                let _ = crate::resume_engine::update_resume_profile(
                    id_val, n, fnm, obj, p_pub, tr, ce, cp, clo, cli, cv, co, ae,
                )
                .await;
            } else {
                let _ = crate::resume_engine::add_resume_profile(
                    n, fnm, obj, p_pub, tr, ce, cp, clo, cli, cv, co, ae,
                )
                .await;
            }
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center gap-3 bg-surface-container-high p-4 border border-outline-variant/30">
                <input
                    type="checkbox"
                    prop:checked=is_public
                    on:change=move |ev| set_is_public.set(event_target_checked(&ev))
                    class="w-5 h-5 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2"
                />
                <div>
                    <div class="font-bold text-sm text-on-surface uppercase tracking-widest">"Public Profile"</div>
                    <div class="text-xs text-outline leading-tight">"If enabled, this profile will be available to visitors on the frontend /resume component."</div>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Profile Name (Internal)"</label>
                    <input type="text" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. Architect Profile" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Document Header Name"</label>
                    <input type="text" prop:value=full_name on:input=move |ev| set_full_name.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. Ruud Salym Erie" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Target Role Headline"</label>
                    <input type="text" prop:value=target_role on:input=move |ev| set_target_role.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. Cloud Engineer" />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Executive Objective / Summary"</label>
                <textarea prop:value=objective on:input=move |ev| set_objective.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Contact Email"</label>
                    <input type="email" prop:value=contact_email on:input=move |ev| set_contact_email.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Contact Phone"</label>
                    <input type="text" prop:value=contact_phone on:input=move |ev| set_contact_phone.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Location (City, State)"</label>
                    <input type="text" prop:value=contact_location on:input=move |ev| set_contact_location.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"LinkedIn / Website"</label>
                    <input type="text" prop:value=contact_link on:input=move |ev| set_contact_link.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
            </div>

            <div class="flex flex-col gap-4 border-t border-outline-variant/30 pt-6">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Category Visibility Configuration"</label>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=work_vis on:change=move |ev| set_work_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Work"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=education_vis on:change=move |ev| set_education_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Education"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=certification_vis on:change=move |ev| set_certification_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Certification"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=skill_vis on:change=move |ev| set_skill_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Skill"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=project_vis on:change=move |ev| set_project_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Project"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=language_vis on:change=move |ev| set_language_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Language"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=volunteer_vis on:change=move |ev| set_volunteer_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Volunteer"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=extracurricular_vis on:change=move |ev| set_extracurricular_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Extracurricular"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="checkbox" prop:checked=hobby_vis on:change=move |ev| set_hobby_vis.set(event_target_checked(&ev)) class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                        <span class="jetbrains text-xs text-on-surface uppercase tracking-wider">"Hobby"</span>
                    </div>
                </div>
            </div>

            <div class="flex flex-col gap-4 border-t border-outline-variant/30 pt-6">
                <div class="flex justify-between items-center">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Document Section Ordering"</label>
                    <span class="text-[0.55rem] text-secondary tracking-widest font-mono">"PDF RENDER PRIORITY"</span>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {move || category_order.get().into_iter().enumerate()
                        .filter(|(_, cat_name)| get_reactive_vis(cat_name))
                        .map(|(idx, cat_name)| {
                        view! {
                            <div class="flex items-center justify-between bg-surface p-2 border border-outline-variant/30 hover:bg-surface-container-high transition-colors">
                                <span class="jetbrains text-xs font-bold text-on-surface uppercase tracking-wider px-2">{cat_name.clone()}</span>
                                <div class="flex gap-1">
                                    <button type="button" on:click=move |_| move_up(idx) class="p-1 hover:bg-surface-container-highest hover:text-primary transition-colors border border-outline-variant/30 bg-surface-container text-outline">
                                        <span class="material-symbols-outlined text-[1rem]">"arrow_upward"</span>
                                    </button>
                                    <button type="button" on:click=move |_| move_down(idx) class="p-1 hover:bg-surface-container-highest hover:text-primary transition-colors border border-outline-variant/30 bg-surface-container text-outline">
                                        <span class="material-symbols-outlined text-[1rem]">"arrow_downward"</span>
                                    </button>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="flex flex-col gap-4 border-t border-outline-variant/30 pt-6">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Assign Global Entries"</label>
                <div class="text-xs text-outline mb-2">"Select which resume entries should be included in this profile."</div>
                <Transition fallback=move || view! { <div class="text-xs text-outline">"Loading entries..."</div> }>
                    <div class="grid grid-cols-1 gap-2">
                        {move || match entries_res.get() {
                            Some(Ok(entries)) => {
                                entries.into_iter().map(|e| {
                                    let eid = e.id;
                                    let is_checked = move || active_entries.get().contains_key(&eid);
                                    let c_str = e.category.to_string();
                                    view! {
                                        <div class="flex flex-col bg-surface border border-outline-variant/30 transition-colors">
                                            <div class="flex items-center gap-3 p-3 hover:bg-surface-container-high">
                                                <input
                                                    type="checkbox"
                                                    prop:checked=is_checked
                                                    on:change=move |ev| toggle_entry(eid, event_target_checked(&ev))
                                                    class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2"
                                                />
                                                <div class="flex flex-col flex-1">
                                                    <span class="jetbrains text-xs font-bold text-on-surface uppercase tracking-wider">"[" {c_str} "] " {e.title.clone()}</span>
                                                    <span class="text-[0.65rem] text-outline truncate">{e.subtitle.clone().unwrap_or_default()}</span>
                                                </div>
                                                <Show when=move || active_entries.get().contains_key(&eid)>
                                                    <button
                                                        type="button"
                                                        on:click=move |_| toggle_expand(eid)
                                                        class="text-[0.65rem] font-bold jetbrains uppercase px-2 py-1 bg-surface-container border border-outline-variant/50 hover:text-primary transition-colors cursor-pointer"
                                                    >
                                                        {move || if expanded_entries.get().contains(&eid) { "Hide Overrides" } else { "Edit Overrides" }}
                                                    </button>
                                                </Show>
                                            </div>
                                            <Show when=move || expanded_entries.get().contains(&eid)>
                                                <div class="p-4 border-t border-outline-variant/30 bg-surface-container-lowest grid grid-cols-1 gap-4">
                                                    <div class="text-[0.65rem] text-secondary tracking-widest uppercase mb-2">"Leave blank to preserve absolute origin value"</div>
                                                    <div class="grid grid-cols-2 gap-4">
                                                        <div class="flex flex-col gap-1 col-span-2 md:col-span-1">
                                                            <label class="jetbrains text-[0.6rem] uppercase text-outline">"Title Override"</label>
                                                            <input type="text" prop:value=move || get_override(eid, "title") on:input=move |ev| update_override(eid, "title", event_target_value(&ev)) placeholder=e.title.clone() class="bg-surface p-2 border border-outline-variant focus:border-primary focus:ring-0 text-xs jetbrains w-full" />
                                                        </div>
                                                        <div class="flex flex-col gap-1 col-span-2 md:col-span-1">
                                                            <label class="jetbrains text-[0.6rem] uppercase text-outline">"Subtitle Override"</label>
                                                            <input type="text" prop:value=move || get_override(eid, "subtitle") on:input=move |ev| update_override(eid, "subtitle", event_target_value(&ev)) placeholder=e.subtitle.clone().unwrap_or_default() class="bg-surface p-2 border border-outline-variant focus:border-primary focus:ring-0 text-xs jetbrains w-full" />
                                                        </div>
                                                        <div class="flex flex-col gap-1 col-span-2">
                                                            <label class="jetbrains text-[0.6rem] uppercase text-outline">"Date Range Override"</label>
                                                            <input type="text" prop:value=move || get_override(eid, "date_range") on:input=move |ev| update_override(eid, "date_range", event_target_value(&ev)) placeholder=e.date_range.clone().unwrap_or_default() class="bg-surface p-2 border border-outline-variant focus:border-primary focus:ring-0 text-xs jetbrains w-full" />
                                                        </div>
                                                        <div class="flex flex-col gap-1 col-span-2">
                                                            <label class="jetbrains text-[0.6rem] uppercase text-outline">"Bullets Override (Newline separated)"</label>
                                                            <textarea prop:value=move || get_override(eid, "bullets") on:input=move |ev| update_override(eid, "bullets", event_target_value(&ev)) rows="3" placeholder=e.bullets.clone().join("\n") class="bg-surface p-2 border border-outline-variant focus:border-primary focus:ring-0 text-xs jetbrains w-full resize-y"></textarea>
                                                        </div>
                                                    </div>
                                                </div>
                                            </Show>
                                        </div>
                                    }
                                }).collect_view()
                            },
                            _ => view! { <div class="text-xs text-error">"Failed to load entries"</div> }.into_view()
                        }}
                    </div>
                </Transition>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                "COMMIT PROFILE TO DATABASE"
            </button>
        </div>
    }
}

// -----------------------------------------
// Landing Page Form
// -----------------------------------------
#[component]
pub fn LandingPageForm(
    initial_page: Option<crate::pages::dynamic_landing::LandingPageRecord>,
) -> impl IntoView {
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_page.is_some();
    let id_val = initial_page.as_ref().map(|p| p.id).unwrap_or(0);

    let (slug, set_slug) = create_signal(
        initial_page
            .as_ref()
            .map(|p| p.slug.clone())
            .unwrap_or_default(),
    );
    let (title, set_title) = create_signal(
        initial_page
            .as_ref()
            .map(|p| p.title.clone())
            .unwrap_or_default(),
    );
    let (description, set_description) = create_signal(
        initial_page
            .as_ref()
            .map(|p| p.description.clone())
            .unwrap_or_default(),
    );
    let (hero_title, set_hero_title) = create_signal(
        initial_page
            .as_ref()
            .map(|p| p.hero_title.clone())
            .unwrap_or_default(),
    );
    let (hero_subtitle, set_hero_subtitle) = create_signal(
        initial_page
            .as_ref()
            .map(|p| p.hero_subtitle.clone())
            .unwrap_or_default(),
    );
    let (dynamic_blocks_json, set_dynamic_blocks_json) = create_signal(
        initial_page
            .as_ref()
            .map(|p| p.dynamic_blocks_json.clone())
            .unwrap_or_else(|| "[]".to_string()),
    );

    let save = move |_| {
        let old_slug_val = initial_page.as_ref().map(|p| p.slug.clone()).unwrap_or_default();
        let s = slug.get_untracked();
        let t = title.get_untracked();
        let d = description.get_untracked();
        let ht = hero_title.get_untracked();
        let hs = hero_subtitle.get_untracked();
        let dbj = dynamic_blocks_json.get_untracked();

        spawn_local(async move {
            if is_edit {
                let _ = crate::pages::dynamic_landing::update_landing_page(
                    old_slug_val, s, t, d, ht, hs, dbj
                )
                .await;
            } else {
                let _ = crate::pages::dynamic_landing::add_landing_page(
                    s, t, d, ht, hs, dbj
                )
                .await;
            }
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Slug (URL path)"</label>
                    <input type="text" prop:value=slug on:input=move |ev| set_slug.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. real-estate" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Page Title (Tab Name)"</label>
                    <input type="text" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Top Description (Small Text)"</label>
                <textarea prop:value=description on:input=move |ev| set_description.set(event_target_value(&ev)) rows="2" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Hero Main Header (Can include HTML)"</label>
                <textarea prop:value=hero_title on:input=move |ev| set_hero_title.set(event_target_value(&ev)) rows="2" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>
            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Hero Subtitle (Underneath header)"</label>
                <textarea prop:value=hero_subtitle on:input=move |ev| set_hero_subtitle.set(event_target_value(&ev)) rows="2" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="border-t border-outline-variant/30 pt-6 mt-4">
                <h3 class="font-label text-sm font-bold text-primary tracking-widest uppercase mb-4">"Dynamic Blocks Payload"</h3>
                <div class="flex flex-col gap-2 mt-4">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Blocks JSON Array"</label>
                    <textarea prop:value=dynamic_blocks_json on:input=move |ev| set_dynamic_blocks_json.set(event_target_value(&ev)) rows="15" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-xs font-mono text-secondary resize-y" placeholder="[ Array of Block JSON Objects ]"></textarea>
                    <span class="text-xs text-outline">"Directly manages the blocks_payload column for this app_page."</span>
                </div>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                "COMMIT TO DATABASE"
            </button>
        </div>
    }
}

// -----------------------------------------
// Mailing List Form
// -----------------------------------------
#[component]
pub fn MailingListForm(
    initial_record: Option<crate::pages::admin::MailingListRecord>,
) -> impl IntoView {
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let (email, set_email) = create_signal(
        initial_record
            .as_ref()
            .map(|r| r.email.clone())
            .unwrap_or_default(),
    );
    let (list_type, set_list_type) = create_signal(
        initial_record
            .as_ref()
            .map(|r| r.list_type.clone())
            .unwrap_or_else(|| "manual_override".to_string()),
    );

    let save = move |_| {
        let e = email.get_untracked();
        let lt = list_type.get_untracked();

        spawn_local(async move {
            let _ = crate::pages::dynamic_landing::handle_dynamic_lead(lt, e, vec![]).await;
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Lead Email"</label>
                <input type="email" prop:value=email on:input=move |ev| set_email.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="guest@example.com" />
            </div>
            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"List Identifier (Tags)"</label>
                <input type="text" prop:value=list_type on:input=move |ev| set_list_type.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
            </div>

            <button on:click=save class="w-full bg-primary text-on-primary py-4 mt-6 jetbrains text-xs font-bold tracking-[0.2em] uppercase hover:bg-primary-container transition-colors shadow-lg">
                "SUBMIT LEAD"
            </button>
        </div>
    }
}

// -----------------------------------------
// Navigation Item Form
// -----------------------------------------
#[component]
pub fn NavItemForm(initial_item: Option<crate::components::nav::NavItemRecord>) -> impl IntoView {
    use crate::components::nav::*;
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_item.is_some();
    let id_val = initial_item.as_ref().map(|p| p.id).unwrap_or_default();

    let (label, set_label) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.label.clone())
            .unwrap_or_default(),
    );
    let (href, set_href) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.href.clone())
            .unwrap_or_default(),
    );
    let (parent_id_str, set_parent_id_str) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.parent_id)
            .map(|id| id.to_string())
            .unwrap_or_default(),
    );
    let (display_order, set_display_order) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.display_order.to_string())
            .unwrap_or_else(|| "0".to_string()),
    );
    let (is_visible, set_is_visible) =
        create_signal(initial_item.as_ref().map(|p| p.is_visible).unwrap_or(true));

    let (loading, set_loading) = create_signal(false);

    // Quick dropdown loader for parent
    let parents_resource = create_resource(move || refresh.get(), |_| get_all_nav_items());

    let save_action = create_action(move |_: &()| {
        let lbl = label.get_untracked();
        let hr = href.get_untracked();
        let pid_str = parent_id_str.get_untracked();
        let ord_str = display_order.get_untracked();
        let vis = is_visible.get_untracked();

        let hr_opt = if hr.is_empty() { None } else { Some(hr) };
        let pid_opt = if pid_str.is_empty() {
            None
        } else {
            pid_str.parse::<uuid::Uuid>().ok()
        };
        let ord = ord_str.parse::<i32>().unwrap_or(0);

        async move {
            set_loading.set(true);
            if is_edit {
                let _ = update_nav_item(id_val, lbl, hr_opt, pid_opt, ord, vis).await;
            } else {
                let _ = add_nav_item(lbl, hr_opt, pid_opt, ord, vis).await;
            }
            set_loading.set(false);
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        }
    });

    view! {
        <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline">"Display Label"</label>
                    <input
                        type="text"
                        prop:value=label
                        on:input=move |ev| set_label.set(event_target_value(&ev))
                        class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-2 jetbrains text-sm text-on-surface transition-all placeholder:text-outline-variant/30"
                        placeholder="e.g. BLOG"
                    />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline">"Weight (Order)"</label>
                    <input
                        type="number"
                        prop:value=display_order
                        on:input=move |ev| set_display_order.set(event_target_value(&ev))
                        class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-2 jetbrains text-sm text-on-surface transition-all"
                    />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline">"Target URL (Href)"</label>
                <input
                    type="text"
                    prop:value=href
                    on:input=move |ev| set_href.set(event_target_value(&ev))
                    class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-2 jetbrains text-sm text-on-surface transition-all placeholder:text-outline-variant/30"
                    placeholder="e.g. /blog or leave empty for Dropdown Header"
                />
                <span class="text-xs text-outline-variant jetbrains">"If left blank, this item acts as a dropdown menu parent."</span>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline">"Dropdown Parent Binding"</label>
                <select
                    on:change=move |ev| set_parent_id_str.set(event_target_value(&ev))
                    class="w-full bg-surface-container-high border-none focus:ring-primary px-4 py-3 jetbrains text-sm text-on-surface outline-none cursor-pointer"
                >
                    <option value="" selected=move || parent_id_str.get().is_empty()>"-- NONE (TOP LEVEL) --"</option>
                    <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                        {move || match parents_resource.get() {
                            Some(Ok(items)) => items.into_iter()
                                .filter(|i| i.href.is_none() || i.href.as_deref() == Some(""))
                                .map(|i| {
                                    let id_s = i.id.to_string();
                                    view! {
                                        <option value=id_s.clone() selected=move || parent_id_str.get() == id_s>
                                            {format!("{} (ID: {})", i.label, i.id)}
                                        </option>
                                    }
                            }).collect_view(),
                            _ => view! { <option disabled=true>"ERROR"</option> }.into_view()
                        }}
                    </Suspense>
                </select>
                <span class="text-xs text-outline-variant jetbrains">"Select a parent item to nest this link inside a dropdown."</span>
            </div>

            <div class="flex items-center gap-3 bg-surface-container/50 p-4 border border-outline-variant/20">
                <label class="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox"
                        class="sr-only peer"
                        prop:checked=is_visible
                        on:change=move |ev| set_is_visible.set(event_target_checked(&ev))
                    />
                    <div class="w-9 h-5 bg-outline-variant peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-surface after:border-outline-variant after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
                    <span class="ml-3 text-xs jetbrains font-bold uppercase tracking-widest text-primary">
                        {move || if is_visible.get() { "VISIBLE ON SITE" } else { "HIDDEN ENTIRELY" }}
                    </span>
                </label>
            </div>

            <div class="flex justify-end gap-4 pt-4 border-t-2 border-outline-variant/30">
                <button
                    on:click=move |_| set_modal_state.set(ModalState::None)
                    class="px-6 py-3 jetbrains text-xs font-bold tracking-widest uppercase text-outline hover:text-on-surface transition-colors"
                    disabled=loading
                >
                    "Cancel"
                </button>
                <button
                    on:click=move |_| save_action.dispatch(())
                    class="bg-primary text-on-primary px-8 py-3 jetbrains text-xs font-bold tracking-widest uppercase hover:bg-primary-container transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                    disabled=loading
                >
                    <Show when=move || loading.get()>
                        <span class="material-symbols-outlined animate-spin text-sm">"progress_activity"</span>
                    </Show>
                    {move || if is_edit { "COMMIT CHANGES" } else { "CREATE NODE" }}
                </button>
            </div>
        </div>
    }
}

// -----------------------------------------
// Footer Item Form
// -----------------------------------------
#[component]
pub fn FooterItemForm(
    initial_item: Option<crate::components::footer::FooterItemRecord>,
) -> impl IntoView {
    use crate::components::footer::*;
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_item.is_some();
    let id_val = initial_item.as_ref().map(|p| p.id).unwrap_or(0);

    let (label, set_label) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.label.clone())
            .unwrap_or_default(),
    );
    let (href, set_href) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.href.clone())
            .unwrap_or_default(),
    );
    let (display_order, set_display_order) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.display_order.to_string())
            .unwrap_or_else(|| "0".to_string()),
    );
    let (is_visible, set_is_visible) =
        create_signal(initial_item.as_ref().map(|p| p.is_visible).unwrap_or(true));

    let (loading, set_loading) = create_signal(false);

    let save_action = create_action(move |_: &()| {
        let lbl = label.get_untracked();
        let hr = href.get_untracked();
        let ord_str = display_order.get_untracked();
        let vis = is_visible.get_untracked();

        let hr_opt = if hr.is_empty() { None } else { Some(hr) };
        let ord = ord_str.parse::<i32>().unwrap_or(0);

        async move {
            set_loading.set(true);
            if is_edit {
                let _ = update_footer_item(id_val, lbl, hr_opt, ord, vis).await;
            } else {
                let _ = add_footer_item(lbl, hr_opt, ord, vis).await;
            }
            set_loading.set(false);
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        }
    });

    view! {
        <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline">"Display Label"</label>
                    <input
                        type="text"
                        prop:value=label
                        on:input=move |ev| set_label.set(event_target_value(&ev))
                        class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-2 jetbrains text-sm text-on-surface transition-all placeholder:text-outline-variant/30"
                        placeholder="e.g. TERMS OF SERVICE"
                    />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline">"Weight (Order)"</label>
                    <input
                        type="number"
                        prop:value=display_order
                        on:input=move |ev| set_display_order.set(event_target_value(&ev))
                        class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-2 jetbrains text-sm text-on-surface transition-all"
                    />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline">"Target URL (Href)"</label>
                <input
                    type="text"
                    prop:value=href
                    on:input=move |ev| set_href.set(event_target_value(&ev))
                    class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-2 jetbrains text-sm text-on-surface transition-all placeholder:text-outline-variant/30"
                    placeholder="e.g. /terms"
                />
            </div>

            <div class="flex items-center gap-3 bg-surface-container/50 p-4 border border-outline-variant/20">
                <label class="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox"
                        class="sr-only peer"
                        prop:checked=is_visible
                        on:change=move |ev| set_is_visible.set(event_target_checked(&ev))
                    />
                    <div class="w-9 h-5 bg-outline-variant peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-surface after:border-outline-variant after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
                    <span class="ml-3 text-xs jetbrains font-bold uppercase tracking-widest text-primary">
                        {move || if is_visible.get() { "VISIBLE ON SITE" } else { "HIDDEN ENTIRELY" }}
                    </span>
                </label>
            </div>

            <div class="flex justify-end gap-4 pt-4 border-t-2 border-outline-variant/30">
                <button
                    on:click=move |_| set_modal_state.set(ModalState::None)
                    class="px-6 py-3 jetbrains text-xs font-bold tracking-widest uppercase text-outline hover:text-on-surface transition-colors"
                    disabled=loading
                >
                    "Cancel"
                </button>
                <button
                    on:click=move |_| save_action.dispatch(())
                    class="bg-primary text-on-primary px-8 py-3 jetbrains text-xs font-bold tracking-widest uppercase hover:bg-primary-container transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                    disabled=loading
                >
                    <Show when=move || loading.get()>
                        <span class="material-symbols-outlined animate-spin text-sm">"progress_activity"</span>
                    </Show>
                    {move || if is_edit { "COMMIT CHANGES" } else { "CREATE NODE" }}
                </button>
            </div>
        </div>
    }
}

// -----------------------------------------
// Base Resume Entry Form
// -----------------------------------------
#[component]
pub fn BaseResumeEntryForm(
    initial_entry: Option<crate::resume_engine::BaseResumeEntry>,
    default_category: Option<crate::resume_engine::ResumeCategory>,
) -> impl IntoView {
    use crate::resume_engine::{add_base_entry, update_base_entry, ResumeCategory};
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_entry.is_some();
    let id_val = initial_entry.as_ref().map(|e| e.id).unwrap_or(0);

    let (category_str, set_category_str) = create_signal(
        initial_entry
            .as_ref()
            .map(|e| e.category.to_string())
            .unwrap_or_else(|| {
                default_category
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "Work".to_string())
            }),
    );
    let (title, set_title) = create_signal(
        initial_entry
            .as_ref()
            .map(|e| e.title.clone())
            .unwrap_or_default(),
    );
    let (subtitle, set_subtitle) = create_signal(
        initial_entry
            .as_ref()
            .and_then(|e| e.subtitle.clone())
            .unwrap_or_default(),
    );
    let (date_range, set_date_range) = create_signal(
        initial_entry
            .as_ref()
            .and_then(|e| e.date_range.clone())
            .unwrap_or_default(),
    );
    let (bullets_str, set_bullets_str) = create_signal(
        initial_entry
            .as_ref()
            .map(|e| e.bullets.join("\n"))
            .unwrap_or_default(),
    );
    let (metadata_str, set_metadata_str) = create_signal(
        initial_entry
            .as_ref()
            .and_then(|e| {
                e.metadata
                    .as_ref()
                    .map(|m| serde_json::to_string_pretty(m).unwrap_or_default())
            })
            .unwrap_or_default(),
    );

    let profiles_res = create_resource(move || (), |_| crate::resume_engine::get_entry_collections());

    // Fetch mapped profiles for this entry
    let mapped_res = create_resource(
        move || (),
        move |_| async move {
            if id_val > 0 {
                crate::resume_engine::get_entry_profile_mappings(id_val).await
            } else {
                Ok(vec![])
            }
        },
    );

    let (active_profiles, set_active_profiles) = create_signal(Vec::<i32>::new());

    create_effect(move |_| {
        if let Some(Ok(mappings)) = mapped_res.get() {
            set_active_profiles.set(mappings);
        }
    });

    let toggle_profile = move |pid: i32, checked: bool| {
        set_active_profiles.update(|state| {
            if checked && !state.contains(&pid) {
                state.push(pid);
            } else if !checked {
                state.retain(|&x| x != pid);
            }
        });
    };

    let save = move |_| {
        let cat_val = match category_str.get_untracked().as_str() {
            "Work" => ResumeCategory::Work,
            "Education" => ResumeCategory::Education,
            "Skill" => ResumeCategory::Skill,
            "Project" => ResumeCategory::Project,
            "Language" => ResumeCategory::Language,
            "Volunteer" => ResumeCategory::Volunteer,
            "Extracurricular" => ResumeCategory::Extracurricular,
            "Hobby" => ResumeCategory::Hobby,
            _ => ResumeCategory::Work,
        };
        let t = title.get_untracked();
        let sub = if subtitle.get_untracked().is_empty() {
            None
        } else {
            Some(subtitle.get_untracked())
        };
        let dr = if date_range.get_untracked().is_empty() {
            None
        } else {
            Some(date_range.get_untracked())
        };
        let b: Vec<String> = bullets_str
            .get_untracked()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect();
        let profs = active_profiles.get_untracked();

        let md_str = metadata_str.get_untracked();
        let md = if md_str.trim().is_empty() {
            None
        } else {
            serde_json::from_str(&md_str).ok()
        };

        spawn_local(async move {
            if is_edit {
                let _ = update_base_entry(id_val, cat_val, t, sub, dr, b, md, profs).await;
            } else {
                let _ = add_base_entry(cat_val, t, sub, dr, b, md, profs).await;
            }
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Category"</label>
                    <select prop:value=category_str on:change=move |ev| set_category_str.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains appearance-none">
                        <option value="Work">"Work"</option>
                        <option value="Education">"Education"</option>
                        <option value="Skill">"Skill"</option>
                        <option value="Project">"Project"</option>
                        <option value="Language">"Language"</option>
                        <option value="Volunteer">"Volunteer"</option>
                        <option value="Extracurricular">"Extracurricular"</option>
                        <option value="Hobby">"Hobby"</option>
                    </select>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Title"</label>
                    <input type="text" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. Software Engineer" />
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Subtitle / Company"</label>
                    <input type="text" prop:value=subtitle on:input=move |ev| set_subtitle.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. Google" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Date Range"</label>
                    <input type="text" prop:value=date_range on:input=move |ev| set_date_range.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. Jan 2020 - Present" />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Bullets (1 per line)"</label>
                <textarea prop:value=bullets_str on:input=move |ev| set_bullets_str.set(event_target_value(&ev)) rows="5" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y leading-relaxed font-mono" placeholder="Maintained distributed systems...\nIncreased performance by 30%..."></textarea>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Metadata (Valid JSON. Optional. Used for generic Projects/Certs URLs etc)"</label>
                <textarea prop:value=metadata_str on:input=move |ev| set_metadata_str.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y leading-relaxed font-mono" placeholder={r#"{"slug": "test-project", "image_url": "..."}"#}></textarea>
            </div>

            <div class="flex flex-col gap-4 border-t border-outline-variant/30 pt-6">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider font-bold">"Assign to Profiles"</label>
                <div class="text-xs text-outline mb-2">"Select which profiles should include this entry by default."</div>
                <Transition fallback=move || view! { <div class="text-xs text-outline">"Loading profiles..."</div> }>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        {move || match profiles_res.get() {
                            Some(Ok(profiles)) => {
                                profiles.into_iter().map(|p| {
                                    let pid = p.id;
                                    let is_checked = move || active_profiles.get().contains(&pid);
                                    view! {
                                        <div class="flex items-center gap-3 bg-surface p-3 border border-outline-variant/30 hover:bg-surface-container-high transition-colors">
                                            <input
                                                type="checkbox"
                                                prop:checked=is_checked
                                                on:change=move |ev| toggle_profile(pid, event_target_checked(&ev))
                                                class="w-4 h-4 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2"
                                            />
                                            <div class="flex flex-col">
                                                <span class="jetbrains text-xs font-bold text-on-surface uppercase tracking-wider">{p.name}</span>
                                                <span class="text-[0.65rem] text-outline truncate">{p.target_role.unwrap_or_default()}</span>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()
                            },
                            _ => view! { <div class="text-xs text-error">"Failed to load profiles"</div> }.into_view()
                        }}
                    </div>
                </Transition>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                {if is_edit { "UPDATE ENTRY" } else { "CREATE ENTRY" }}
            </button>
        </div>
    }
}

// -----------------------------------------
// Service Form
// -----------------------------------------
#[component]
pub fn ServiceForm(initial_item: Option<crate::b2b::ServiceRecord>) -> impl IntoView {
    use crate::b2b::*;
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_item.is_some();
    let id_val = initial_item.as_ref().map(|p| p.id).unwrap_or(uuid::Uuid::nil());

    let (title, set_title) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.title.clone())
            .unwrap_or_default(),
    );
    let (description, set_description) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.description.clone())
            .unwrap_or_default(),
    );

    let default_deliverables = "[\n  \"First deliverable\"\n]".to_string();
    let initial_deliverables = if let Some(item) = initial_item.as_ref() {
        if item.deliverables.is_empty() {
            default_deliverables
        } else {
            serde_json::to_string_pretty(&item.deliverables).unwrap_or(default_deliverables)
        }
    } else {
        default_deliverables
    };
    let (deliverables_str, set_deliverables_str) = create_signal(initial_deliverables);

    let (price_range, set_price_range) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.price_range.clone())
            .unwrap_or_default(),
    );
    let (is_visible, set_is_visible) =
        create_signal(initial_item.as_ref().map(|p| p.is_visible).unwrap_or(true));
    let (display_order, set_display_order) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.display_order.to_string())
            .unwrap_or_else(|| "0".to_string()),
    );

    let save = move |_| {
        let t = title.get_untracked();
        let desc = description.get_untracked();
        let deliv_json = deliverables_str.get_untracked();
        let deliv_vec: Vec<String> = serde_json::from_str(&deliv_json).unwrap_or_default();
        let pr = price_range.get_untracked();
        let pr_opt = if pr.is_empty() { None } else { Some(pr) };
        let iv = is_visible.get_untracked();
        let ord = display_order.get_untracked().parse::<i32>().unwrap_or(0);

        spawn_local(async move {
            if is_edit {
                let _ = update_service(id_val, t, desc, deliv_vec, pr_opt, iv, ord).await;
            } else {
                let _ = add_service(t, desc, deliv_vec, pr_opt, iv, ord).await;
            }
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Title"</label>
                <input type="text" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Description"</label>
                <textarea prop:value=description on:input=move |ev| set_description.set(event_target_value(&ev)) rows="5" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Deliverables (JSON Array Array of Strings)"</label>
                    <textarea prop:value=deliverables_str on:input=move |ev| set_deliverables_str.set(event_target_value(&ev)) rows="4" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm font-mono text-secondary resize-y"></textarea>
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Price Range"</label>
                    <input type="text" prop:value=price_range on:input=move |ev| set_price_range.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="e.g. $5k - $15k" />
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Display Order"</label>
                    <input type="number" prop:value=display_order on:input=move |ev| set_display_order.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex items-center gap-3 pt-6">
                    <input type="checkbox" prop:checked=is_visible on:change=move |ev| set_is_visible.set(event_target_checked(&ev)) class="w-5 h-5 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                    <div class="font-bold text-sm text-on-surface uppercase tracking-widest">"Visible"</div>
                </div>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                "SAVE SERVICE"
            </button>
        </div>
    }
}

// -----------------------------------------
// Case Study Form
// -----------------------------------------
#[component]
pub fn CaseStudyForm(initial_item: Option<crate::b2b::CaseStudyRecord>) -> impl IntoView {
    use crate::b2b::*;
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_item.is_some();
    let id_val = initial_item.as_ref().map(|p| p.id).unwrap_or(uuid::Uuid::nil());

    let (client_name, set_client_name) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.client_name.clone())
            .unwrap_or_default(),
    );
    let (problem, set_problem) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.problem.clone())
            .unwrap_or_default(),
    );
    let (solution, set_solution) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.solution.clone())
            .unwrap_or_default(),
    );
    let (roi_impact, set_roi_impact) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.roi_impact.clone())
            .unwrap_or_default(),
    );
    let (is_visible, set_is_visible) =
        create_signal(initial_item.as_ref().map(|p| p.is_visible).unwrap_or(true));
    let (display_order, set_display_order) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.display_order.to_string())
            .unwrap_or_else(|| "0".to_string()),
    );

    let save = move |_| {
        let cn = client_name.get_untracked();
        let prob = problem.get_untracked();
        let sol = solution.get_untracked();
        let roi = roi_impact.get_untracked();
        let iv = is_visible.get_untracked();
        let ord = display_order.get_untracked().parse::<i32>().unwrap_or(0);

        spawn_local(async move {
            if is_edit {
                let _ = update_case_study(id_val, cn, prob, sol, roi, iv, ord).await;
            } else {
                let _ = add_case_study(cn, prob, sol, roi, iv, ord).await;
            }
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Client Name / Title"</label>
                <input type="text" prop:value=client_name on:input=move |ev| set_client_name.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Problem"</label>
                <textarea prop:value=problem on:input=move |ev| set_problem.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Solution"</label>
                <textarea prop:value=solution on:input=move |ev| set_solution.set(event_target_value(&ev)) rows="5" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"ROI Impact"</label>
                <textarea prop:value=roi_impact on:input=move |ev| set_roi_impact.set(event_target_value(&ev)) rows="3" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Display Order"</label>
                    <input type="number" prop:value=display_order on:input=move |ev| set_display_order.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex items-center gap-3 pt-6">
                    <input type="checkbox" prop:checked=is_visible on:change=move |ev| set_is_visible.set(event_target_checked(&ev)) class="w-5 h-5 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                    <div class="font-bold text-sm text-on-surface uppercase tracking-widest">"Visible"</div>
                </div>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                "SAVE CASE STUDY"
            </button>
        </div>
    }
}

// -----------------------------------------
// Highlight Form
// -----------------------------------------
#[component]
pub fn HighlightForm(initial_item: Option<crate::b2b::HighlightRecord>) -> impl IntoView {
    use crate::b2b::*;
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_item.is_some();
    let id_val = initial_item.as_ref().map(|p| p.id).unwrap_or(uuid::Uuid::nil());

    let (title, set_title) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.title.clone())
            .unwrap_or_default(),
    );
    let (url, set_url) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.url.clone())
            .unwrap_or_default(),
    );
    let (description, set_description) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.description.clone())
            .unwrap_or_default(),
    );
    let (image_url, set_image_url) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.image_url.clone())
            .unwrap_or_default(),
    );
    let (is_visible, set_is_visible) =
        create_signal(initial_item.as_ref().map(|p| p.is_visible).unwrap_or(true));
    let (display_order, set_display_order) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.display_order.to_string())
            .unwrap_or_else(|| "0".to_string()),
    );

    let save = move |_| {
        let t = title.get_untracked();
        let u = url.get_untracked();
        let d = description.get_untracked();
        let d_opt = if d.is_empty() { None } else { Some(d) };
        let iu = image_url.get_untracked();
        let iu_opt = if iu.is_empty() { None } else { Some(iu) };
        let iv = is_visible.get_untracked();
        let ord = display_order.get_untracked().parse::<i32>().unwrap_or(0);

        spawn_local(async move {
            if is_edit {
                let _ = update_highlight(id_val, t, u, iu_opt, d_opt, iv, ord).await;
            } else {
                let _ = add_highlight(t, u, iu_opt, d_opt, iv, ord).await;
            }
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Title"</label>
                    <input type="text" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"URL"</label>
                    <input type="text" prop:value=url on:input=move |ev| set_url.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="https://" />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Description"</label>
                <textarea prop:value=description on:input=move |ev| set_description.set(event_target_value(&ev)) rows="2" class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains resize-y"></textarea>
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Image URL"</label>
                <input type="text" prop:value=image_url on:input=move |ev| set_image_url.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="/assets/image.png" />
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Display Order"</label>
                    <input type="number" prop:value=display_order on:input=move |ev| set_display_order.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" />
                </div>
                <div class="flex items-center gap-3 pt-6">
                    <input type="checkbox" prop:checked=is_visible on:change=move |ev| set_is_visible.set(event_target_checked(&ev)) class="w-5 h-5 text-primary bg-surface border-outline-variant focus:ring-primary focus:ring-2" />
                    <div class="font-bold text-sm text-on-surface uppercase tracking-widest">"Visible"</div>
                </div>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                "SAVE HIGHLIGHT"
            </button>
        </div>
    }
}

// -----------------------------------------
// PageHeader Form
// -----------------------------------------
#[component]
pub fn PageHeaderForm(
    initial_item: Option<crate::components::dynamic_header::PageHeaderData>,
) -> impl IntoView {
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_item.is_some();
    let (route_path, set_route_path) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.route_path.clone())
            .unwrap_or_default(),
    );
    let (badge_text, set_badge_text) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.badge_text.clone())
            .unwrap_or_default(),
    );
    let (title, set_title) = create_signal(
        initial_item
            .as_ref()
            .map(|p| p.title.clone())
            .unwrap_or_default(),
    );
    let (subtitle, set_subtitle) = create_signal(
        initial_item
            .as_ref()
            .and_then(|p| p.subtitle.clone())
            .unwrap_or_default(),
    );

    let save = move |_| {
        let rp = route_path.get_untracked();
        let bt = badge_text.get_untracked();
        let bt_opt = if bt.is_empty() { None } else { Some(bt) };
        let t = title.get_untracked();
        let s = subtitle.get_untracked();
        let s_opt = if s.is_empty() { None } else { Some(s) };

        spawn_local(async move {
            if let Ok(_) =
                crate::components::dynamic_header::update_page_header(rp, bt_opt, t, s_opt).await
            {
                set_refresh.set(refresh.get_untracked() + 1);
                set_modal_state.set(ModalState::None);
            }
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Route Path (e.g. /projects)"</label>
                <input type="text" prop:value=route_path on:input=move |ev| set_route_path.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" disabled=is_edit />
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Title"</label>
                <input type="text" prop:value=title on:input=move |ev| set_title.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm font-bold" />
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Badge Text (Optional)"</label>
                <input type="text" prop:value=badge_text on:input=move |ev| set_badge_text.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm" />
            </div>

            <div class="flex flex-col gap-2">
                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Subtitle (Optional)"</label>
                <textarea prop:value=subtitle on:input=move |ev| set_subtitle.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 min-h-[100px] text-sm leading-relaxed font-sans"></textarea>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                "SAVE HEADER_NODE"
            </button>
        </div>
    }
}

// -----------------------------------------
// Lead Option Form
// -----------------------------------------
#[component]
pub fn LeadOptionForm(
    initial_item: Option<crate::pages::landing::LeadCaptureOption>,
) -> impl IntoView {
    let set_modal_state = expect_context::<WriteSignal<ModalState>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let refresh = expect_context::<ReadSignal<i32>>();

    let is_edit = initial_item.is_some();
    let id_val = initial_item.as_ref().map(|o| o.id);

    let (value_key, set_value_key) = create_signal(
        initial_item
            .as_ref()
            .map(|o| o.value_key.clone())
            .unwrap_or_default(),
    );
    let (label, set_label) = create_signal(
        initial_item
            .as_ref()
            .map(|o| o.label.clone())
            .unwrap_or_default(),
    );
    let (is_active, set_is_active) =
        create_signal(initial_item.as_ref().map(|o| o.is_active).unwrap_or(true));
    let (display_order, set_display_order) =
        create_signal(initial_item.as_ref().map(|o| o.display_order).unwrap_or(10));

    let save = move |_| {
        let pk = value_key.get_untracked();
        let l = label.get_untracked();
        let ia = is_active.get_untracked();
        let ord = display_order.get_untracked();

        spawn_local(async move {
            let _ = crate::pages::landing::upsert_lead_option(id_val, pk, l, ia, ord).await;
            set_refresh.set(refresh.get_untracked() + 1);
            set_modal_state.set(ModalState::None);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Option Key (No spaces)"</label>
                    <input type="text" prop:value=value_key on:input=move |ev| set_value_key.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono" placeholder="my_option_key" />
                </div>
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Display Label"</label>
                    <input type="text" prop:value=label on:input=move |ev| set_label.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="Front-End Text Display" />
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4 mt-4">
                <div class="flex flex-col gap-2">
                    <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Display Order"</label>
                    <input type="number" prop:value=display_order on:input=move |ev| set_display_order.set(event_target_value(&ev).parse().unwrap_or(0)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains font-mono" />
                </div>
                <div class="flex items-center gap-4 pt-6">
                    <label class="flex items-center space-x-3 cursor-pointer group">
                        <input type="checkbox"
                            class="w-5 h-5 bg-transparent border-2 border-outline-variant text-primary focus:ring-primary focus:ring-offset-surface-container-low"
                            prop:checked=is_active
                            on:change=move |ev| set_is_active.set(event_target_checked(&ev))
                        />
                        <span class="jetbrains text-sm font-bold tracking-widest text-on-surface uppercase group-hover:text-primary transition-colors">"Is Active?"</span>
                    </label>
                </div>
            </div>

            <button on:click=save class="mt-8 bg-primary text-on-primary font-bold jetbrains uppercase w-full py-4 tracking-widest hover:bg-primary-container transition-colors">
                {if is_edit { "UPDATE LEAD OPTION" } else { "CREATE LEAD OPTION" }}
            </button>
        </div>
    }
}
