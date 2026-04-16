use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileHeaderBlockData {
    pub full_name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub objective: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub contact: ProfileContact,
    #[serde(default)]
    pub badges: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ProfileContact {
    #[serde(default)] pub email: Option<String>,
    #[serde(default)] pub phone: Option<String>,
    #[serde(default)] pub location: Option<String>,
    #[serde(default)] pub github_url: Option<String>,
    #[serde(default)] pub linkedin_url: Option<String>,
    #[serde(default)] pub website_url: Option<String>,
}

#[component]
pub fn ProfileHeaderBlock(data: ProfileHeaderBlockData) -> impl IntoView {
    view! {
        <section class="w-full bg-surface-container-lowest border-b border-outline-variant/30 pt-16 pb-12">
            <div class="container mx-auto px-4 max-w-4xl">
                <div class="flex flex-col md:flex-row gap-8 items-start">
                    {if let Some(avatar) = data.avatar_url {
                        view! {
                            <div class="h-32 w-32 md:h-48 md:w-48 shrink-0 rounded-2xl overflow-hidden border-4 border-surface shadow-lg relative group">
                                <div class="absolute inset-0 bg-primary/20 mix-blend-overlay group-hover:opacity-0 transition-opacity z-10" />
                                <img src={avatar} alt={data.full_name.clone()} class="h-full w-full object-cover filter grayscale group-hover:grayscale-0 transition-all duration-500" />
                            </div>
                        }.into_view()
                    } else { view! {}.into_view() }}
                    
                    <div class="flex-grow space-y-4">
                        <div>
                            <h1 class="text-4xl md:text-5xl font-black text-on-surface tracking-tight mb-2">
                                {data.full_name}
                            </h1>
                            {if let Some(title) = data.title {
                                view! { <div class="text-xl md:text-2xl text-primary font-medium">{title}</div> }.into_view()
                            } else { view! {}.into_view() }}
                        </div>

                        {if data.contact != ProfileContact::default() {
                            view! {
                                <div class="flex flex-wrap gap-x-6 gap-y-2 text-sm text-on-surface-variant pt-2">
                                    {if let Some(location) = data.contact.location {
                                        view! { <span class="flex items-center gap-1.5"><span class="material-symbols-outlined text-[1rem]">"location_on"</span>{location}</span> }.into_view()
                                    } else { view! {}.into_view() }}
                                    
                                    {if let Some(email) = data.contact.email {
                                        view! { <a href=format!("mailto:{}", email) class="flex items-center gap-1.5 hover:text-primary transition-colors"><span class="material-symbols-outlined text-[1rem]">"mail"</span>{email}</a> }.into_view()
                                    } else { view! {}.into_view() }}
                                    
                                    {if let Some(github) = data.contact.github_url {
                                        view! { <a href={github} target="_blank" class="flex items-center gap-1.5 hover:text-primary transition-colors hover:underline">"GitHub"</a> }.into_view()
                                    } else { view! {}.into_view() }}
                                    
                                    {if let Some(linkedin) = data.contact.linkedin_url {
                                        view! { <a href={linkedin} target="_blank" class="flex items-center gap-1.5 hover:text-primary transition-colors hover:underline">"LinkedIn"</a> }.into_view()
                                    } else { view! {}.into_view() }}
                                </div>
                            }.into_view()
                        } else { view! {}.into_view() }}

                        {if let Some(objective) = data.objective {
                            view! {
                                <p class="text-on-surface-variant text-base md:text-lg leading-relaxed pt-2 max-w-3xl">
                                    {objective}
                                </p>
                            }.into_view()
                        } else { view! {}.into_view() }}

                        {if !data.badges.is_empty() {
                            view! {
                                <div class="flex flex-wrap gap-2 pt-4">
                                    {data.badges.into_iter().map(|badge| view! {
                                        <span class="px-3 py-1 bg-surface-container-high rounded-full text-xs font-semibold text-on-surface tracking-wide uppercase border border-outline/10">
                                            {badge}
                                        </span>
                                    }).collect_view()}
                                </div>
                            }.into_view()
                        } else { view! {}.into_view() }}
                    </div>
                </div>
            </div>
        </section>
    }
}
