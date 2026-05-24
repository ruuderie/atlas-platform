use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline::{CrmTimeline, CrmNote, CrmActivity};
use shared_ui::components::properties_editor::PropertiesEditor;
use shared_ui::utils::ResourceState;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ContactRecord {
    pub id: uuid::Uuid,
    pub customer_id: Option<uuid::Uuid>,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[server(GetNetworkContacts, "/api")]
pub async fn get_contacts() -> Result<Vec<ContactRecord>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let formatted = items.into_iter().map(|item| {
            let id = uuid::Uuid::parse_str(item.get("id").and_then(|v| v.as_str()).unwrap_or_default()).unwrap_or_default();
            let customer_id = item.get("customer_id").and_then(|v| v.as_str()).and_then(|s| uuid::Uuid::parse_str(s).ok());
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let first_name = item.get("first_name").and_then(|v| v.as_str()).map(String::from);
            let last_name = item.get("last_name").and_then(|v| v.as_str()).map(String::from);
            let email = item.get("email").and_then(|v| v.as_str()).map(String::from);
            let phone = item.get("phone").and_then(|v| v.as_str()).map(String::from);
            let whatsapp = item.get("whatsapp").and_then(|v| v.as_str()).map(String::from);
            let telegram = item.get("telegram").and_then(|v| v.as_str()).map(String::from);
            let twitter = item.get("twitter").and_then(|v| v.as_str()).map(String::from);
            let instagram = item.get("instagram").and_then(|v| v.as_str()).map(String::from);
            let facebook = item.get("facebook").and_then(|v| v.as_str()).map(String::from);
            let properties = item.get("properties").cloned();
            
            // Format dates
            let created_at_str = item.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|_| created_at_str.to_string());
                
            let updated_at_str = item.get("updated_at").and_then(|v| v.as_str()).unwrap_or_default();
            let updated_at = chrono::DateTime::parse_from_rfc3339(updated_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|_| updated_at_str.to_string());

            ContactRecord {
                id,
                customer_id,
                name,
                first_name,
                last_name,
                email,
                phone,
                whatsapp,
                telegram,
                twitter,
                instagram,
                facebook,
                properties,
                created_at,
                updated_at,
            }
        }).collect();
        Ok(formatted)
    } else {
        Err(ServerFnError::new("Failed to fetch contacts from backend"))
    }
}

#[server(DeleteNetworkContact, "/api")]
pub async fn delete_contact(id: uuid::Uuid) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}", api_base_url(), id);
    let client = reqwest::Client::new();
    let res = client
        .delete(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to delete contact"))
    }
}

#[server(GetNetworkContactCrmStatuses, "/api")]
pub async fn get_contact_crm_statuses() -> Result<Vec<CrmStatusOption>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/crm/status-options?object_type=Contact", api_base_url());
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let options = items.into_iter().map(|item| {
            CrmStatusOption {
                status_key: item.get("status_key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                label: item.get("label").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                color: item.get("color").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                sort_order: item.get("sort_order").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                is_system: item.get("is_system").and_then(|v| v.as_bool()).unwrap_or(false),
            }
        }).collect();
        Ok(options)
    } else {
        Err(ServerFnError::new("Failed to fetch contact crm statuses"))
    }
}

#[server(UpdateNetworkContactDetails, "/api")]
pub async fn update_contact_details(
    id: uuid::Uuid,
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    whatsapp: Option<String>,
    telegram: Option<String>,
    twitter: Option<String>,
    instagram: Option<String>,
    facebook: Option<String>,
    properties: Option<serde_json::Value>,
) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}", api_base_url(), id);
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "name": name,
        "first_name": first_name,
        "last_name": last_name,
        "email": email,
        "phone": phone,
        "whatsapp": whatsapp,
        "telegram": telegram,
        "twitter": twitter,
        "instagram": instagram,
        "facebook": facebook,
        "properties": properties
    });

    let res = client
        .put(&url)
        .header("Cookie", format!("session={}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to update contact details"))
    }
}

