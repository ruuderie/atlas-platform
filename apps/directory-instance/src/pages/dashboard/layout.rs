use leptos::prelude::*;
use leptos_router::components::Outlet;
use crate::auth::AuthContext;
use crate::app::DirectoryConfig;

#[component]
pub fn DashboardLayout() -> impl IntoView {
    let auth = use_context::<AuthContext>().expect("AuthContext missing");
    let directory = use_context::<DirectoryConfig>().expect("DirectoryConfig missing");
    
    // Derived states
    let user_name = Signal::derive(move || {
        match auth.user.get() {
            Some(Ok(Some(u))) => format!("{} {}", u.first_name, u.last_name),
            _ => "Loading...".to_string()
        }
    });

    let accounts = Signal::derive(move || {
        match auth.accounts.get() {
            Some(Ok(accs)) => accs,
            _ => vec![]
        }
    });
    
    // Local state for active account switcher
    let show_dropdown = RwSignal::new(false);
    let selected_account_id = RwSignal::new(None::<String>); // Default to none, then set first

    // Side effect to set default account if not set
    Effect::new(move |_| {
        if selected_account_id.get().is_none() {
            if let Some(Ok(accs)) = auth.accounts.get() {
                if !accs.is_empty() {
                    selected_account_id.set(Some(accs[0].account.id.clone()));
                }
            }
        }
    });

    let active_account_name = Signal::derive(move || {
        let accs = accounts.get();
        if let Some(ref current_id) = selected_account_id.get() {
            if let Some(acc) = accs.iter().find(|a| &a.account.id == current_id) {
                return acc.account.name.clone();
            }
        }
        "Select Account".to_string()
    });

    let handle_logout = move |_| {
        crate::auth::clear_auth_token();
        window().location().set_href("/auth/login").unwrap();
    };

    view! {
        <div class="min-h-screen bg-surface-container-lowest flex bg-slate-50 relative">
            // Sidebar Navigation
            <aside class="w-64 bg-white border-r border-outline-variant/30 hidden md:flex flex-col h-screen sticky top-0">
                <div class="p-6 border-b border-outline-variant/30 flex items-center justify-between">
                    <a href="/" class="font-headline font-extrabold text-xl tracking-tight text-[#004289]">{directory.name.clone()}</a>
                </div>
                
                // Active Account Switcher
                <div class="p-4 relative">
                    <button class="w-full flex items-center justify-between p-3 rounded-xl border border-outline-variant/50 hover:bg-surface-container-lowest transition-colors text-left"
                            on:click=move |_| show_dropdown.update(|v| *v = !*v)>
                        <div class="flex items-center gap-3 overflow-hidden">
                            <div class="w-8 h-8 rounded bg-[#004289]/10 text-[#004289] flex items-center justify-center font-bold text-sm shrink-0">
                                {move || active_account_name.get().chars().next().unwrap_or('A').to_uppercase().to_string()}
                            </div>
                            <div class="truncate">
                                <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Active Account"</div>
                                <div class="text-sm font-bold text-on-surface truncate">{move || active_account_name.get()}</div>
                            </div>
                        </div>
                        <span class="material-symbols-outlined text-on-surface-variant">"unfold_more"</span>
                    </button>
                    
                    {move || match show_dropdown.get() {
                        true => view! {
                            <div class="absolute top-[80px] left-4 right-4 bg-white border border-outline-variant/50 shadow-premium rounded-xl overflow-hidden z-20 py-2">
                                <div class="px-3 py-2 text-[10px] font-bold text-on-surface-variant uppercase tracking-wider">"Your Accounts"</div>
                                {accounts.get().into_iter().map(|acc| {
                                    let id = acc.account.id.clone();
                                    let id2 = acc.account.id.clone();
                                    let is_active = move || selected_account_id.get().as_ref() == Some(&id2);
                                    view! {
                                        <button class="w-full flex items-center justify-between px-4 py-2 hover:bg-surface-container transition-colors text-left"
                                                on:click=move |_| {
                                                    selected_account_id.set(Some(id.clone()));
                                                    show_dropdown.set(false);
                                                }>
                                            <div class="flex flex-col">
                                                <span class="text-sm font-bold text-on-surface">{acc.account.name.clone()}</span>
                                                <span class="text-xs text-on-surface-variant">{acc.role.clone()}</span>
                                            </div>
                                            {move || if is_active() {
                                                view! { <span class="material-symbols-outlined text-[#004289] text-sm font-bold">"check"</span> }.into_any()
                                            } else { view! { <span/> }.into_any() }}
                                        </button>
                                    }
                                }).collect_view()}
                                <div class="border-t border-outline-variant/30 mt-2 pt-2">
                                    <button class="w-full flex items-center gap-2 px-4 py-2 text-sm text-[#004289] hover:bg-surface-container font-medium transition-colors">
                                        <span class="material-symbols-outlined text-[18px]">"add"</span>
                                        "Create New Account"
                                    </button>
                                </div>
                            </div>
                        }.into_any(),
                        false => view! { <span/> }.into_any()
                    }}
                </div>
                
                // Main Navigation
                <nav class="flex-1 px-4 py-4 space-y-1 overflow-y-auto">
                    {vec![
                        ("dashboard", "Overview", "/dashboard"),
                        ("list_alt", "My Listings", "/dashboard/listings"),
                        ("chat", "Leads & Messages", "/dashboard/leads"),
                        ("bar_chart", "Analytics", "/dashboard/analytics"),
                        ("settings", "Account Settings", "/dashboard/settings"),
                    ].into_iter().map(|(icon, label, path)| {
                        view! {
                            <a href=path class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-on-surface-variant hover:text-[#004289] hover:bg-[#004289]/5 font-medium transition-colors group">
                                <span class="material-symbols-outlined text-[20px] group-hover:text-[#004289] transition-colors">{icon}</span>
                                {label}
                            </a>
                        }
                    }).collect_view()}
                </nav>
                
                // User Profile Bottom
                <div class="p-4 border-t border-outline-variant/30">
                    <div class="flex items-center gap-3 mb-4 px-2">
                        <div class="w-10 h-10 rounded-full bg-surface-container overflow-hidden">
                            <img src="https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Avatar" class="w-full h-full object-cover"/>
                        </div>
                        <div class="flex-1 truncate">
                            <div class="text-sm font-bold text-on-surface truncate">{move || user_name.get()}</div>
                            <div class="text-xs text-on-surface-variant truncate">"Service Provider"</div>
                        </div>
                    </div>
                    <button class="w-full flex items-center justify-center gap-2 py-2 text-sm text-error hover:bg-error/5 rounded-lg font-bold transition-colors" on:click=handle_logout>
                        <span class="material-symbols-outlined text-[18px]">"logout"</span>
                        "Sign out"
                    </button>
                </div>
            </aside>
            
            // Mobile Header
            <div class="md:hidden fixed top-0 w-full bg-white border-b border-outline-variant/30 p-4 flex items-center justify-between z-30">
                <a href="/" class="font-headline font-extrabold text-lg text-[#004289]">{directory.name.clone()}</a>
                <button class="material-symbols-outlined text-on-surface">"menu"</button>
            </div>
            
            // Main Content Area
            <main class="flex-1 flex flex-col pt-[72px] md:pt-0 min-h-screen max-w-full overflow-x-hidden">
                <div class="p-8 md:p-12 max-w-7xl mx-auto w-full flex-1">
                    <Outlet />
                </div>
            </main>
        </div>
    }
}

#[component]
pub fn DashboardOverview() -> impl IntoView {
    view! {
        <div class="space-y-8 animate-fade-scale w-full">
            <h1 class="text-3xl font-headline font-extrabold text-on-surface tracking-tight">"Welcome to your Dashboard"</h1>
            
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-outline-variant/30">
                    <div class="w-12 h-12 rounded-xl bg-[#004289]/10 text-[#004289] flex items-center justify-center mb-4">
                        <span class="material-symbols-outlined text-2xl">"visibility"</span>
                    </div>
                    <div class="text-3xl font-headline font-bold text-on-surface mb-1">"1,248"</div>
                    <div class="text-sm font-bold text-on-surface-variant">"Profile Views (Last 30d)"</div>
                </div>
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-outline-variant/30">
                    <div class="w-12 h-12 rounded-xl bg-tertiary/10 text-tertiary flex items-center justify-center mb-4">
                        <span class="material-symbols-outlined text-2xl">"chat"</span>
                    </div>
                    <div class="text-3xl font-headline font-bold text-on-surface mb-1">"24"</div>
                    <div class="text-sm font-bold text-on-surface-variant">"New Leads"</div>
                </div>
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-outline-variant/30">
                    <div class="w-12 h-12 rounded-xl bg-emerald-100 text-emerald-600 flex items-center justify-center mb-4">
                        <span class="material-symbols-outlined text-2xl">"star"</span>
                    </div>
                    <div class="text-3xl font-headline font-bold text-on-surface mb-1">"4.9"</div>
                    <div class="text-sm font-bold text-on-surface-variant">"Average Rating"</div>
                </div>
            </div>

            <div class="bg-white rounded-2xl shadow-sm border border-outline-variant/30 overflow-hidden">
                <div class="p-6 border-b border-outline-variant/30 flex justify-between items-center">
                    <h2 class="text-xl font-headline font-bold text-on-surface">"Recent Leads"</h2>
                    <a href="#" class="text-sm font-bold text-[#004289] hover:underline">"View all"</a>
                </div>
                <div class="divide-y divide-outline-variant/30">
                    {vec![
                        ("Sarah L.", "Kitchen Remodel Inquiry", "2 hours ago", true),
                        ("Mike D.", "Plumbing emergency quote", "Yesterday", false),
                        ("Jennifer P.", "Handyman services needed", "2 days ago", false),
                    ].into_iter().map(|(name, topic, time, is_new)| {
                        view! {
                            <div class="p-6 hover:bg-surface-container-lowest transition-colors flex justify-between items-center group cursor-pointer">
                                <div class="flex items-center gap-4">
                                    <div class="w-10 h-10 rounded-full bg-surface-container flex items-center justify-center font-bold text-on-surface-variant">
                                        {name.chars().next().unwrap()}
                                    </div>
                                    <div>
                                        <div class="font-bold text-on-surface flex items-center gap-2">
                                            {name}
                                            {if is_new {
                                                view! { <span class="bg-[#004289] text-white text-[10px] px-2 py-0.5 rounded-full uppercase tracking-widest">"New"</span> }.into_any()
                                            } else { view! { <span/> }.into_any() }}
                                        </div>
                                        <div class="text-sm text-on-surface-variant font-medium">{topic}</div>
                                    </div>
                                </div>
                                <div class="flex items-center gap-4 text-sm text-on-surface-variant">
                                    <span class="hidden md:inline">{time}</span>
                                    <span class="material-symbols-outlined opacity-0 group-hover:opacity-100 transition-opacity">"chevron_right"</span>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
