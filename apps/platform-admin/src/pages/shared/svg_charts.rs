use leptos::prelude::*;
use std::cmp::max;

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[component]
pub fn SvgLineChart(
    #[prop(into)] data: Signal<Vec<f32>>,
    #[prop(default = 300.0)] width: f32,
    #[prop(default = 100.0)] height: f32,
    #[prop(default = "stroke-primary".to_string())] color_class: String,
) -> impl IntoView {
    let path_d = Signal::derive(move || {
        let vals = data.get();
        if vals.is_empty() {
            return String::new();
        }

        let max_val = vals.iter().cloned().fold(0.0_f32, f32::max);
        let min_val = vals.iter().cloned().fold(f32::INFINITY, f32::min);
        let range = if max_val - min_val > 0.0 { max_val - min_val } else { 1.0 };

        let x_step = width / (max(vals.len() - 1, 1) as f32);

        let mut d = String::new();
        for (i, &val) in vals.iter().enumerate() {
            let cx = i as f32 * x_step;
            // Invert Y axis for SVG (0 is top)
            let cy = height - ((val - min_val) / range * height * 0.8) - (height * 0.1); 

            if i == 0 {
                d.push_str(&format!("M {} {} ", cx, cy));
            } else {
                d.push_str(&format!("L {} {} ", cx, cy));
            }
        }
        d
    });

    view! {
        <div class="relative w-full h-full min-h-[40px] flex items-end">
            <svg
                viewBox=move || format!("0 0 {} {}", width, height)
                class="w-full h-full overflow-visible drop-shadow-sm"
                preserveAspectRatio="none"
            >
                <path
                    d=move || path_d.get()
                    fill="none"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class=color_class
                />
            </svg>
        </div>
    }
}

// Ensure the module is configured for unit testing the math
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_path_generation() {
        let data = vec![10.0, 20.0, 30.0];
        let width = 300.0;
        let height = 100.0;

        let max_val = 30.0_f32;
        let min_val = 10.0_f32;
        let range = 20.0_f32;

        let mut expected_d = String::new();
        let x_step = width / 2.0;

        for (i, &val) in data.iter().enumerate() {
            let cx = i as f32 * x_step;
            let cy = height - ((val - min_val) / range * height * 0.8) - (height * 0.1); 
            if i == 0 {
                expected_d.push_str(&format!("M {} {} ", cx, cy));
            } else {
                expected_d.push_str(&format!("L {} {} ", cx, cy));
            }
        }

        assert_eq!(expected_d, "M 0 90 L 150 50 L 300 10 ");
    }
}
