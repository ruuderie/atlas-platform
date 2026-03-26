use leptos::prelude::*;
use shared_ui::components::icon::Icon;

#[derive(Clone, PartialEq)]
pub struct Partner {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: String,
}

#[component]
pub fn RecommendedPartners(
    #[prop(default = vec![])] partners: Vec<Partner>,
) -> impl IntoView {
    // Default fallback partners if none provided
    let display_partners = if partners.is_empty() {
        vec![
            Partner {
                name: "Atlas Intake Pros".to_string(),
                description: "Dedicated 24/7 legal receptionists to answer your leads instantly.".to_string(),
                url: "/admin/partners/intake-pros".to_string(),
                icon: "lucide-headset".to_string(),
            },
            Partner {
                name: "Local SEO Optimizer".to_string(),
                description: "Gain organic visibility and stop paying for individual leads.".to_string(),
                url: "/admin/partners/local-seo".to_string(),
                icon: "lucide-trending-up".to_string(),
            },
            Partner {
                name: "SMS Follow-Up CRM".to_string(),
                description: "Automate SMS chase sequences for unresponsive leads.".to_string(),
                url: "/admin/partners/sms-crm".to_string(),
                icon: "lucide-message-square".to_string(),
            },
        ]
    } else {
        partners
    };

    view! {
        <div class="bg-white rounded-lg border border-gray-200 shadow-sm p-4 w-full">
            <div class="flex items-center gap-2 mb-4 pb-2 border-b border-gray-100">
                <Icon name="lucide-briefcase" class="w-5 h-5 text-gray-400" />
                <h3 class="text-sm font-semibold text-gray-700 uppercase tracking-wider">
                    "Recommended Partners"
                </h3>
            </div>
            <div class="space-y-4">
                <For
                    each=move || display_partners.clone()
                    key=|partner| partner.name.clone()
                    children=move |partner| {
                        view! {
                            <a 
                                href=partner.url.clone() 
                                class="group block p-3 rounded-md hover:bg-gray-50 transition-colors border border-transparent hover:border-gray-100"
                            >
                                <div class="flex items-start gap-3">
                                    <div class="p-2 bg-gray-100 text-gray-600 rounded-md group-hover:bg-blue-50 group-hover:text-blue-600 transition-colors">
                                        <Icon name=partner.icon.clone() class="w-5 h-5" />
                                    </div>
                                    <div>
                                        <h4 class="text-sm font-medium text-gray-900 group-hover:text-blue-700 transition-colors">
                                            {partner.name.clone()}
                                        </h4>
                                        <p class="text-xs text-gray-500 mt-1 leading-relaxed">
                                            {partner.description.clone()}
                                        </p>
                                    </div>
                                </div>
                            </a>
                        }
                    }
                />
            </div>
        </div>
    }
}