#[server(GetNetworkContactNotes, "/api")]
pub async fn get_contact_notes(contact_id: uuid::Uuid) -> Result<Vec<CrmNote>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}/notes", api_base_url(), contact_id);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let notes = items.into_iter().map(|item| {
            let created_at_str = item.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| created_at_str.to_string());

            CrmNote {
                id: item.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                content: item.get("content").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                created_at,
            }
        }).collect();
        Ok(notes)
    } else {
        Err(ServerFnError::new("Failed to fetch contact notes"))
    }
}

#[server(AddNetworkContactNote, "/api")]
pub async fn add_contact_note(contact_id: uuid::Uuid, content: String) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/notes", api_base_url());
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "entity_type": "Contact",
        "entity_id": contact_id,
        "content": content
    });

    let res = client
        .post(&url)
        .header("Cookie", format!("session={}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to add contact note"))
    }
}

#[server(GetNetworkContactActivities, "/api")]
pub async fn get_contact_activities(contact_id: uuid::Uuid) -> Result<Vec<CrmActivity>, ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/contacts/{}/activities", api_base_url(), contact_id);
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("Cookie", format!("session={}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        let items: Vec<serde_json::Value> = res.json().await.map_err(|e| ServerFnError::new(e.to_string()))?;
        let activities = items.into_iter().map(|item| {
            let created_at_str = item.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| created_at_str.to_string());

            CrmActivity {
                id: item.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                activity_type: item.get("activity_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                description: item.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                created_at,
            }
        }).collect();
        Ok(activities)
    } else {
        Err(ServerFnError::new("Failed to fetch contact activities"))
    }
}

#[server(LogNetworkContactActivity, "/api")]
pub async fn log_contact_activity(contact_id: uuid::Uuid, activity_type: String, description: String) -> Result<(), ServerFnError> {
    use axum::http::request::Parts;
    use crate::auth::api_base_url;

    let session_cookie = if let Some(req_parts) = use_context::<Parts>() {
        req_parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|cookies| {
                cookies.split(';').find_map(|part| {
                    let part = part.trim();
                    part.strip_prefix("session=").map(|t| t.to_string())
                })
            })
    } else {
        None
    };

    let Some(token) = session_cookie else {
        return Err(ServerFnError::new("Unauthorized"));
    };

    let url = format!("{}/api/activities", api_base_url());
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "contact_id": contact_id,
        "activity_type": activity_type.clone(),
        "title": format!("Logged {}", activity_type),
        "description": description,
        "status": "Completed"
    });

    let res = client
        .post(&url)
        .header("Cookie", format!("session={}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to log contact activity"))
    }
}

