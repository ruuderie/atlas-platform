use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;

use crate::pages::dashboard::Dashboard;
use crate::pages::multi_site::MultiSite;
use crate::pages::crm_grid::CrmGrid;
use crate::pages::cms_editor::CmsEditor;

use shared_ui::components::ui::header::Header;
use shared_ui::components::ui::select::{Select, SelectTrigger, SelectValue, SelectContent, SelectGroup, SelectLabel, SelectOption};
use shared_ui::components::ui::avatar::{Avatar, AvatarImage, AvatarFallback};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="flex h-screen w-full bg-background text-foreground">
                <aside class="w-64 border-r border-border bg-card flex flex-col">
                    <div class="h-16 flex items-center px-6 border-b border-border">
                        <h2 class="text-lg font-semibold text-primary">"Admin Portal"</h2>
                    </div>
                    <nav class="flex-1 p-4 space-y-2">
                        <a href="/" class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Platform Overview"</a>
                        <a href="/sites" class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Network Directories"</a>
                        <a href="/crm" class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Sales & Relationships"</a>
                        <a href="/cms" class="block px-4 py-2 text-sm rounded-md hover:bg-secondary hover:text-secondary-foreground transition-colors">"Content Management"</a>
                    </nav>
                </aside>
                
                <div class="flex-1 flex flex-col overflow-hidden">
                    <Header>
                        <div class="h-16 flex items-center justify-between px-6 border-b border-border bg-card/50 backdrop-blur-sm relative z-50">
                            <div class="flex items-center space-x-4">
                                <span class="text-sm font-medium text-muted-foreground">"Active Site:"</span>
                                <div class="w-[240px]">
                                    <Select default_value="all".to_string()>
                                        <SelectTrigger id="site-selector".to_string() class="w-full bg-background relative z-50">
                                            <SelectValue placeholder="Select a site...".to_string() />
                                        </SelectTrigger>
                                        <SelectContent class="z-[100] mt-1 bg-popover shadow-md border rounded-md">
                                            <SelectGroup>
                                                <SelectLabel>"Global"</SelectLabel>
                                                <SelectOption value="all".to_string()>"Global (All Sites)"</SelectOption>
                                            </SelectGroup>
                                            <SelectGroup>
                                                <SelectLabel>"Sites"</SelectLabel>
                                                <SelectOption value="transportation".to_string()>"Transportation Directory"</SelectOption>
                                                <SelectOption value="healthcare".to_string()>"Healthcare Directory"</SelectOption>
                                            </SelectGroup>
                                        </SelectContent>
                                    </Select>
                                </div>
                            </div>
                            <div class="flex items-center space-x-3">
                                <span class="text-sm font-medium">"Admin"</span>
                                <Avatar class="w-8 h-8".to_string()>
                                    <AvatarFallback>"A"</AvatarFallback>
                                </Avatar>
                            </div>
                        </div>
                    </Header>
                    
                    <main class="flex-1 overflow-auto p-8 relative z-0">
                        <Routes fallback=|| "Not found.">
                            <Route path=path!("/") view=Dashboard />
                            <Route path=path!("/sites") view=MultiSite />
                            <Route path=path!("/sites/:id") view=crate::pages::site_dashboard::SiteDashboard />
                            <Route path=path!("/crm") view=CrmGrid />
                            <Route path=path!("/crm/:entity/:id") view=crate::pages::crm_detail::CrmDetail />
                            <Route path=path!("/cms") view=CmsEditor />
                        </Routes>
                    </main>
                </div>
            </div>
        </Router>
    }
}
