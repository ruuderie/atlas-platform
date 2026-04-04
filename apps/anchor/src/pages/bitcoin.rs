use leptos::*;
use leptos_meta::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BitcoinBlockRecord {
    pub id: String,
    pub height: i64,
    pub timestamp: i64,
    pub tx_count: i32,
    pub size: i32,
    pub weight: i32,
    pub difficulty: f64,
}

#[server(GetBitcoinBlocks, "/api")]
pub async fn get_bitcoin_blocks(limit: i64) -> Result<Vec<BitcoinBlockRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = sqlx::query(
        "SELECT id, height, timestamp, tx_count, size, weight, difficulty 
         FROM bitcoin_blocks 
         ORDER BY height DESC 
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;

    let mut blocks = Vec::new();
    for row in rows {
        blocks.push(BitcoinBlockRecord {
            id: row.get("id"),
            height: row.get("height"),
            timestamp: row.get("timestamp"),
            tx_count: row.get("tx_count"),
            size: row.get("size"),
            weight: row.get("weight"),
            difficulty: row.get("difficulty"),
        });
    }

    Ok(blocks)
}

#[component]
pub fn BitcoinDashboard() -> impl IntoView {
    let blocks_resource = create_resource(|| (), |_| get_bitcoin_blocks(15));

    view! {
        <Title text="Bitcoin // The Mechanical Reality"/>
        <div class="min-h-screen bg-surface text-on-surface font-sans selection:bg-secondary-container selection:text-on-secondary-container pt-32 pb-20 relative overflow-hidden">
            // Background effect
            <div class="absolute inset-0 z-0 opacity-20 dark:opacity-10 pointer-events-none"
                style="background-image: linear-gradient(var(--color-outline-variant) 1px, transparent 1px), linear-gradient(90deg, var(--color-outline-variant) 1px, transparent 1px); background-size: 40px 40px;">
            </div>

            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10 pt-16">

                // Hero Section
                <div class="mb-20 grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
                    <div>
                        <div class="inline-block px-3 py-1 mb-6 bg-surface-container-high border border-outline-variant text-on-surface text-xs font-mono tracking-widest uppercase shadow-sm flex items-center w-max">
                            <span class="w-2 h-2 rounded-full bg-[#f7931a] animate-pulse mr-2"></span>
                            "System Synced"
                        </div>
                        <h1 class="text-5xl lg:text-7xl font-bold text-primary tracking-tighter leading-[1.1] mb-6 drop-shadow-sm dark:drop-shadow-lg uppercase">
                            "The Mechanical"<br/>
                            <span class="text-[#f7931a]">"Reality of Bitcoin."</span>
                        </h1>
                        <p class="text-xl text-on-surface-variant font-medium leading-relaxed max-w-xl border-l-4 border-secondary/50 pl-5">
                            "The least interesting aspect of Bitcoin is its price. Beyond the speculation lies an immutable, globally synchronized cryptosystem marching forward unconditionally."
                        </p>
                    </div>
                </div>

                // Blocks Data
                <Suspense fallback=move || view! {
                    <div class="animate-pulse w-full h-[600px] bg-surface-container rounded-sm border border-outline-variant flex items-center justify-center">
                        <span class="text-outline font-mono tracking-widest uppercase">"Decrypting Ledger..."</span>
                    </div>
                }>
                    {move || {
                        let blocks = blocks_resource.get().unwrap_or(Ok(vec![])).unwrap_or_default();

                        if blocks.is_empty() {
                            view! {
                                <div class="p-8 text-center text-on-surface-variant font-mono bg-surface-container rounded-sm border border-outline-variant">
                                    "No blocks synchronized yet. The system is initializing."
                                </div>
                            }.into_view()
                        } else {
                            let latest = blocks[0].clone();
                            let difficulty_trillions = latest.difficulty / 1_000_000_000_000.0;

                            view! {
                                <div class="space-y-12">
                                    // Latest Block Stats Grid
                                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                                        <StatCard title="LATEST HEIGHT" value=format!("#{}", latest.height) icon="deployed_code" color="text-primary" />
                                        <StatCard title="NETWORK DIFFICULTY" value=format!("{:.2} T", difficulty_trillions) icon="trending_up" color="text-[#f7931a]" />
                                        <StatCard title="BLOCK SIZE" value=format!("{:.2} MB", latest.size as f64 / 1_000_000.0) icon="data_exploration" color="text-secondary" />
                                        <StatCard title="TRANSACTIONS" value=format!("{}", latest.tx_count) icon="swap_horiz" color="text-tertiary" />
                                    </div>

                                    // Immutable Chain Timeline
                                    <div class="bg-surface-container-low shadow-sm border border-outline-variant p-6 lg:p-10 overflow-hidden relative">
                                        <div class="absolute top-0 right-0 p-32 opacity-[0.02] dark:opacity-5 pointer-events-none select-none text-[30rem] leading-none material-symbols-outlined font-black">
                                            "link"
                                        </div>
                                        <h2 class="text-sm font-bold tracking-[0.2em] text-on-surface-variant uppercase mb-8 font-mono border-b border-outline-variant/50 pb-4">
                                            "Immutable Chain Sequence"
                                        </h2>

                                        <div class="overflow-x-auto">
                                            <table class="w-full text-left whitespace-nowrap">
                                                <thead>
                                                    <tr class="text-xs text-outline uppercase tracking-widest font-mono border-b border-outline-variant">
                                                        <th class="py-4 pr-6 font-medium">"Height"</th>
                                                        <th class="py-4 px-6 font-medium">"Hash Identifier"</th>
                                                        <th class="py-4 px-6 font-medium text-right">"Mined At"</th>
                                                        <th class="py-4 px-6 font-medium text-right">"Transactions"</th>
                                                        <th class="py-4 pl-6 font-medium text-right">"Size"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-outline-variant/30">
                                                    {blocks.into_iter().enumerate().map(|(idx, block)| {
                                                        // Convert timestamp to naive datetime string
                                                        let date_str = chrono::DateTime::from_timestamp(block.timestamp, 0)
                                                            .unwrap_or_default()
                                                            .format("%Y-%m-%d %H:%M:%S UTC").to_string();

                                                        let is_latest = idx == 0;

                                                        view! {
                                                            <tr class=format!("group hover:bg-surface-container transition-colors {}", if is_latest { "bg-primary/5" } else { "" })>
                                                                <td class="py-4 pr-6 font-mono">
                                                                    <div class="flex items-center">
                                                                        {if is_latest {
                                                                            view! { <span class="w-1.5 h-1.5 rounded-full bg-primary mr-2 shadow-sm"></span> }.into_view()
                                                                        } else {
                                                                            view! { <span class="w-1.5 h-1.5 rounded-full bg-outline-variant mr-2"></span> }.into_view()
                                                                        }}
                                                                        <span class=if is_latest { "text-primary font-bold" } else { "text-on-surface" }>
                                                                            {block.height}
                                                                        </span>
                                                                    </div>
                                                                </td>
                                                                <td class="py-4 px-6">
                                                                    <div class="font-mono text-[11px] text-on-surface-variant group-hover:text-on-surface transition-colors w-48 md:w-auto truncate md:overflow-visible">
                                                                        {block.id}
                                                                    </div>
                                                                </td>
                                                                <td class="py-4 px-6 text-right font-mono text-xs text-on-surface-variant">
                                                                    {date_str}
                                                                </td>
                                                                <td class="py-4 px-6 text-right font-mono text-on-surface">
                                                                    {block.tx_count}
                                                                </td>
                                                                <td class="py-4 pl-6 text-right font-mono text-on-surface">
                                                                    {format!("{:.2} MB", block.size as f64 / 1_000_000.0)}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn StatCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    color: &'static str,
) -> impl IntoView {
    view! {
        <div class="bg-surface-container-low shadow-sm border border-outline-variant p-6 flex flex-col hover:-translate-y-1 hover:shadow-md transition-all duration-300 group">
            <div class="flex items-center justify-between mb-4">
                <span class="text-[0.65rem] font-bold tracking-[0.2em] text-outline uppercase font-mono group-hover:text-primary transition-colors">{title}</span>
                <span class=format!("material-symbols-outlined text-[1.2rem] opacity-70 group-hover:opacity-100 transition-opacity {}", color)>{icon}</span>
            </div>
            <div class=format!("text-2xl lg:text-3xl font-bold font-mono tracking-tight {}", color)>
                {value}
            </div>
        </div>
    }
}