#[component]
pub fn ContactTable() -> impl IntoView {
    let (refresh, set_refresh) = signal(0);
    let contacts_res = Resource::new(move || refresh.get(), |_| get_contacts());
    let statuses_res = Resource::new(|| (), |_| get_contact_crm_statuses());

    let (selected_contact, set_selected_contact) = signal::<Option<ContactRecord>>(None);

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let res = contacts_res.get();
                let statuses = statuses_res.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <div class="relative w-full flex flex-col lg:flex-row gap-6">
                        // Table container
                        <div class="flex-1 overflow-x-auto bg-surface-container-lowest border border-outline-variant/30 rounded-xl p-6 shadow-sm">
                            <table class="w-full text-left jetbrains text-sm">
                                <thead>
                                    <tr class="text-outline border-b border-outline-variant/30 uppercase text-xs tracking-wider">
                                        <th class="py-4 px-4 font-semibold">"Name"</th>
                                        <th class="py-4 px-4 font-semibold">"Contact"</th>
                                        <th class="py-4 px-4 font-semibold">"Social channels"</th>
                                        <th class="py-4 px-4 font-semibold">"Status"</th>
                                        <th class="py-4 px-4 font-semibold text-right">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-outline-variant/20">
                                    {match ResourceState::from(res) {
                                        ResourceState::Ready(items) => {
                                            if items.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="5" class="py-12 text-center text-outline-variant">
                                                            "NO_ACTIVE_CONTACTS"
                                                        </td>
                                                    </tr>
                                                }.into_any()
                                            } else {
                                                items.into_iter().map(|contact| {
                                                    let c = contact.clone();
                                                    let email_disp = contact.email.clone().unwrap_or_else(|| "-".to_string());
                                                    let phone_disp = contact.phone.clone().unwrap_or_else(|| "-".to_string());
                                                    
                                                    // Extract status from properties JSON
                                                    let status_disp = contact.properties.as_ref()
                                                        .and_then(|p| p.get("status"))
                                                        .and_then(|s| s.as_str())
                                                        .unwrap_or("prospect")
                                                        .to_string();
                                                    
                                                    // Dynamic pipeline-based status badge styling
                                                    let matched_color = statuses.iter()
                                                        .find(|s| s.status_key.to_lowercase() == status_disp.to_lowercase())
                                                        .map(|s| s.color.as_str())
                                                        .unwrap_or("slate");
                                                        
                                                    let badge_classes = match matched_color {
                                                        "blue" => "bg-blue-500/10 text-blue-500 border-blue-500/20",
                                                        "purple" => "bg-purple-500/10 text-purple-500 border-purple-500/20",
                                                        "indigo" => "bg-indigo-500/10 text-indigo-500 border-indigo-500/20",
                                                        "orange" => "bg-orange-500/10 text-orange-500 border-orange-500/20",
                                                        "emerald" => "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
                                                        "rose" => "bg-rose-500/10 text-rose-500 border-rose-500/20",
                                                        _ => "bg-slate-500/10 text-slate-400 border-slate-500/20",
                                                    };

                                                    view! {
                                                        <tr 
                                                            class=move || format!(
                                                                "hover:bg-surface-container-high transition-all duration-150 cursor-pointer {}",
                                                                if selected_contact.get().map(|s| s.id) == Some(c.id) { "bg-surface-container-high border-l-4 border-primary" } else { "" }
                                                            )
                                                            on:click=move |_| set_selected_contact.set(Some(c.clone()))
                                                        >
                                                            <td class="py-4 px-4 font-bold text-primary">{contact.name}</td>
                                                            <td class="py-4 px-4">
                                                                <div class="text-xs text-outline">{email_disp}</div>
                                                                <div class="text-[10px] text-outline-variant">{phone_disp}</div>
                                                            </td>
                                                            <td class="py-4 px-4 text-xs font-mono text-outline-variant">
                                                                <div class="flex gap-2">
                                                                    <Show when=move || contact.twitter.is_some()>
                                                                        <span class="bg-surface-container px-1.5 py-0.5 rounded text-[10px]">"X"</span>
                                                                    </Show>
                                                                    <Show when=move || contact.whatsapp.is_some()>
                                                                        <span class="bg-emerald-500/10 text-emerald-600 px-1.5 py-0.5 rounded text-[10px]">"WA"</span>
                                                                    </Show>
                                                                    <Show when=move || contact.telegram.is_some()>
                                                                        <span class="bg-blue-500/10 text-blue-600 px-1.5 py-0.5 rounded text-[10px]">"TG"</span>
                                                                    </Show>
                                                                </div>
                                                            </td>
                                                            <td class="py-4 px-4">
                                                                <span class=format!("px-2 py-0.5 border rounded text-[10px] font-bold uppercase {}", badge_classes)>
                                                                    {status_disp}
                                                                </span>
                                                            </td>
                                                            <td class="py-4 px-4 text-right">
                                                                <button 
                                                                    on:click=move |e| {
                                                                        e.stop_propagation();
                                                                        let id = contact.id;
                                                                        leptos::task::spawn_local(async move {
                                                                            if let Ok(_) = delete_contact(id).await {
                                                                                set_refresh.set(refresh.get_untracked() + 1);
                                                                                if selected_contact.get().map(|s| s.id) == Some(id) {
                                                                                    set_selected_contact.set(None);
                                                                                }
                                                                            }
                                                                        });
                                                                    } 
                                                                    class="text-error hover:underline text-xs tracking-wider uppercase font-bold"
                                                                >
                                                                    "Drop"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>().into_any()
                                            }
                                        }
                                        ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                                        ResourceState::Error(_) => view! { <tr><td colspan="5" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                                    }}
                                </tbody>
                            </table>
                        </div>

                        // Detail Overlay Modal / Split CRM Panel
                        <Show when=move || selected_contact.get().is_some()>
                            <ContactCrmPane 
                                contact_record=selected_contact.get().unwrap() 
                                stages=statuses.clone()
                                on_close=Callback::new(move |_: ()| set_selected_contact.set(None))
                                set_refresh=set_refresh
                                refresh=refresh
                            />
                        </Show>
                    </div>
                }
            }.into_any()
            }
        </Transition>
    }
}

