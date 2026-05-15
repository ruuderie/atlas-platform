import os
import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Refactor `match res { Some(Ok(v)) => ..., _ => ... }`
    # We must match the res binding or resource.get() and the closing brace.
    # Pattern to match:
    # {match <expr> {
    #     Some(Ok(<var>)) => <success_arm>,
    #     _ => view! { <err_view> }.into_any(),
    # }}
    
    # We'll use a robust regex that relies on the structure.
    # The `_ => view! { ... }.into_any()` is the hallmark.
    
    # Note: Sometimes there is already `None => view! ...` from a previous patch.
    # But checking the file, they are mostly `_ =>`.

    # Regex to find match blocks:
    # match (\w+(?:\.get\(\))?) \{\s*Some\(Ok\(([\w_]+)\)\) => (.*?),?\s*_ => view! \{ (.*?) \}\.into_any\(\),?\s*\}\}?
    # We need to account for nested braces in success_arm. It's safer to just do a naive substitution for the known text block.

    # Instead of full regex, let's use string manipulation based on `_ => view! { <...> }.into_any()`
    # Let's search for `match res {` or `match stats_resource.get() {`
    
    lines = content.split('\n')
    new_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Match start
        m_start = re.search(r'match ([\w\.]+get\(\)|res) \{', line)
        if m_start:
            expr = m_start.group(1)
            # Find the Some(Ok(...)) and _ => arms
            # We will read ahead until we find `_ => view!`
            buffer = []
            j = i + 1
            found_ok = False
            ok_var = ""
            ok_arm = []
            err_view = ""
            found_err = False
            
            while j < len(lines):
                buffer.append(lines[j])
                if "Some(Ok(" in lines[j] and "=>" in lines[j]:
                    found_ok = True
                    m_ok = re.search(r'Some\(Ok\(([\w_]+)\)\)\s*=>', lines[j])
                    if m_ok:
                        ok_var = m_ok.group(1)
                
                if "_ => view!" in lines[j] and ".into_any()" in lines[j]:
                    found_err = True
                    m_err = re.search(r'_ => view! \{ (.*?) \}\.into_any\(\)', lines[j])
                    if m_err:
                        err_view = m_err.group(1)
                    break
                    
                if "}" in lines[j] and "match" not in lines[j] and not found_ok:
                    # Not a match block we care about
                    break
                j += 1
                
            if found_ok and found_err and j + 1 < len(lines) and lines[j+1].strip().startswith('}'):
                # We successfully grabbed the block.
                # Reconstruct it
                prefix = line[:line.find("match ")]
                new_lines.append(prefix + f"match shared_ui::utils::resource_state::ResourceState::from({expr}) {{")
                
                # Now extract the ok_arm from the buffer
                # Everything between Some(Ok(..)) and _ => view!
                ok_arm_str = "\n".join(buffer[:-1]).replace(f"Some(Ok({ok_var})) =>", f"shared_ui::utils::resource_state::ResourceState::Ready({ok_var}) =>")
                new_lines.append(ok_arm_str)
                
                # Insert the Loading and Error states
                # If the error view is a <tr>, the loading view should be a <tr class="hidden"></tr>
                # Else it should be <div class="hidden"></div>
                if "<tr" in err_view:
                    loading_view = '<tr class="hidden"></tr>'
                elif "<option" in err_view:
                    loading_view = '<option class="hidden" disabled=true></option>'
                elif "<span" in err_view:
                    loading_view = '<span class="hidden"></span>'
                else:
                    loading_view = '<div class="hidden"></div>'
                    
                indent = " " * (len(line) - len(line.lstrip()) + 4)
                new_lines.append(f"{indent}shared_ui::utils::resource_state::ResourceState::Loading => view! {{ {loading_view} }}.into_any(),")
                new_lines.append(f"{indent}shared_ui::utils::resource_state::ResourceState::Error(_) => view! {{ {err_view} }}.into_any(),")
                
                # We skip to j
                i = j
                continue
        
        new_lines.append(line)
        i += 1
        
    with open(filepath, 'w') as f:
        f.write("\n".join(new_lines))

files_to_process = [
    "apps/anchor/src/pages/admin.rs",
    "apps/anchor/src/components/admin_modal.rs",
    "apps/anchor/src/pages/legacy/services.rs",
    "apps/anchor/src/pages/book.rs",
    "apps/anchor/src/pages/legal.rs",
    "apps/platform-admin/src/pages/app_dashboard.rs",
    "apps/platform-admin/src/pages/crm_detail.rs",
    "apps/network-instance/src/pages/search.rs",
    "apps/shared-ui/src/components/ui/related_list.rs"
]

for fp in files_to_process:
    if os.path.exists(fp):
        print(f"Processing {fp}")
        process_file(fp)
    else:
        print(f"Not found: {fp}")
