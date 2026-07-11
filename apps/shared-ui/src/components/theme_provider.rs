use leptos::prelude::*;

/// Provides per-tenant theming by injecting dynamic CSS custom properties.
///
/// This component handles **runtime** color overrides (e.g., a tenant's brand
/// color). Static design token mappings from the host app's design system to
/// shared-ui's semantic tokens are handled separately in the app's CSS file
/// via the "shared-ui Token Bridge" `:root` block.
///
/// See: `apps/shared-ui/THEMING.md` for the full architecture guide.
///
/// # Usage
/// ```rust
/// // In your app's root layout:
/// let brand = Signal::derive(move || tenant.get().brand_color);
/// view! {
///     <ThemeProvider primary_color=brand>
///         <App/>
///     </ThemeProvider>
/// }
/// ```
///
/// # What this injects
/// - `--color-primary` + `--brand-primary` — resolved from `primary_color` prop
/// - Derived foreground + tint vars computed from the primary color
///
/// For a static palette, you do NOT need ThemeProvider. Just ensure the
/// "shared-ui Token Bridge" block is present in your app's root CSS.
#[component]
pub fn ThemeProvider(
    /// The tenant's brand/primary color. Can be a hex string (#2079f7),
    /// an RGB string, or any valid CSS color value.
    #[prop(into)]
    primary_color: Signal<String>,
    /// Optional: override the primary-foreground color (text on primary bg).
    /// Defaults to white (#ffffff).
    #[prop(into, optional, default = "#ffffff".to_string())]
    primary_foreground: String,
    children: Children,
) -> impl IntoView {
    let pf = primary_foreground.clone();
    view! {
        <style id="atlas-platform-theme">
            {move || {
                let color = primary_color.get();
                let pf_cloned = pf.clone();
                format!(
                    ":root {{ \
                        --color-primary: {color}; \
                        --color-primary-foreground: {pf_cloned}; \
                        --brand-primary: {color}; \
                    }}",
                    color = color,
                    pf_cloned = pf_cloned,
                )
            }}
        </style>
        {children()}
    }
}