#[component]
fn ContactCrmPane(
    contact_record: ContactRecord,
    stages: Vec<CrmStatusOption>,
    on_close: Callback<()>,
    set_refresh: WriteSignal<i32>,
    refresh: ReadSignal<i32>,
) -> impl IntoView {
    // Extract status from properties JSON
    let status_val = contact_record.properties.as_ref()
        .and_then(|p| p.get("status"))
        .and_then(|s| s.as_str())
        .unwrap_or("prospect")
        .to_string();

    let (current_stage, set_current_stage) = signal(status_val);
    
    let contact_id = contact_record.id;
    let notes_res = Resource::new(move || refresh.get(), move |_| get_contact_notes(contact_id));
    let activities_res = Resource::new(move || refresh.get(), move |_| get_contact_activities(contact_id));

    // Field signals for standard details editing
    let (name, set_name) = signal(contact_record.name.clone());
    let (first_name, set_first_name) = signal(contact_record.first_name.clone().unwrap_or_default());
    let (last_name, set_last_name) = signal(contact_record.last_name.clone().unwrap_or_default());
    let (email, set_email) = signal(contact_record.email.clone().unwrap_or_default());
    let (phone, set_phone) = signal(contact_record.phone.clone().unwrap_or_default());
    let (whatsapp, set_whatsapp) = signal(contact_record.whatsapp.clone().unwrap_or_default());
    let (telegram, set_telegram) = signal(contact_record.telegram.clone().unwrap_or_default());
    let (twitter, set_twitter) = signal(contact_record.twitter.clone().unwrap_or_default());
    let (instagram, set_instagram) = signal(contact_record.instagram.clone().unwrap_or_default());
    let (facebook, set_facebook) = signal(contact_record.facebook.clone().unwrap_or_default());
    
    // Properties JSON RwSignal for PropertiesEditor
    let properties_signal = RwSignal::new(contact_record.properties.clone());
    
    let (edit_mode, set_edit_mode) = signal(false);
    let (save_error, set_save_error) = signal::<Option<String>>(None);

    let handle_stage_change = move |new_stage: String| {
        set_current_stage.set(new_stage.clone());
        let stage_cl = new_stage.clone();
        
        // Update status in properties JSON payload
        let mut props = properties_signal.get_untracked().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(ref mut map) = props {
            map.insert("status".to_string(), serde_json::Value::String(stage_cl.clone()));
        }
        properties_signal.set(Some(props.clone()));
        
        let n = name.get();
        let fn_val = Some(first_name.get()).filter(|s| !s.is_empty());
        let ln_val = Some(last_name.get()).filter(|s| !s.is_empty());
        let em_val = Some(email.get()).filter(|s| !s.is_empty());
        let ph_val = Some(phone.get()).filter(|s| !s.is_empty());
        let wa_val = Some(whatsapp.get()).filter(|s| !s.is_empty());
        let tg_val = Some(telegram.get()).filter(|s| !s.is_empty());
        let tw_val = Some(twitter.get()).filter(|s| !s.is_empty());
        let ig_val = Some(instagram.get()).filter(|s| !s.is_empty());
        let fb_val = Some(facebook.get()).filter(|s| !s.is_empty());

        leptos::task::spawn_local(async move {
            if let Ok(_) = update_contact_details(
                contact_id, n, fn_val, ln_val, em_val, ph_val, wa_val, tg_val, tw_val, ig_val, fb_val, Some(props)
            ).await {
                // Log timeline activity
                let _ = log_contact_activity(contact_id, "stage_change".to_string(), format!("Status transitioned to {}", stage_cl)).await;
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    };

    let handle_save_details = move |_| {
        let fn_val = first_name.get();
        let ln_val = last_name.get();
        if fn_val.is_empty() && ln_val.is_empty() {
            set_save_error.set(Some("First Name or Last Name is required".to_string()));
            return;
        }
        let n = format!("{} {}", fn_val, ln_val).trim().to_string();
        set_name.set(n.clone());
        set_save_error.set(None);

        let fn_opt = Some(fn_val).filter(|s| !s.is_empty());
        let ln_opt = Some(ln_val).filter(|s| !s.is_empty());
        let em_val = Some(email.get()).filter(|s| !s.is_empty());
        let ph_val = Some(phone.get()).filter(|s| !s.is_empty());
        let wa_val = Some(whatsapp.get()).filter(|s| !s.is_empty());
        let tg_val = Some(telegram.get()).filter(|s| !s.is_empty());
        let tw_val = Some(twitter.get()).filter(|s| !s.is_empty());
        let ig_val = Some(instagram.get()).filter(|s| !s.is_empty());
        let fb_val = Some(facebook.get()).filter(|s| !s.is_empty());
        
        // Include status in the saved properties JSON
        let mut props = properties_signal.get().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        if let serde_json::Value::Object(ref mut map) = props {
            map.insert("status".to_string(), serde_json::Value::String(current_stage.get_untracked()));
        }

        leptos::task::spawn_local(async move {
            match update_contact_details(
                contact_id, n, fn_opt, ln_opt, em_val, ph_val, wa_val, tg_val, tw_val, ig_val, fb_val, Some(props)
            ).await {
                Ok(_) => {
                    set_edit_mode.set(false);
                    set_refresh.set(refresh.get_untracked() + 1);
                }
                Err(e) => {
                    set_save_error.set(Some(format!("Save failed: {}", e)));
                }
            }
        });
    };

    let add_note_cb = Callback::new(move |text: String| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_contact_note(contact_id, text).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let log_activity_cb = Callback::new(move |(act_type, desc): (String, String)| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = log_contact_activity(contact_id, act_type, desc).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    view! {
        <div class="w-full lg:w-[480px] shrink-0 bg-surface-container p-6 rounded-xl border border-outline-variant/30 flex flex-col max-h-[85vh] overflow-y-auto shadow-lg relative animate-slide-in">
            // Header actions
            <div class="flex items-center justify-between border-b border-outline-variant/30 pb-4 mb-6">
                <div>
                    <span class="text-[9px] font-bold tracking-widest text-outline-variant uppercase jetbrains">"CONTACT_CRM_PROFILE"</span>
                    <h3 class="text-lg font-bold text-on-surface flex items-center gap-2 mt-0.5">
                        {move || name.get()}
                    </h3>
                </div>
                <button on:click=move |_| on_close.run(()) class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface">
                    <span class="material-symbols-outlined text-sm">"close"</span>
                </button>
            </div>

            // Chevron Pipeline Stage Bar
            <div class="mb-6">
                <label class="block text-[10px] jetbrains uppercase text-outline mb-2">"Relationship Status"</label>
                <CrmStageBar
                    stages=stages
                    current_stage=current_stage.into()
                    on_stage_change=handle_stage_change
                />
            </div>

            // Details Section
            <div class="space-y-4 mb-6">
                <div class="flex justify-between items-center border-b border-outline-variant/15 pb-2">
                    <span class="text-[10px] jetbrains font-bold uppercase text-outline">"Information details"</span>
                    <button
                        on:click=move |_| set_edit_mode.update(|m| *m = !*m)
                        class="text-primary hover:underline text-[10px] jetbrains font-bold uppercase tracking-wider"
                    >
                        {move || if edit_mode.get() { "Cancel" } else { "Edit Details" }}
                    </button>
                </div>

                <Show
                    when=move || edit_mode.get()
                    fallback=move || view! {
                        <div class="grid grid-cols-2 gap-4 text-xs font-mono bg-surface-container-lowest p-4 rounded-lg border border-outline-variant/10">
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"First Name"</span>
                                <span class="text-on-surface font-semibold">{move || if first_name.get().is_empty() { "-".to_string() } else { first_name.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Last Name"</span>
                                <span class="text-on-surface font-semibold">{move || if last_name.get().is_empty() { "-".to_string() } else { last_name.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Email"</span>
                                <span class="text-on-surface font-semibold break-all">{move || if email.get().is_empty() { "-".to_string() } else { email.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Phone"</span>
                                <span class="text-on-surface font-semibold">{move || if phone.get().is_empty() { "-".to_string() } else { phone.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"WhatsApp"</span>
                                <span class="text-on-surface font-semibold">{move || if whatsapp.get().is_empty() { "-".to_string() } else { whatsapp.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Telegram"</span>
                                <span class="text-on-surface font-semibold">{move || if telegram.get().is_empty() { "-".to_string() } else { telegram.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Twitter / X"</span>
                                <span class="text-on-surface font-semibold">{move || if twitter.get().is_empty() { "-".to_string() } else { twitter.get() }}</span>
                            </div>
                            <div>
                                <span class="text-outline-variant text-[10px] block uppercase">"Instagram"</span>
                                <span class="text-on-surface font-semibold">{move || if instagram.get().is_empty() { "-".to_string() } else { instagram.get() }}</span>
                            </div>
                        </div>
                    }
                >
                    <div class="space-y-3 bg-surface-container-lowest p-4 rounded-lg border border-outline-variant/20">
                        <div class="grid grid-cols-2 gap-3">
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"First Name *"</label>
                                <input 
                                    type="text" 
                                    prop:value=first_name
                                    on:input=move |ev| set_first_name.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Last Name"</label>
                                <input 
                                    type="text" 
                                    prop:value=last_name
                                    on:input=move |ev| set_last_name.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Email"</label>
                                <input 
                                    type="email" 
                                    prop:value=email
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Phone"</label>
                                <input 
                                    type="text" 
                                    prop:value=phone
                                    on:input=move |ev| set_phone.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"WhatsApp"</label>
                                <input 
                                    type="text" 
                                    prop:value=whatsapp
                                    on:input=move |ev| set_whatsapp.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Telegram"</label>
                                <input 
                                    type="text" 
                                    prop:value=telegram
                                    on:input=move |ev| set_telegram.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Twitter / X"</label>
                                <input 
                                    type="text" 
                                    prop:value=twitter
                                    on:input=move |ev| set_twitter.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Instagram"</label>
                                <input 
                                    type="text" 
                                    prop:value=instagram
                                    on:input=move |ev| set_instagram.set(event_target_value(&ev))
                                    class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                />
                            </div>
                        </div>
                        <Show when=move || save_error.get().is_some()>
                            <div class="bg-error/10 border-l-4 border-error p-3 jetbrains text-xs text-error font-medium">
                                {move || save_error.get().unwrap_or_default()}
                            </div>
                        </Show>
                        <div class="flex justify-end">
                            <button
                                on:click=handle_save_details
                                class="bg-primary text-on-primary px-4 py-2 text-xs jetbrains font-bold uppercase tracking-wider hover:bg-primary-container rounded"
                            >
                                "Save Changes"
                            </button>
                        </div>
                    </div>
                </Show>
            </div>

            // Reusable Headless Custom Properties Editor (JSON-based metadata)
            <div class="border-t border-outline-variant/30 pt-6 mb-6">
                <PropertiesEditor
                    properties=properties_signal
                />
            </div>

            // Timeline (Notes & Activities)
            <div class="border-t border-outline-variant/30 pt-6">
                <CrmTimeline
                    notes=Signal::derive(move || notes_res.get().and_then(|r| r.ok()).unwrap_or_default())
                    activities=Signal::derive(move || activities_res.get().and_then(|r| r.ok()).unwrap_or_default())
                    on_add_note=add_note_cb
                    on_log_activity=log_activity_cb
                />
            </div>
        </div>
    }
}
