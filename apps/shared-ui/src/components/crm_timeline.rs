use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CrmNote {
    pub id: String,
    pub content: String,
    pub created_at: String, // Pre-formatted or ISO string
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CrmActivity {
    pub id: String,
    pub activity_type: String, // "call", "email", "meeting", "task", etc.
    pub description: String,
    pub created_at: String, // Pre-formatted or ISO string
}

#[component]
pub fn CrmTimeline(
    notes: Signal<Vec<CrmNote>>,
    activities: Signal<Vec<CrmActivity>>,
    #[prop(into)] on_add_note: Callback<String>,
    #[prop(into)] on_log_activity: Callback<(String, String)>, // (activity_type, description)
) -> impl IntoView {
    let (note_text, set_note_text) = signal(String::new());
    let (activity_type, set_activity_type) = signal("call".to_string());
    let (activity_desc, set_activity_desc) = signal(String::new());

    let (active_tab, set_active_tab) = signal("note"); // "note" or "activity"

    let submit_note = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let text = note_text.get();
        if !text.trim().is_empty() {
            on_add_note.run(text);
            set_note_text.set(String::new());
        }
    };

    let submit_activity = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let desc = activity_desc.get();
        let act_type = activity_type.get();
        if !desc.trim().is_empty() {
            on_log_activity.run((act_type, desc));
            set_activity_desc.set(String::new());
        }
    };

    // Helper to render activity/note icon
    let get_icon = |activity_type: &str| -> &'static str {
        match activity_type.to_lowercase().as_str() {
            "call" => "call",
            "email" => "mail",
            "meeting" => "calendar_today",
            "task" => "task_alt",
            "stage_change" => "analytics",
            "conversion" => "celebration",
            _ => "chat_bubble",
        }
    };

    // Helper to render activity icon color
    let get_icon_color = |activity_type: &str| -> &'static str {
        match activity_type.to_lowercase().as_str() {
            "call" => "bg-blue-500/10 text-blue-500 border-blue-500/30",
            "email" => "bg-purple-500/10 text-purple-500 border-purple-500/30",
            "meeting" => "bg-indigo-500/10 text-indigo-500 border-indigo-500/30",
            "task" => "bg-orange-500/10 text-orange-500 border-orange-500/30",
            "stage_change" => "bg-emerald-500/10 text-emerald-500 border-emerald-500/30",
            "conversion" => "bg-amber-500/10 text-amber-500 border-amber-500/30",
            _ => "bg-slate-500/10 text-slate-500 border-slate-500/30",
        }
    };

    // Combines notes and activities into a single chronologically sorted vector of feed items
    #[derive(Clone, PartialEq)]
    enum FeedItem {
        Note(CrmNote),
        Activity(CrmActivity),
    }

    let combined_feed = move || {
        let mut feed = Vec::new();
        for n in notes.get() {
            feed.push((n.created_at.clone(), FeedItem::Note(n)));
        }
        for a in activities.get() {
            feed.push((a.created_at.clone(), FeedItem::Activity(a)));
        }
        // Sort descending (newest first)
        feed.sort_by(|a, b| b.0.cmp(&a.0));
        feed.into_iter().map(|(_, item)| item).collect::<Vec<_>>()
    };

    view! {
        <div class="space-y-6">
            // Tab Selection for Composer
            <div class="border-b border-outline-variant/30 flex gap-4">
                <button
                    on:click=move |_| set_active_tab.set("note")
                    class=move || format!(
                        "pb-2 text-xs jetbrains font-bold uppercase tracking-wider transition-colors border-b-2 {}",
                        if active_tab.get() == "note" { "border-primary text-primary" } else { "border-transparent text-outline hover:text-on-surface" }
                    )
                >
                    "Add Note"
                </button>
                <button
                    on:click=move |_| set_active_tab.set("activity")
                    class=move || format!(
                        "pb-2 text-xs jetbrains font-bold uppercase tracking-wider transition-colors border-b-2 {}",
                        if active_tab.get() == "activity" { "border-primary text-primary" } else { "border-transparent text-outline hover:text-on-surface" }
                    )
                >
                    "Log Activity"
                </button>
            </div>

            // Composers
            <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 shadow-inner">
                <Show
                    when=move || active_tab.get() == "note"
                    fallback=move || view! {
                        <form on:submit=submit_activity class="space-y-4">
                            <div class="flex gap-4 items-center">
                                <div class="flex-1">
                                    <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Activity Type"</label>
                                    <select
                                        prop:value=activity_type
                                        on:change=move |ev| set_activity_type.set(event_target_value(&ev))
                                        class="bg-surface-container-low border border-outline-variant/30 px-3 py-2 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                                    >
                                        <option value="call">"Phone Call"</option>
                                        <option value="email">"Email Sent/Received"</option>
                                        <option value="meeting">"Meeting Held"</option>
                                        <option value="task">"Task Completed"</option>
                                    </select>
                                </div>
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Description"</label>
                                <textarea
                                    prop:value=activity_desc
                                    on:input=move |ev| set_activity_desc.set(event_target_value(&ev))
                                    placeholder="e.g. Discussed proposal, scheduled a follow-up call next week..."
                                    rows="3"
                                    class="w-full bg-surface-container-low border border-outline-variant/30 p-3 text-sm focus:outline-none focus:border-primary text-on-surface resize-none rounded"
                                ></textarea>
                            </div>
                            <div class="flex justify-end">
                                <button
                                    type="submit"
                                    class="bg-primary text-on-primary px-4 py-2 text-xs jetbrains font-bold uppercase tracking-wider hover:bg-primary-container transition-colors rounded"
                                >
                                    "Log Activity"
                                </button>
                            </div>
                        </form>
                    }
                >
                    <form on:submit=submit_note class="space-y-4">
                        <div>
                            <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Note Content"</label>
                            <textarea
                                prop:value=note_text
                                on:input=move |ev| set_note_text.set(event_target_value(&ev))
                                placeholder="Type a note (Markdown supported)..."
                                rows="3"
                                class="w-full bg-surface-container-low border border-outline-variant/30 p-3 text-sm focus:outline-none focus:border-primary text-on-surface resize-none rounded"
                            ></textarea>
                        </div>
                        <div class="flex justify-end">
                            <button
                                type="submit"
                                class="bg-primary text-on-primary px-4 py-2 text-xs jetbrains font-bold uppercase tracking-wider hover:bg-primary-container transition-colors rounded"
                            >
                                "Save Note"
                            </button>
                        </div>
                    </form>
                </Show>
            </div>

            // Timeline Feed
            <div class="relative pl-6 border-l-2 border-outline-variant/20 ml-3 space-y-6 pt-2">
                <For
                    each=combined_feed
                    key=|item| match item {
                        FeedItem::Note(n) => format!("note-{}", n.id),
                        FeedItem::Activity(a) => format!("act-{}", a.id),
                    }
                    children=move |item| {
                        match item {
                            FeedItem::Note(note) => {
                                view! {
                                    <div class="relative group">
                                        // Bullet circle icon
                                        <div class="absolute -left-[35px] top-1.5 w-6 h-6 rounded-full flex items-center justify-center border bg-surface-container-lowest text-primary border-outline-variant/40">
                                            <span class="material-symbols-outlined text-sm">"description"</span>
                                        </div>
                                        // Box
                                        <div class="bg-surface-container-lowest border border-outline-variant/25 p-4 rounded-lg shadow-sm hover:shadow-md transition-shadow">
                                            <div class="flex justify-between items-center mb-2">
                                                <span class="text-xs font-bold text-primary jetbrains uppercase tracking-wide">"NOTE"</span>
                                                <span class="text-[10px] text-outline font-semibold jetbrains">{note.created_at.clone()}</span>
                                            </div>
                                            <p class="text-sm text-on-surface whitespace-pre-wrap leading-relaxed">{note.content.clone()}</p>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            FeedItem::Activity(act) => {
                                let icon = get_icon(&act.activity_type);
                                let icon_color = get_icon_color(&act.activity_type);
                                let label = act.activity_type.clone();

                                view! {
                                    <div class="relative group">
                                        // Bullet circle icon
                                        <div class=format!("absolute -left-[35px] top-1.5 w-6 h-6 rounded-full flex items-center justify-center border {}", icon_color)>
                                            <span class="material-symbols-outlined text-sm">{icon}</span>
                                        </div>
                                        // Box
                                        <div class="bg-surface-container-lowest border border-outline-variant/25 p-4 rounded-lg shadow-sm hover:shadow-md transition-shadow">
                                            <div class="flex justify-between items-center mb-2">
                                                <span class="text-xs font-bold text-secondary-container text-secondary jetbrains uppercase tracking-wide">
                                                    {label}
                                                </span>
                                                <span class="text-[10px] text-outline font-semibold jetbrains">{act.created_at.clone()}</span>
                                            </div>
                                            <p class="text-sm text-on-surface leading-relaxed">{act.description.clone()}</p>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }
                />

                <Show when=move || combined_feed().is_empty()>
                    <div class="text-center py-8 text-outline text-xs jetbrains">
                        "NO_TIMELINE_ENTRIES_YET"
                    </div>
                </Show>
            </div>
        </div>
    }
}
